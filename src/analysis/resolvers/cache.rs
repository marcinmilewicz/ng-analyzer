use crate::analysis::models::import::ResolvedImport;
use dashmap::DashMap;
use std::path::Path;
use std::sync::Arc;

/// Cache of resolved imports shared across threads.
///
/// The key must include the importing file's directory, for EVERY kind of
/// specifier — module resolution is defined relative to the importing file and
/// nothing about it is workspace-global:
///
/// * `./model` obviously means something different in every directory;
/// * a bare specifier walks up the directory chain looking for `node_modules`,
///   so `@tanstack/react-query` resolves to `apps/x/node_modules/...` from
///   inside that app and to the root `node_modules/...` from a lib — an npm
///   workspace with a nested install hits this constantly;
/// * a tsconfig alias is resolved against the `paths`/`baseUrl` of the project
///   that owns the file, and projects may override both.
///
/// Sharing one bucket across directories does not merely lose precision: the
/// cache is filled from several threads, so whichever file resolves first wins
/// for everyone and the whole report changes between runs. Determinism (NFR-2)
/// is not negotiable — a baseline or a snapshot test is worthless without it.
pub struct ImportCache {
    cache: Arc<DashMap<String, DashMap<String, ResolvedImport>>>,
}

impl ImportCache {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(DashMap::new()),
        }
    }

    fn key(import_path: &str, current_file: &Path) -> String {
        let dir = current_file
            .parent()
            .map(|parent| parent.to_string_lossy().to_string())
            .unwrap_or_default();
        format!("{}\u{0}{}", dir, import_path)
    }

    pub fn get(
        &self,
        import_path: &str,
        current_file: &Path,
        name: &str,
    ) -> Option<ResolvedImport> {
        self.cache
            .get(&Self::key(import_path, current_file))
            .and_then(|inner| inner.get(name).map(|import| import.clone()))
    }

    pub fn insert(
        &self,
        import_path: &str,
        current_file: &Path,
        name: String,
        resolved: ResolvedImport,
    ) {
        self.cache
            .entry(Self::key(import_path, current_file))
            .or_default()
            .insert(name, resolved);
    }

    pub fn clone_cache(&self) -> Self {
        Self {
            cache: Arc::clone(&self.cache),
        }
    }
}

impl Clone for ImportCache {
    fn clone(&self) -> Self {
        self.clone_cache()
    }
}
