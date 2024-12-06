use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::PathBuf;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedImport {
    pub source: String,         // Original import source
    pub resolved_path: PathBuf, // Absolute resolved path
    pub import_type: ImportType,
    pub imported_item: ImportedItem,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImportKind {
    Named,
    Default,
    Namespace,
}

impl fmt::Display for ResolvedImport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?} from {}", self.imported_item, self.source)?;

        if self.resolved_path != PathBuf::from("unknown") {
            write!(f, " [resolved: {}]", self.resolved_path.display())?;
        }

        Ok(())
    }
}

impl fmt::Display for ImportKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ImportKind::Named => write!(f, "named"),
            ImportKind::Default => write!(f, "default"),
            ImportKind::Namespace => write!(f, "namespace"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImportType {
    Relative,   // ./path or ../path
    Absolute,   // /path
    Package,    // @angular/core etc
    NodeModule, // Regular node_module import
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportedItem {
    pub name: String,
    pub alias: Option<String>,
    pub import_kind: ImportKind,
}
