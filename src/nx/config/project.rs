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

impl NxProjectConfig {
    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_source_root(&self) -> &str {
        &self.source_root
    }

    pub fn get_prefix(&self) -> Option<&str> {
        self.prefix.as_deref()
    }

    pub fn get_tags(&self) -> Option<&Vec<String>> {
        self.tags.as_ref()
    }

    pub fn get_project_type(&self) -> &str {
        &self.project_type
    }
}
