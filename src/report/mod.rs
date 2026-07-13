pub mod baseline;
pub mod graph_export;
pub mod html;
pub mod sarif;
pub mod terminal;
pub mod usages;

use crate::analyses::AnalysesSection;
use crate::analysis::resolvers::import_graph::ImportGraphSnapshot;
use crate::ng::models::NgAnalysisResults;
use crate::ng::templates::TemplateUsageInfo;
use serde::Serialize;

/// Complete analysis output — every subcommand renders a slice of this.
#[derive(Serialize)]
pub struct FullReport {
    #[serde(flatten)]
    pub results: NgAnalysisResults,
    pub template_usages: Vec<TemplateUsageInfo>,
    pub import_graph: ImportGraphSnapshot,
    pub analysis: AnalysesSection,
}
