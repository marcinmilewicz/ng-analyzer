use crate::analyses::project_map::ProjectCatalog;
use crate::analysis::models::import::UnresolvedScope;
use crate::ng::models::NgAnalysisResults;
use serde::Serialize;
use std::collections::BTreeMap;
use std::path::PathBuf;

/// Trust metric for every graph-derived analysis.
///
/// A specifier the resolver cannot map to a file is an edge that never enters
/// the graph — and a missing edge is precisely how a live symbol gets reported
/// as dead. `unused` is therefore only as trustworthy as `unresolved_internal`
/// is empty; when it is not, the dead-code findings must be read as a lower
/// bound with unknown false positives, not as a work list.
///
/// External misses (an npm package that is not installed, or has no resolvable
/// entry) are tracked separately and are harmless: nothing this workspace
/// declares lives behind them.
#[derive(Debug, Serialize)]
pub struct ResolutionHealth {
    /// Import specifiers successfully mapped to a file.
    pub resolved_imports: usize,
    /// Unresolved specifiers pointing INSIDE the workspace. Should be zero.
    pub unresolved_internal: Vec<UnresolvedRef>,
    /// Unresolved bare specifiers with no matching tsconfig alias, grouped.
    pub unresolved_external: Vec<UnresolvedPackage>,
}

#[derive(Debug, Serialize)]
pub struct UnresolvedRef {
    pub file: PathBuf,
    pub specifier: String,
    pub project: String,
}

#[derive(Debug, Serialize)]
pub struct UnresolvedPackage {
    pub specifier: String,
    /// Number of files importing it.
    pub files: usize,
}

impl ResolutionHealth {
    /// True when the symbol graph is complete enough for dead-code findings
    /// to be taken at face value.
    pub fn is_trustworthy(&self) -> bool {
        self.unresolved_internal.is_empty()
    }
}

pub fn check_resolution(results: &NgAnalysisResults, catalog: &ProjectCatalog) -> ResolutionHealth {
    let mut unresolved_internal = Vec::new();
    let mut external: BTreeMap<&str, usize> = BTreeMap::new();
    let mut resolved_imports = 0usize;

    for file in &results.source_files {
        resolved_imports += file.imports.len() + file.dynamic_imports.len();

        for unresolved in &file.unresolved_imports {
            match unresolved.scope {
                UnresolvedScope::Internal => unresolved_internal.push(UnresolvedRef {
                    file: file.path.clone(),
                    specifier: unresolved.specifier.clone(),
                    project: catalog
                        .project_of(&file.path)
                        .map(|project| project.name.clone())
                        .unwrap_or_default(),
                }),
                UnresolvedScope::External => {
                    *external.entry(unresolved.specifier.as_str()).or_insert(0) += 1;
                }
            }
        }
    }

    unresolved_internal.sort_by(|a, b| (&a.file, &a.specifier).cmp(&(&b.file, &b.specifier)));

    let mut unresolved_external: Vec<UnresolvedPackage> = external
        .into_iter()
        .map(|(specifier, files)| UnresolvedPackage {
            specifier: specifier.to_string(),
            files,
        })
        .collect();
    // Most-imported first: those are the ones worth installing types for.
    unresolved_external.sort_by(|a, b| {
        b.files
            .cmp(&a.files)
            .then_with(|| a.specifier.cmp(&b.specifier))
    });

    ResolutionHealth {
        resolved_imports,
        unresolved_internal,
        unresolved_external,
    }
}
