use crate::analysis::models::ts_config::TSConfig;
use dashmap::DashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, SystemTime};

pub struct CachedFileReader {
    cache: Arc<DashMap<PathBuf, CachedContent>>,
    cache_duration: Duration,
}

struct CachedContent {
    content: String,
    timestamp: SystemTime,
}

impl CachedFileReader {
    pub fn new(cache_duration: Duration) -> Self {
        Self {
            cache: Arc::new(DashMap::new()),
            cache_duration,
        }
    }

    pub fn read_file(&self, path: &Path) -> io::Result<String> {
        if let Some(cached) = self.cache.get(path) {
            if cached.timestamp.elapsed().unwrap_or(self.cache_duration) < self.cache_duration {
                return Ok(cached.content.clone());
            }
        }

        let content = fs::read_to_string(path)?;

        self.cache.insert(
            path.to_path_buf(),
            CachedContent {
                content: content.clone(),
                timestamp: SystemTime::now(),
            },
        );

        Ok(content)
    }

    pub fn remove_from_cache(&self, path: &Path) {
        self.cache.remove(path);
    }

    pub fn get_cache_stats(&self) -> CacheStats {
        let total_entries = self.cache.len();
        let total_size: usize = self.cache.iter().map(|entry| entry.content.len()).sum();

        CacheStats {
            total_entries,
            total_size,
        }
    }

    pub fn clone_cache(&self) -> Self {
        Self {
            cache: Arc::clone(&self.cache),
            cache_duration: self.cache_duration,
        }
    }
}

pub struct CacheStats {
    pub total_entries: usize,
    pub total_size: usize,
}

pub struct TSConfigCache {
    reader: CachedFileReader,
    parsed_configs: Arc<DashMap<PathBuf, TSConfig>>,
}

impl TSConfigCache {
    pub fn new(cache_duration: Duration) -> Self {
        Self {
            reader: CachedFileReader::new(cache_duration),
            parsed_configs: Arc::new(DashMap::new()),
        }
    }

    pub fn get_config(&self, path: &Path) -> io::Result<TSConfig> {
        if let Some(config) = self.parsed_configs.get(path) {
            return Ok(config.clone());
        }

        let content = self.reader.read_file(path)?;
        let config: TSConfig = serde_json::from_str(&content)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        self.parsed_configs
            .insert(path.to_path_buf(), config.clone());
        Ok(config)
    }
}
