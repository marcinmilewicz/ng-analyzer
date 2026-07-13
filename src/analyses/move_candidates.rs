use crate::analyses::project_map::{is_test_file, ProjectCatalog};
use crate::analysis::models::file_facts::ExportKind;
use crate::analysis::models::import::ImportKind;
use crate::ng::models::NgAnalysisResults;
use serde::Serialize;
use std::collections::HashMap;
use std::path::PathBuf;

/// A symbol whose only consumers live in a single other project — moving it
/// there would cut a package dependency.
#[derive(Debug, Serialize)]
pub struct MoveCandidate {
    pub symbol: String,
    pub file: PathBuf,
    pub from_project: String,
    pub to_project: String,
    pub external_usages: usize,
    pub internal_usages: usize,
}

pub fn find_move_candidates(
    results: &NgAnalysisResults,
    catalog: &ProjectCatalog,
) -> Vec<MoveCandidate> {
    // (declaring file, exported name) -> using project -> count.
    // Test files are ignored: a symbol used by another project's tests only
    // is not worth moving.
    let mut usage_by_project: HashMap<(PathBuf, String), HashMap<String, usize>> = HashMap::new();

    for file in &results.source_files {
        if is_test_file(&file.path) {
            continue;
        }
        let Some(from) = catalog.project_of(&file.path) else {
            continue;
        };
        for import in &file.imports {
            // `import './x'` names no symbol — it cannot make one a candidate
            // for moving, and its empty binding would pollute the usage keys.
            if import.imported_item.import_kind == ImportKind::SideEffect {
                continue;
            }
            let exported = import
                .imported_item
                .alias
                .clone()
                .unwrap_or_else(|| import.imported_item.name.clone());
            *usage_by_project
                .entry((import.resolved_path.clone(), exported))
                .or_default()
                .entry(from.name.clone())
                .or_insert(0) += 1;
        }
    }

    let mut candidates = Vec::new();

    for file in &results.source_files {
        let Some(home) = catalog.project_of(&file.path) else {
            continue;
        };
        for export in &file.exports {
            if matches!(export.kind, ExportKind::ReExport | ExportKind::ReExportAll) {
                continue;
            }
            let Some(by_project) = usage_by_project.get(&(file.path.clone(), export.name.clone()))
            else {
                continue;
            };

            let internal = by_project.get(&home.name).copied().unwrap_or(0);
            let external: Vec<(&String, &usize)> = by_project
                .iter()
                .filter(|(project, _)| *project != &home.name)
                .collect();

            // The whole point: used exactly by ONE other project and not at
            // home (barrel re-exports don't count as home usage).
            if internal == 0 && external.len() == 1 {
                let (to_project, count) = external[0];
                candidates.push(MoveCandidate {
                    symbol: export.name.clone(),
                    file: file.path.clone(),
                    from_project: home.name.clone(),
                    to_project: to_project.clone(),
                    external_usages: *count,
                    internal_usages: internal,
                });
            }
        }
    }

    candidates.sort_by(|a, b| (&a.file, &a.symbol).cmp(&(&b.file, &b.symbol)));
    candidates
}
