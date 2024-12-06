use crate::ng::models::ng_base::NgBaseInfo;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NgModuleInfo {
    #[serde(flatten)]
    pub base: NgBaseInfo,
    pub declarations: Vec<String>,
    pub exports: Vec<String>,
    pub providers: Vec<String>,
    pub bootstrap: Vec<String>,
}

impl NgModuleInfo {
    pub fn new(
        class_name: String,
        declarations: Vec<String>,
        exports: Vec<String>,
        providers: Vec<String>,
        source_path: PathBuf,
        relative_path: String,
        package_name: String,
    ) -> Self {
        let base = NgBaseInfo::new(
            class_name,
            Vec::new(),
            source_path,
            relative_path,
            package_name,
        );

        Self {
            base,
            declarations,
            exports,
            providers,
            bootstrap: Vec::new(),
        }
    }
}
