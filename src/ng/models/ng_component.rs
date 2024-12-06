use crate::analysis::models::import::ResolvedImport;
use crate::ng::models::ng_base::NgBaseInfo;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NgComponentInfo {
    #[serde(flatten)]
    pub base: NgBaseInfo,
    pub selector: String,
    pub template_path: String,
    pub style_paths: Vec<String>,
    pub standalone: bool,
}

impl NgComponentInfo {
    pub fn new(
        name: String,
        selector: String,
        template_path: String,
        style_paths: Vec<String>,
        standalone: bool,
        imports: Vec<ResolvedImport>,
        source_path: PathBuf,
        relative_path: String,
        package_name: String,
    ) -> Self {
        let base = NgBaseInfo::new(name, imports, source_path, relative_path, package_name);

        Self {
            base,
            selector,
            template_path,
            style_paths,
            standalone,
        }
    }
}
