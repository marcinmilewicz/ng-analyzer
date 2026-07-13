use crate::ng::models::ng_base::NgBaseInfo;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NgDirectiveInfo {
    #[serde(flatten)]
    pub base: NgBaseInfo,
    pub selector: String,
    pub standalone: bool,
    pub host_bindings: Vec<String>,
    pub host_listeners: Vec<String>,
    #[serde(default)]
    pub inputs: Vec<String>,
    #[serde(default)]
    pub outputs: Vec<String>,
}
