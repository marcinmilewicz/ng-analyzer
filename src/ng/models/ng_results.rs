use super::{NgComponentInfo, NgModuleInfo, NgServiceInfo};
use crate::analysis::resolvers::import_resolver::ImportResolver;
use crate::ng;
use std::collections::HashMap;

use crate::analysis::processor::collector::AnalysisCollector;
use crate::analysis::processor::context::AnalysisContext;
use crate::analysis::utils::path_utils::get_relative_path;
use crate::ng::models::component_usage::NgReferences;
use crate::ng::models::ng_directive::NgDirectiveInfo;
use crate::ng::models::ng_element::NgElement;
use crate::ng::models::ng_pipe::NgPipeInfo;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct NgAnalysisResults {
    base_path: PathBuf,

    pub components: Vec<NgComponentInfo>,
    pub directives: Vec<NgDirectiveInfo>,
    pub modules: Vec<NgModuleInfo>,
    pub pipes: Vec<NgPipeInfo>,
    pub services: Vec<NgServiceInfo>,

    pub component_selector_map: HashMap<String, NgComponentInfo>,
    pub directive_selector_map: HashMap<String, NgDirectiveInfo>,
    pub pipe_selector_map: HashMap<String, NgPipeInfo>,
    #[serde(skip)]
    pub path_map: HashMap<String, NgElement>,
}

impl NgAnalysisResults {
    pub fn new(base_path: PathBuf) -> Self {
        Self {
            base_path,
            components: Vec::new(),
            directives: Vec::new(),
            modules: Vec::new(),
            pipes: Vec::new(),
            services: Vec::new(),
            component_selector_map: HashMap::new(),
            directive_selector_map: HashMap::new(),
            pipe_selector_map: HashMap::new(),
            path_map: HashMap::new(),
        }
    }

    pub fn build_maps(&mut self) {
        self.component_selector_map = self
            .components
            .iter()
            .map(|comp| (comp.selector.clone(), comp.clone()))
            .collect();

        self.directive_selector_map = self
            .directives
            .iter()
            .map(|dir| (dir.selector.clone(), dir.clone()))
            .collect();

        self.pipe_selector_map = self
            .pipes
            .iter()
            .map(|pipe| (pipe.name.clone(), pipe.clone()))
            .collect();

        self.path_map = HashMap::new();

        for component in self.components.iter() {
            self.path_map.insert(
                component.base.relative_path.clone(),
                NgElement::Component(component.clone()),
            );
        }

        for directive in self.directives.iter() {
            self.path_map.insert(
                directive.base.relative_path.clone(),
                NgElement::Directive(directive.clone()),
            );
        }

        for pipe in self.pipes.iter() {
            self.path_map.insert(
                pipe.base.relative_path.clone(),
                NgElement::Pipe(pipe.clone()),
            );
        }

        for module in self.modules.iter() {
            self.path_map.insert(
                get_relative_path(&module.base.source_path.clone(), &self.base_path.as_path()),
                NgElement::Module(module.clone()),
            );
        }

        for service in self.services.iter() {
            self.path_map.insert(
                get_relative_path(&service.base.source_path.clone(), &self.base_path.as_path()),
                NgElement::Service(service.clone()),
            );
        }
    }

    pub fn analyze_dependencies(&mut self) {
        let mut usage_stats: HashMap<String, NgReferences> = HashMap::new();

        for component in &self.components {
            for used_selector in &component.template_usages.components {
                if let Some(used_component) = self.component_selector_map.get(used_selector) {
                    usage_stats
                        .entry(used_component.base.name.clone())
                        .or_insert_with(NgReferences::default)
                        .used_in_templates
                        .push(component.base.relative_path.clone());
                }
            }

            for used_selector in &component.template_usages.directives {
                if let Some(used_directive) = self.directive_selector_map.get(used_selector) {
                    usage_stats
                        .entry(used_directive.base.name.clone())
                        .or_insert_with(NgReferences::default)
                        .used_in_templates
                        .push(component.base.relative_path.clone());
                }
            }

            for used_name in &component.template_usages.pipes {
                if let Some(used_pipe) = self.pipe_selector_map.get(used_name) {
                    usage_stats
                        .entry(used_pipe.base.name.clone())
                        .or_insert_with(NgReferences::default)
                        .used_in_templates
                        .push(component.base.relative_path.clone());
                }
            }
        }

        for component in &mut self.components {
            if let Some(usage) = usage_stats.get(&component.base.name) {
                component.references = NgReferences {
                    used_in_templates: usage.used_in_templates.clone(),
                    used_in_class: vec![],
                };
            } else {
                component.references = NgReferences::default();
            }
        }
    }
}

impl AnalysisCollector for NgAnalysisResults {
    fn extend(&mut self, other: Self) {
        self.components.extend(other.components);
        self.services.extend(other.services);
        self.modules.extend(other.modules);
        self.directives.extend(other.directives);
        self.pipes.extend(other.pipes);

        self.component_selector_map
            .extend(other.component_selector_map);
        self.directive_selector_map
            .extend(other.directive_selector_map);
        self.pipe_selector_map.extend(other.pipe_selector_map);
        self.path_map.extend(other.path_map);
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
