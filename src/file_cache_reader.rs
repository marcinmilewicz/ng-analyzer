use dashmap::DashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Read-once file cache shared across threads. The analysis is a single
/// batch run, so entries never expire.
pub struct CachedFileReader {
    cache: Arc<DashMap<PathBuf, String>>,
}

impl CachedFileReader {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(DashMap::new()),
        }
    }

    pub fn read_file(&self, path: &Path) -> io::Result<String> {
        if let Some(cached) = self.cache.get(path) {
            return Ok(cached.clone());
        }

        let content = fs::read_to_string(path)?;
        self.cache.insert(path.to_path_buf(), content.clone());
        Ok(content)
    }

    pub fn clone_cache(&self) -> Self {
        Self {
            cache: Arc::clone(&self.cache),
        }
    }
}

impl Clone for CachedFileReader {
    fn clone(&self) -> Self {
        self.clone_cache()
    }
}
