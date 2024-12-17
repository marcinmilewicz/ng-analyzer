use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NgReferences {
    pub used_by_template: Vec<String>,
    pub used_by_imports: Vec<String>,
}

impl NgReferences {
    pub fn new() -> Self {
        Self {
            used_by_template: Vec::new(),
            used_by_imports: Vec::new(),
        }
    }
}

impl Default for NgReferences {
    fn default() -> Self {
        Self::new()
    }
}
