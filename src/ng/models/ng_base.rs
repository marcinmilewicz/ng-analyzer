use crate::analysis::models::import::ResolvedImport;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NgBaseInfo {
    pub name: String,
    pub imports: Vec<ResolvedImport>,
    pub source_path: PathBuf,
    pub relative_path: String,
    pub package_name: String,
}

impl NgBaseInfo {
    pub fn new(
        name: String,
        imports: Vec<ResolvedImport>,
        source_path: PathBuf,
        relative_path: String,
        package_name: String,
    ) -> Self {
        let base = Self {
            name,
            imports,
            source_path,
            relative_path,
            package_name,
        };
        base
    }
}
