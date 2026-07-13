use crate::analysis::models::ts_config::TSConfig;
use crate::nx::config::NxProjectConfig;
use crate::nx::nx_project::NxProject;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// tsconfig files tried next to `project.json`, in order of preference.
const TSCONFIG_CANDIDATES: &[&str] = &["tsconfig.json", "tsconfig.lib.json", "tsconfig.app.json"];

/// Workspace-level tsconfig fallbacks when a project has none of its own.
const WORKSPACE_TSCONFIG_CANDIDATES: &[&str] = &["tsconfig.base.json", "tsconfig.json"];

#[derive(Debug)]
pub struct NxWorkspace {
    projects: HashMap<PathBuf, NxProject>,
    workspace_root: PathBuf,
}

impl NxWorkspace {
    pub fn new(workspace_root: PathBuf) -> Self {
        Self {
            projects: HashMap::new(),
            workspace_root,
        }
    }

    pub fn get_projects(&self) -> &HashMap<PathBuf, NxProject> {
        &self.projects
    }

    pub fn workspace_root(&self) -> &Path {
        &self.workspace_root
    }

    pub fn load_configuration(&mut self) -> std::io::Result<()> {
        self.load_projects()?;
        Ok(())
    }

    fn collect_project_files(&self, project_root: &Path) -> HashSet<PathBuf> {
        WalkDir::new(walkable_root(project_root))
            .into_iter()
            .filter_entry(|e| !is_ignored_entry(e))
            .filter_map(|entry| {
                entry.ok().and_then(|e| {
                    if e.file_type().is_file() {
                        Some(crate::analysis::utils::path_utils::normalize_path(e.path()))
                    } else {
                        None
                    }
                })
            })
            .collect()
    }

    fn load_projects(&mut self) -> std::io::Result<()> {
        for entry in WalkDir::new(&self.workspace_root)
            .into_iter()
            .filter_entry(|e| !is_ignored_entry(e))
        {
            let entry = entry?;
            if entry.file_name() != "project.json" {
                continue;
            }

            // Lexically normalized (`./apps/x` → `apps/x`): every path in the
            // analysis must share one canonical spelling, because usage joins
            // key on (file path, symbol) and `project_of` on `starts_with` —
            // resolved imports are normalized the same way.
            let project_root = match entry.path().parent() {
                Some(parent) => crate::analysis::utils::path_utils::normalize_path(parent),
                None => continue,
            };

            match self.parse_project_config(entry.path()) {
                Ok(project_config) => {
                    let tsconfig = self.load_project_tsconfig(&project_root);
                    let files = self.collect_project_files(&project_root);
                    let name = project_config.resolved_name(&project_root);

                    let project = NxProject::with_files(name, project_config, tsconfig, files);
                    self.projects.insert(project_root, project);
                }
                Err(e) => {
                    eprintln!("⚠️ Error parsing project.json at {:?}: {}", entry.path(), e);
                }
            }
        }
        Ok(())
    }

    /// Finds the best tsconfig for a project. Projects without any tsconfig
    /// fall back to the workspace-level one; a missing tsconfig never causes
    /// the project to be skipped.
    fn load_project_tsconfig(&self, project_root: &Path) -> TSConfig {
        let candidates = TSCONFIG_CANDIDATES
            .iter()
            .map(|name| project_root.join(name))
            .chain(
                WORKSPACE_TSCONFIG_CANDIDATES
                    .iter()
                    .map(|name| self.workspace_root.join(name)),
            );

        for candidate in candidates {
            if !candidate.exists() {
                continue;
            }
            match self.parse_tsconfig(&candidate) {
                Ok(config) => {
                    let mut visited = HashSet::new();
                    return self.resolve_extended_tsconfig(&config, &candidate, &mut visited);
                }
                Err(e) => {
                    eprintln!("⚠️ Error parsing tsconfig at {:?}: {}", candidate, e);
                }
            }
        }

        TSConfig {
            compiler_options: None,
            extends: None,
        }
    }

    fn parse_project_config(&self, path: &Path) -> std::io::Result<NxProjectConfig> {
        let content = fs::read_to_string(path)?;
        // project.json / tsconfig files are JSONC in the wild.
        let config: NxProjectConfig =
            serde_json::from_str(&crate::analysis::utils::jsonc::strip_jsonc(&content))
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        Ok(config)
    }

    fn parse_tsconfig(&self, path: &Path) -> std::io::Result<TSConfig> {
        let content = fs::read_to_string(path)?;
        let mut config: TSConfig =
            serde_json::from_str(&crate::analysis::utils::jsonc::strip_jsonc(&content))
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        Self::absolutize_base_url(&mut config, path);
        Ok(config)
    }

    /// TypeScript resolves a relative `baseUrl` against the directory of the
    /// tsconfig file that DECLARES it, not the workspace root — e.g. an app's
    /// `"baseUrl": "../.."` points at the workspace root, not two levels above
    /// it. Anchor it here, while the declaring file is still known; the merge
    /// in `resolve_extended_tsconfig` then combines already-absolute values.
    fn absolutize_base_url(config: &mut TSConfig, config_path: &Path) {
        let Some(options) = &mut config.compiler_options else {
            return;
        };
        let Some(base_url) = &options.base_url else {
            return;
        };
        if Path::new(base_url).is_absolute() {
            return;
        }
        if let Some(config_dir) = config_path.parent() {
            options.base_url = Some(
                crate::analysis::utils::path_utils::normalize_path(config_dir.join(base_url))
                    .to_string_lossy()
                    .into_owned(),
            );
        }
    }

    /// Resolves the `extends` chain recursively (relative paths and
    /// node_modules-style specifiers), merging `paths` and `baseUrl` with
    /// child-wins semantics, matching TypeScript behaviour.
    fn resolve_extended_tsconfig(
        &self,
        config: &TSConfig,
        config_path: &Path,
        visited: &mut HashSet<PathBuf>,
    ) -> TSConfig {
        let mut resolved_config = config.clone();

        let Some(extends_specifier) = &config.extends else {
            return resolved_config;
        };

        let Some(extended_path) = self.resolve_extends_specifier(extends_specifier, config_path)
        else {
            eprintln!(
                "⚠️ Cannot resolve tsconfig extends {:?} from {:?}",
                extends_specifier, config_path
            );
            return resolved_config;
        };

        if !visited.insert(extended_path.clone()) {
            eprintln!("⚠️ Circular tsconfig extends chain at {:?}", extended_path);
            return resolved_config;
        }

        let extended_config = match self.parse_tsconfig(&extended_path) {
            Ok(parsed) => self.resolve_extended_tsconfig(&parsed, &extended_path, visited),
            Err(e) => {
                eprintln!(
                    "⚠️ Error parsing extended tsconfig {:?}: {}",
                    extended_path, e
                );
                return resolved_config;
            }
        };

        if let Some(extended_options) = extended_config.compiler_options {
            match &mut resolved_config.compiler_options {
                Some(current_options) => {
                    if let Some(extended_paths) = extended_options.paths {
                        match &mut current_options.paths {
                            Some(current_paths) => {
                                for (key, value) in extended_paths {
                                    current_paths.entry(key).or_insert(value);
                                }
                            }
                            None => current_options.paths = Some(extended_paths),
                        }
                    }
                    if current_options.base_url.is_none() {
                        current_options.base_url = extended_options.base_url;
                    }
                }
                None => resolved_config.compiler_options = Some(extended_options),
            }
        }

        resolved_config
    }

    /// `extends` may be a relative path ("./tsconfig.base.json") or a package
    /// specifier ("@tsconfig/node18/tsconfig.json"), looked up in node_modules
    /// walking up from the config's directory.
    fn resolve_extends_specifier(&self, specifier: &str, config_path: &Path) -> Option<PathBuf> {
        let base_dir = config_path.parent()?;

        if specifier.starts_with("./") || specifier.starts_with("../") {
            return Self::with_json_extension(base_dir.join(specifier));
        }

        let mut current = base_dir.to_path_buf();
        loop {
            let candidate = current.join("node_modules").join(specifier);
            if let Some(found) = Self::with_json_extension(candidate) {
                return Some(found);
            }
            if !current.pop() {
                return None;
            }
        }
    }

    fn with_json_extension(path: PathBuf) -> Option<PathBuf> {
        if path.exists() {
            return Some(path);
        }
        let with_ext = PathBuf::from(format!("{}.json", path.display()));
        if with_ext.exists() {
            return Some(with_ext);
        }
        None
    }
}

/// Normalized project roots can be empty (a project.json at the workspace
/// root analyzed via `-d .`); WalkDir needs `.` for "current directory".
pub(crate) fn walkable_root(root: &Path) -> &Path {
    if root.as_os_str().is_empty() {
        Path::new(".")
    } else {
        root
    }
}

/// Hidden dirs (.vercel, .next, .ai, …) and build outputs are never NX
/// projects or sources. The walk root itself (depth 0) is never filtered —
/// analyzing a workspace from inside a dotted path must still work.
fn is_ignored_entry(entry: &walkdir::DirEntry) -> bool {
    entry.depth() > 0
        && crate::analysis::utils::path_utils::is_ignored_dir_component(entry.file_name())
}
