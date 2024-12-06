use crate::ng::models::ng_base::NgBaseInfo;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use crate::analysis::models::import::ResolvedImport;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NgServiceInfo {
    #[serde(flatten)]
    pub base: NgBaseInfo,
    pub provided_in: String,
}

impl NgServiceInfo {
    pub fn new(
        class_name: String,
        provided_in: String,
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
            provided_in,
        }
    }
}