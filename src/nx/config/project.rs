use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct NxProjectConfig {
    pub name: String,
    #[serde(rename = "sourceRoot")]
    pub source_root: String,
    pub prefix: Option<String>,
    pub tags: Option<Vec<String>>,
    #[serde(rename = "projectType")]
    pub project_type: String,
}
