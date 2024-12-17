use crate::analysis::models::import::ResolvedImport;
use crate::ng::models::ng_base::NgBaseInfo;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NgTestSpecInfo {
    #[serde(flatten)]
    pub base: NgBaseInfo,
}

impl NgTestSpecInfo {
    pub fn new(
        name: String,
        imports: Vec<ResolvedImport>,
        source_path: PathBuf,
        relative_path: String,
        package_name: String,
    ) -> Self {
        let base = NgBaseInfo::new(name, imports, source_path, relative_path, package_name);

        Self { base }
    }
}
