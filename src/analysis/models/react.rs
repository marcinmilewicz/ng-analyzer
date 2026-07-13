use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// A React function component found in a .tsx file: a capitalized top-level
/// function or a capitalized const holding an arrow/function (possibly
/// wrapped in `memo(...)` / `forwardRef(...)`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReactComponentInfo {
    pub name: String,
    pub source_path: PathBuf,
    pub package_name: String,
    /// True when wrapped in memo()/forwardRef().
    pub wrapped: bool,
}

/// One `<Component prop1 prop2={...}>` occurrence in JSX.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JsxUsageInfo {
    pub component: String,
    pub props: Vec<String>,
}
