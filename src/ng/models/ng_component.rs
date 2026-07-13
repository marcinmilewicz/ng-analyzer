use crate::ng::models::ng_base::NgBaseInfo;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NgComponentInfo {
    #[serde(flatten)]
    pub base: NgBaseInfo,
    pub selector: String,
    pub template_path: String,
    /// Inline `template:` content (empty template_path when present).
    #[serde(default)]
    pub template_inline: Option<String>,
    pub style_paths: Vec<String>,
    pub standalone: bool,
    /// Identifiers from the decorator's `imports: [...]` (standalone scope).
    #[serde(default)]
    pub standalone_imports: Vec<String>,
    /// Identifiers from `providers: [...]`.
    #[serde(default)]
    pub providers: Vec<String>,
    /// Input names: `@Input()` properties and signal `input()`/`model()`.
    #[serde(default)]
    pub inputs: Vec<String>,
    /// Output names: `@Output()` properties and signal `output()`.
    #[serde(default)]
    pub outputs: Vec<String>,
}
