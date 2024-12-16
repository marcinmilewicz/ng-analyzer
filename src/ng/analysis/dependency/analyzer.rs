use crate::ng::analysis::dependency::maps::{NgPathMap, NgSelectorMaps};
use crate::ng::models::ng_references::NgReferences;
use crate::ng::models::{NgAnalysisResults, NgComponentInfo};
use std::collections::HashMap;
use crate::ng::models::ng_base::NgBaseInfo;

pub struct NgDependencyAnalyzer {
    selector_maps: NgSelectorMaps,
    ng_path_map: NgPathMap,
}

impl NgDependencyAnalyzer {
    pub fn new(selector_maps: NgSelectorMaps, ng_path_map: NgPathMap) -> Self {
        Self {
            selector_maps,
            ng_path_map,
        }
    }

    fn get_base_key(base: &NgBaseInfo) -> String {
        format!(
            "{}:{}",
            base.name,
            base.relative_path
        )
    }

    pub fn analyze_dependencies(&mut self, results: &mut NgAnalysisResults) {
        let mut usage_stats: HashMap<String, NgReferences> = HashMap::new();

        for component in &results.components {
            for component_usage in &component.template_usages.components {
                if let Some(used_component) = self.selector_maps.component_selector_map.get(component_usage) {
                    let used_component_key = Self::get_base_key(&used_component.base);
                    usage_stats
                        .entry(used_component_key)
                        .or_insert_with(NgReferences::new)
                        .used_in_templates
                        .push(Self::get_base_key(&component.base));
                }
            }

            for directive_usage in &component.template_usages.directives {
                if let Some(used_directive) = self.selector_maps.directive_selector_map.get(directive_usage) {
                    let used_directive_key = Self::get_base_key(&used_directive.base);
                    usage_stats
                        .entry(used_directive_key)
                        .or_insert_with(NgReferences::new)
                        .used_in_templates
                        .push(Self::get_base_key(&component.base));
                }
            }

            for pipe_usage in &component.template_usages.pipes {
                if let Some(used_pipe) = self.selector_maps.pipe_selector_map.get(pipe_usage) {
                    let used_pipe_key = Self::get_base_key(&used_pipe.base);
                    usage_stats
                        .entry(used_pipe_key)
                        .or_insert_with(NgReferences::new)
                        .used_in_templates
                        .push(Self::get_base_key(&component.base));
                }
            }
        }

        for component in &mut results.components {
            let component_key = Self::get_base_key(&component.base);
            if let Some(references) = usage_stats.get(&component_key) {
                component.base.references = references.clone();
            }
        }

        for pipe in &mut results.pipes {
            let pipe_key = Self::get_base_key(&pipe.base);
            if let Some(references) = usage_stats.get(&pipe_key) {
                pipe.base.references = references.clone();
            }
        }

        for directive in &mut results.directives {
            let directive_key = Self::get_base_key(&directive.base);
            if let Some(references) = usage_stats.get(&directive_key) {
                directive.base.references = references.clone();
            }
        }
    }
}
