use super::{NgComponentInfo, NgModuleInfo, NgServiceInfo};
use crate::analysis::models::file_facts::FileFactsInfo;
use crate::analysis::models::react::ReactComponentInfo;
use crate::analysis::resolvers::import_resolver::ImportResolver;
use crate::ng;

use crate::analysis::processor::collector::AnalysisCollector;
use crate::analysis::processor::context::AnalysisContext;
use crate::ng::models::ng_directive::NgDirectiveInfo;
use crate::ng::models::ng_pipe::NgPipeInfo;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct NgAnalysisResults {
    pub components: Vec<NgComponentInfo>,
    pub directives: Vec<NgDirectiveInfo>,
    pub modules: Vec<NgModuleInfo>,
    pub pipes: Vec<NgPipeInfo>,
    pub services: Vec<NgServiceInfo>,
    /// Framework-agnostic per-file facts (exports, imports, dynamic imports).
    #[serde(default)]
    pub source_files: Vec<FileFactsInfo>,
    /// React function components found in .tsx files.
    #[serde(default)]
    pub react_components: Vec<ReactComponentInfo>,
}

impl NgAnalysisResults {
    /// Sorts every collection so the output is deterministic regardless of
    /// filesystem walk order and parallel scheduling.
    pub fn sort_deterministic(&mut self) {
        self.components.sort_by(|a, b| {
            (&a.base.source_path, &a.base.name).cmp(&(&b.base.source_path, &b.base.name))
        });
        self.directives.sort_by(|a, b| {
            (&a.base.source_path, &a.base.name).cmp(&(&b.base.source_path, &b.base.name))
        });
        self.modules.sort_by(|a, b| {
            (&a.base.source_path, &a.base.name).cmp(&(&b.base.source_path, &b.base.name))
        });
        self.pipes.sort_by(|a, b| {
            (&a.base.source_path, &a.base.name).cmp(&(&b.base.source_path, &b.base.name))
        });
        self.services.sort_by(|a, b| {
            (&a.base.source_path, &a.base.name).cmp(&(&b.base.source_path, &b.base.name))
        });
        self.source_files.sort_by(|a, b| a.path.cmp(&b.path));
        self.react_components
            .sort_by(|a, b| (&a.source_path, &a.name).cmp(&(&b.source_path, &b.name)));
    }
}

impl AnalysisCollector for NgAnalysisResults {
    fn extend(&mut self, other: Self) {
        self.components.extend(other.components);
        self.services.extend(other.services);
        self.modules.extend(other.modules);
        self.directives.extend(other.directives);
        self.pipes.extend(other.pipes);
        self.source_files.extend(other.source_files);
        self.react_components.extend(other.react_components);
    }

    fn process_file(
        path: &Path,
        resolver: &mut ImportResolver,
        context: &AnalysisContext,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        ng::visitors::analyze_file(
            path,
            &context.project_path,
            &context.source_map,
            context.project_name.as_ref().clone(),
            context.project_ts_config.clone(),
            context.default_standalone,
            resolver,
            &context.file_reader,
        )
    }
}
