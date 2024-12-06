use crate::analysis::models::import::ResolvedImport;
use crate::ng::models::ng_base::NgBaseInfo;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NgDirectiveInfo {
    #[serde(flatten)]
    pub base: NgBaseInfo,
    pub selector: String,
    pub standalone: bool,
    pub host_bindings: Vec<String>,
    pub host_listeners: Vec<String>,
}

impl NgDirectiveInfo {
    pub fn new(
        class_name: String,
        selector: String,
        standalone: bool,
        host_bindings: Vec<String>,
        host_listeners: Vec<String>,
        imports: Vec<ResolvedImport>,
        source_path: PathBuf,
        relative_path: String,
        package_name: String,
    ) -> Self {
        let base = NgBaseInfo::new(
            class_name,
            imports,
            source_path,
            relative_path,
            package_name,
        );

        Self {
            base,
            selector,
            standalone,
            host_bindings,
            host_listeners,
        }
    }
}
