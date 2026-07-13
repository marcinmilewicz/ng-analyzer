use crate::analysis::models::import::{ResolvedImport, UnresolvedImport};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Framework-agnostic facts about a single source file — the foundation for
/// unused-code detection, package statistics and move-candidate analysis.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileFactsInfo {
    pub path: PathBuf,
    pub package_name: String,
    /// Everything this file exports (including re-exports).
    pub exports: Vec<ExportInfo>,
    /// Static imports resolved to their declaring files.
    pub imports: Vec<ResolvedImport>,
    /// `import('...')` expressions — lazy edges (Angular routes, React.lazy).
    pub dynamic_imports: Vec<ResolvedImport>,
    /// Local names of imports actually referenced in the file body
    /// (identifier or type usage — covers DI constructor types). An import
    /// whose local name is absent here is a leftover statement: it must NOT
    /// count as a usage of the symbol it names.
    pub used_import_names: Vec<String>,
    /// Specifiers the resolver could not map to a file — each one is an edge
    /// missing from the graph. Deduplicated per file.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub unresolved_imports: Vec<UnresolvedImport>,
    /// JSX component usages (React .tsx files).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub jsx_usages: Vec<crate::analysis::models::react::JsxUsageInfo>,
    /// Same-file references between top-level declarations — a union member
    /// referenced by an exported union type is alive when the union is, even
    /// though nobody imports the member directly.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub local_references: Vec<LocalReference>,
}

/// `from` declares a top-level name (`""` = top-level statements, which run
/// on module load); `to` lists the OTHER top-level names it references.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LocalReference {
    pub from: String,
    pub to: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ExportInfo {
    pub name: String,
    pub kind: ExportKind,
    /// Specifier a re-export forwards to (`export { X } from './y'`).
    ///
    /// `None` means the name is declared in THIS file — including the
    /// `class X {}; export { X }` spelling, which shares `ExportKind::ReExport`
    /// with a true re-export but is nothing like it: a file full of those
    /// declares its own symbols, whereas a file of true re-exports is a
    /// pass-through that can never receive an inbound edge.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from_module: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum ExportKind {
    Class,
    Function,
    Variable,
    Interface,
    TypeAlias,
    Enum,
    /// `export { X } from './y'` / `export { X }` — tell the two apart with
    /// `ExportInfo::from_module`.
    ReExport,
    /// `export * from './y'`
    ReExportAll,
    Default,
}
