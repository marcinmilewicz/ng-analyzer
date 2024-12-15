use crate::ng::analysis::dependency::maps::{NgPathMap, NgSelectorMaps};
use crate::ng::models::ng_references::NgReferences;
use crate::ng::models::NgAnalysisResults;
use std::collections::HashMap;

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

    pub fn analyze_dependencies(&self, results: &mut NgAnalysisResults) {
        //let mut usage_stats = self.collect_usage_statistics(components);
        let mut usage_stats: HashMap<String, NgReferences> = HashMap::new();

        for component in &results.components {
            for used_selector in &component.template_usages.components {
                if let Some(used_component) =
                    self.selector_maps.component_selector_map.get(used_selector)
                {
                    usage_stats
                        .entry(used_component.base.name.clone())
                        .or_insert_with(NgReferences::default)
                        .used_in_templates
                        .push(component.base.relative_path.clone());
                }
            }

            for used_selector in &component.template_usages.directives {
                if let Some(used_directive) =
                    self.selector_maps.directive_selector_map.get(used_selector)
                {
                    usage_stats
                        .entry(used_directive.base.name.clone())
                        .or_insert_with(NgReferences::default)
                        .used_in_templates
                        .push(component.base.relative_path.clone());
                }
            }

            for used_name in &component.template_usages.pipes {
                if let Some(used_pipe) = self.selector_maps.pipe_selector_map.get(used_name) {
                    usage_stats
                        .entry(used_pipe.base.name.clone())
                        .or_insert_with(NgReferences::default)
                        .used_in_templates
                        .push(component.base.relative_path.clone());
                }
            }
        }

        for component in &mut results.components {
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
