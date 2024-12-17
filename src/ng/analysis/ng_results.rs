use crate::analysis::resolvers::import_resolver::ImportResolver;
use crate::ng;
use crate::ng::models::{NgComponentInfo, NgModuleInfo, NgServiceInfo};
use std::collections::HashMap;

use crate::analysis::processor::collector::AnalysisCollector;
use crate::analysis::processor::context::AnalysisContext;
use crate::ng::models::ng_directive::NgDirectiveInfo;
use crate::ng::models::ng_element::NgElement;
use crate::ng::models::ng_other::NgOtherInfo;
use crate::ng::models::ng_pipe::NgPipeInfo;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use crate::ng::models::ng_spec::NgTestSpecInfo;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct NgAnalysisResults {
    base_path: PathBuf,

    pub components: Vec<NgComponentInfo>,
    pub directives: Vec<NgDirectiveInfo>,
    pub modules: Vec<NgModuleInfo>,
    pub others: Vec<NgOtherInfo>,
    pub pipes: Vec<NgPipeInfo>,
    pub services: Vec<NgServiceInfo>,
    pub test_specs: Vec<NgTestSpecInfo>,

    pub component_selector_map: HashMap<String, NgComponentInfo>,
    pub directive_selector_map: HashMap<String, NgDirectiveInfo>,
    pub pipe_selector_map: HashMap<String, NgPipeInfo>,
    #[serde(skip)]
    pub path_map: HashMap<String, NgElement>,
}

impl AnalysisCollector for NgAnalysisResults {
    fn extend(&mut self, other: Self) {
        self.components.extend(other.components);
        self.directives.extend(other.directives);
        self.modules.extend(other.modules);
        self.others.extend(other.others);
        self.pipes.extend(other.pipes);
        self.services.extend(other.services);
        self.test_specs.extend(other.test_specs);
    }

    fn process_file(
        file_path: &PathBuf,
        resolver: &mut ImportResolver,
        context: &AnalysisContext,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        ng::visitors::analyze_file(
            file_path,
            context.base_path.clone(),
            &context.source_map,
            context.project_name.clone().parse()?,
            context.project_ts_config.clone(),
            resolver,
            &context.file_reader,
        )
    }
}
