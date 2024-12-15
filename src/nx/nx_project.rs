use crate::analysis::models::ts_config::TSConfig;
use crate::nx::config::NxProjectConfig;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct NxProject {
    pub name: String,
    pub config: NxProjectConfig,
    pub ts_config: TSConfig,
    pub files: HashSet<PathBuf>,
}

impl NxProject {
    pub fn with_files(
        name: String,
        config: NxProjectConfig,
        ts_config: TSConfig,
        files: HashSet<PathBuf>,
    ) -> Self {
        Self {
            name,
            config,
            ts_config,
            files,
        }
    }
}
