use serde::{Deserialize, Serialize};
use std::path::Path;

/// Raw shape of `project.json`. In NX both `name` and `sourceRoot` are optional
/// (inferred from the project directory when absent), so we must not fail
/// deserialization when they are missing.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct NxProjectConfig {
    pub name: Option<String>,
    #[serde(rename = "sourceRoot")]
    pub source_root: Option<String>,
    pub prefix: Option<String>,
    pub tags: Option<Vec<String>>,
    #[serde(rename = "projectType")]
    pub project_type: Option<String>,
}

impl NxProjectConfig {
    /// Project name: explicit `name` field, otherwise the directory name.
    pub fn resolved_name(&self, project_root: &Path) -> String {
        self.name.clone().unwrap_or_else(|| {
            project_root
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "unknown".to_string())
        })
    }
}
