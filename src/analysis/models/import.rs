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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ImportKind {
    Named,
    Default,
    Namespace,
    /// `import './polyfills'` — no local binding; runs the module for its
    /// top-level effects and keeps it (but none of its exports) alive.
    SideEffect,
}

impl fmt::Display for ResolvedImport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?} from {}", self.imported_item, self.source)?;

        if self.resolved_path != *"unknown" {
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
            ImportKind::SideEffect => write!(f, "side-effect"),
        }
    }
}

/// An import specifier the resolver could not map to a file. Every one of
/// these is a dependency edge missing from the graph — and a missing edge is
/// how a live symbol gets reported as dead. The count is the trust metric for
/// the whole `unused` analysis.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct UnresolvedImport {
    pub specifier: String,
    pub scope: UnresolvedScope,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum UnresolvedScope {
    /// Relative/absolute path, or a bare specifier matching a tsconfig alias:
    /// it points INSIDE the workspace, so the lost edge corrupts the graph.
    Internal,
    /// A bare specifier with no matching alias — an npm package that is not
    /// installed or has no resolvable entry. Harmless for dead-code analysis:
    /// nothing we own is declared there.
    External,
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
