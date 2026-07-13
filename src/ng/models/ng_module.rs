use crate::ng::models::ng_base::NgBaseInfo;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NgModuleInfo {
    #[serde(flatten)]
    pub base: NgBaseInfo,
    pub declarations: Vec<String>,
    /// Identifiers from the NgModule `imports: [...]` array.
    #[serde(default)]
    pub imports_idents: Vec<String>,
    pub exports: Vec<String>,
    pub providers: Vec<String>,
    pub bootstrap: Vec<String>,
}
