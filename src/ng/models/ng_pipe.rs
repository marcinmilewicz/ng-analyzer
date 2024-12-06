use crate::analysis::models::import::ResolvedImport;
use crate::ng::models::ng_base::NgBaseInfo;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NgPipeInfo {
    #[serde(flatten)]
    pub base: NgBaseInfo,
    pub name: String,
    pub pure: bool,
    pub standalone: bool,
}

impl NgPipeInfo {
    pub fn new(
        class_name: String,
        name: String,
        pure: bool,
        standalone: bool,
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
            name,
            pure,
            standalone,
        }
    }
}
