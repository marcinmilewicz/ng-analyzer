use crate::analysis::models::import::{ImportType, UnresolvedScope};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

const TS_EXTENSIONS: &[&str] = &[".ts", ".tsx", ".d.ts"];

/// Where an import that FAILED to resolve was pointing. A bare specifier is
/// only external when no tsconfig alias claims it — `@org/lib-x` with a
/// `paths` entry is a broken internal edge, not a missing npm package, and
/// the two must never be reported in the same bucket.
pub fn classify_unresolved(
    import_path: &str,
    ts_paths: &HashMap<String, Vec<String>>,
) -> UnresolvedScope {
    if import_path.starts_with("./")
        || import_path.starts_with("../")
        || import_path.starts_with('/')
    {
        return UnresolvedScope::Internal;
    }

    let claimed_by_alias = ts_paths.keys().any(|alias| match alias.split_once('*') {
        Some((prefix, suffix)) => {
            import_path.len() >= prefix.len() + suffix.len()
                && import_path.starts_with(prefix)
                && import_path.ends_with(suffix)
        }
        None => alias == import_path,
    });

    if claimed_by_alias {
        UnresolvedScope::Internal
    } else {
        UnresolvedScope::External
    }
}

/// `./x.js` → `x.ts`/`x.tsx`, `./x.mjs` → `x.mts`, `./x.cjs` → `x.cts`
/// (TypeScript NodeNext/bundler module resolution).
pub(crate) fn map_js_specifier_to_ts(path: &std::path::Path) -> Option<std::path::PathBuf> {
    let ext = path.extension()?.to_str()?;
    let candidates: &[&str] = match ext {
        "js" => &["ts", "tsx", "d.ts"],
        "jsx" => &["tsx"],
        "mjs" => &["mts"],
        "cjs" => &["cts"],
        _ => return None,
    };
    for candidate in candidates {
        let mapped = path.with_extension(candidate);
        if mapped.is_file() {
            return Some(mapped);
        }
    }
    None
}

pub struct ImportPathResolver {
    /// Root of the analyzed workspace; tsconfig `paths` and absolute imports
    /// are resolved against it (combined with `baseUrl` when present).
    workspace_root: PathBuf,
}

impl ImportPathResolver {
    pub fn new(workspace_root: PathBuf) -> Self {
        Self { workspace_root }
    }

    pub fn resolve_import(
        &self,
        import_path: &str,
        current_file: &Path,
        ts_paths: &HashMap<String, Vec<String>>,
        base_url: Option<&str>,
    ) -> (Option<PathBuf>, ImportType) {
        if import_path.starts_with("./") || import_path.starts_with("../") {
            return (
                self.resolve_relative_import(import_path, current_file),
                ImportType::Relative,
            );
        }

        if import_path.starts_with('/') {
            return (
                self.resolve_absolute_import(import_path),
                ImportType::Absolute,
            );
        }

        // Bare specifier: tsconfig path aliases take precedence over
        // node_modules, and aliases do NOT have to start with '@'.
        if let Some(resolved) = self.resolve_ts_paths_import(import_path, ts_paths, base_url) {
            return (Some(resolved), ImportType::Package);
        }

        (
            self.resolve_node_module_import(import_path, current_file),
            ImportType::NodeModule,
        )
    }

    /// Resolves `./x` / `../x` against the importing file's directory.
    /// Returns None when nothing exists at the target (no phantom paths).
    pub fn resolve_relative_import(
        &self,
        import_path: &str,
        current_file: &Path,
    ) -> Option<PathBuf> {
        let parent = current_file.parent()?;
        Self::resolve_as_file_or_index(parent.join(import_path))
    }

    pub fn resolve_absolute_import(&self, import_path: &str) -> Option<PathBuf> {
        Self::resolve_as_file_or_index(
            self.workspace_root
                .join(import_path.trim_start_matches('/')),
        )
    }

    fn resolve_ts_paths_import(
        &self,
        import_path: &str,
        ts_paths: &HashMap<String, Vec<String>>,
        base_url: Option<&str>,
    ) -> Option<PathBuf> {
        // `baseUrl` arrives pre-anchored against its declaring tsconfig's
        // directory (see NxWorkspace::absolutize_base_url) — TypeScript
        // resolves it relative to that file, NOT the workspace root. It is
        // used as-is; joining onto workspace_root would double relative
        // roots (`-d tests/fixture` + `tests/fixture/...`).
        let paths_base = match base_url {
            Some(url) => PathBuf::from(url),
            None => self.workspace_root.clone(),
        };

        // Exact aliases first, then longest wildcard prefix (TS semantics).
        if let Some(targets) = ts_paths.get(import_path) {
            for target in targets {
                if let Some(resolved) = Self::resolve_as_file_or_index(paths_base.join(target)) {
                    return Some(resolved);
                }
            }
        }

        let mut wildcard_aliases: Vec<(&String, &Vec<String>)> = ts_paths
            .iter()
            .filter(|(alias, _)| alias.contains('*'))
            .collect();
        wildcard_aliases.sort_by_key(|(alias, _)| std::cmp::Reverse(alias.len()));

        for (alias, targets) in wildcard_aliases {
            let prefix = alias.trim_end_matches('*');
            if let Some(suffix) = import_path.strip_prefix(prefix) {
                for target in targets {
                    let candidate = target.replace('*', suffix);
                    if let Some(resolved) =
                        Self::resolve_as_file_or_index(paths_base.join(candidate))
                    {
                        return Some(resolved);
                    }
                }
            }
        }

        None
    }

    pub fn resolve_node_module_import(
        &self,
        import_path: &str,
        current_file: &Path,
    ) -> Option<PathBuf> {
        let mut current = current_file.parent()?.to_path_buf();
        loop {
            let node_modules = current.join("node_modules");
            if node_modules.is_dir() {
                if let Some(resolved) = Self::resolve_in_node_modules(&node_modules, import_path) {
                    return Some(resolved);
                }
            }
            if !current.pop() {
                return None;
            }
        }
    }

    /// Package name is the first specifier component — the first two for
    /// scoped packages ("@org/pkg/sub" → "@org/pkg" + "sub").
    fn split_package_specifier(import_path: &str) -> (&str, Option<&str>) {
        let name_len = if import_path.starts_with('@') {
            match import_path.match_indices('/').nth(1) {
                Some((i, _)) => i,
                None => import_path.len(),
            }
        } else {
            import_path.find('/').unwrap_or(import_path.len())
        };
        let subpath = import_path[name_len..].strip_prefix('/');
        (&import_path[..name_len], subpath.filter(|s| !s.is_empty()))
    }

    fn resolve_in_node_modules(node_modules: &Path, import_path: &str) -> Option<PathBuf> {
        let (package_name, subpath) = Self::split_package_specifier(import_path);
        let linked_dir = node_modules.join(package_name);
        if !linked_dir.exists() {
            return None;
        }

        // npm/pnpm workspace packages are symlinks back into the workspace
        // (node_modules/@org/x → ../../libs/x). Resolve them LEXICALLY so the
        // result keeps the workspace-root-relative spelling every other
        // analyzed path uses — fs::canonicalize would force absolute paths.
        let package_dir = match std::fs::read_link(&linked_dir) {
            Ok(target) if target.is_absolute() => {
                crate::analysis::utils::path_utils::normalize_path(target)
            }
            // A relative link target is relative to the link's own parent —
            // node_modules/@org for scoped packages, node_modules otherwise.
            Ok(target) => crate::analysis::utils::path_utils::normalize_path(
                linked_dir.parent().unwrap_or(node_modules).join(target),
            ),
            Err(_) => linked_dir,
        };

        if let Some(sub) = subpath {
            return Self::resolve_as_file_or_index(package_dir.join(sub))
                .or_else(|| Some(node_modules.join(import_path)).filter(|p| p.exists()));
        }

        // Bare package import: follow the manifest to the entry source, so a
        // barrel there can be traced to declaring files. Opaque packages
        // (no resolvable entry) keep returning their directory.
        Self::package_entry_point(&package_dir)
            .or_else(|| Self::resolve_as_file_or_index(package_dir.clone()))
            .or(Some(package_dir))
    }

    /// Entry file from package.json: `types` > `main` > `exports["."]`
    /// (string, or object's `types`/`import`/`default`). Only returned when
    /// the target actually exists.
    fn package_entry_point(package_dir: &Path) -> Option<PathBuf> {
        let manifest = std::fs::read_to_string(package_dir.join("package.json")).ok()?;
        let manifest: serde_json::Value = serde_json::from_str(&manifest).ok()?;

        let dot_export = manifest.get("exports").map(|exports| {
            let dot = exports.get(".").unwrap_or(exports);
            dot.as_str().map(str::to_string).or_else(|| {
                ["types", "import", "default"]
                    .iter()
                    .find_map(|key| dot.get(key)?.as_str().map(str::to_string))
            })
        });

        [
            manifest
                .get("types")
                .and_then(|v| v.as_str())
                .map(str::to_string),
            manifest
                .get("main")
                .and_then(|v| v.as_str())
                .map(str::to_string),
            dot_export.flatten(),
        ]
        .into_iter()
        .flatten()
        .find_map(|entry| Self::resolve_as_file_or_index(package_dir.join(entry)))
    }

    /// Tries `path` as-is, with TS extensions appended, then as a directory
    /// with `index.*`. NodeNext/bundler-style specifiers ending in `.js`
    /// map back to their TypeScript source. Returns None when nothing exists.
    fn resolve_as_file_or_index(path: PathBuf) -> Option<PathBuf> {
        if path.is_file() {
            return Some(path);
        }

        // ESM style: `./helper.js` written in source, file is helper.ts.
        if let Some(mapped) = map_js_specifier_to_ts(&path) {
            return Some(mapped);
        }

        let display = path.display().to_string();
        for ext in TS_EXTENSIONS {
            let with_ext = PathBuf::from(format!("{}{}", display, ext));
            if with_ext.is_file() {
                return Some(with_ext);
            }
        }

        if path.is_dir() {
            for ext in TS_EXTENSIONS {
                let index = path.join(format!("index{}", ext));
                if index.is_file() {
                    return Some(index);
                }
            }
        }

        None
    }
}
