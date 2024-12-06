use crate::analysis::models::import::ResolvedImport;
use dashmap::DashMap;
use std::sync::Arc;

pub struct ImportCache {
    cache: Arc<DashMap<String, DashMap<String, ResolvedImport>>>
}

impl ImportCache {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(DashMap::new())
        }
    }

    pub fn get(&self, import_path: &str, name: &str) -> Option<ResolvedImport> {
        if let Some(inner_map) = self.cache.get(import_path) {
            if let Some(import) = inner_map.get(name) {
                return Some(import.clone());
            }
        }
        None
    }

    pub fn insert(&self, import_path: String, name: String, path: ResolvedImport) {
        self.cache
            .entry(import_path)
            .or_insert_with(DashMap::new)
            .insert(name, path);
    }

    pub fn clone_cache(&self) -> Self {
        Self {
            cache: Arc::clone(&self.cache)
        }
    }
}