use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NgReferences {
    pub used_in_templates: Vec<String>,
    pub used_in_class:Vec<String>,
}


impl NgReferences {
    pub fn new() -> Self {
        Self {
            used_in_templates: Vec::new(),
            used_in_class:Vec::new(),
        }
    }
}

impl Default for NgReferences {
    fn default() -> Self {
        Self::new()
    }
}