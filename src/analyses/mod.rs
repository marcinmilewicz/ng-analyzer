pub mod boundaries;
pub mod move_candidates;
pub mod project_map;
pub mod react_usage;
pub mod resolution;
pub mod stats;
pub mod unused;

use crate::analysis::resolvers::import_graph::ImportGraph;
use crate::ng::models::NgAnalysisResults;
use crate::ng::templates::TemplateUsageInfo;
use project_map::ProjectCatalog;
use serde::Serialize;
use std::path::Path;

/// All derived analyses over the collected facts.
#[derive(Serialize)]
pub struct AnalysesSection {
    /// Completeness of the symbol graph — read this BEFORE trusting `unused`.
    pub resolution: resolution::ResolutionHealth,
    pub stats: stats::StatsReport,
    pub unused: unused::UnusedReport,
    pub move_candidates: Vec<move_candidates::MoveCandidate>,
    pub boundary_violations: Vec<boundaries::BoundaryViolation>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub react_usage: Vec<react_usage::ReactComponentUsage>,
}

pub fn run_analyses(
    results: &NgAnalysisResults,
    template_usages: &[TemplateUsageInfo],
    import_graph: &ImportGraph,
    catalog: &ProjectCatalog,
    workspace_root: &Path,
) -> AnalysesSection {
    let resolution = resolution::check_resolution(results, catalog);
    let stats = stats::build_stats(results, template_usages, catalog);
    let unused = unused::find_unused(results, template_usages, import_graph, catalog);
    let move_candidates = move_candidates::find_move_candidates(results, catalog);
    let config = boundaries::load_config(workspace_root);
    let boundary_violations = boundaries::check_boundaries(&stats.dependencies, catalog, &config);
    let react_usage = react_usage::analyze_react_usage(results);

    AnalysesSection {
        resolution,
        stats,
        unused,
        move_candidates,
        boundary_violations,
        react_usage,
    }
}
