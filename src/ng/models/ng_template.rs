use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TemplateUsage {
    pub components: Vec<String>,
    pub pipes: Vec<String>,
    pub directives: Vec<String>,
}

impl TemplateUsage {
    pub fn new() -> Self {
        Self {
            components: Vec::new(),
            pipes: Vec::new(),
            directives: Vec::new(),
        }
    }

    pub fn default() -> Self {
        Self::new()
    }
}
