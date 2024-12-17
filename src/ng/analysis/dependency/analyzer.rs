use crate::ng::analysis::dependency::maps::{NgPathMap, NgSelectorMaps};
use crate::ng::analysis::ng_results::NgAnalysisResults;
use crate::ng::models::ng_base::NgBaseInfo;
use crate::ng::models::ng_element::NgElement;
use crate::ng::models::ng_references::NgReferences;
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

    fn get_base_key(base: &NgBaseInfo) -> String {
        format!("{}:{}", base.name, base.relative_path)
    }

    pub fn analyze_dependencies(&mut self, results: &mut NgAnalysisResults) {
        let mut usage_stats: HashMap<String, NgReferences> = HashMap::new();

        self.find_dependencies_based_on_template_usages(results, &mut usage_stats);
        self.find_dependencies_based_on_imports(results, &mut usage_stats);
        self.assign_references(results, &usage_stats);
    }

    fn process_imports(
        &self,
        element: &NgElement,
        usage_stats: &mut HashMap<String, NgReferences>,
    ) {
        let element_key = Self::get_base_key(element.get_base());

        for import in &element.get_base().imports {
            if let Some(used_element) = self.ng_path_map.path_map.get(&import.relative_path) {
                let used_element_key = Self::get_base_key(&used_element.get_base());

                if element_key != used_element_key {
                    usage_stats
                        .entry(used_element_key)
                        .or_insert_with(NgReferences::new)
                        .used_by_imports
                        .push(element_key.clone());
                }
            }
        }
    }

    fn find_dependencies_based_on_imports(
        &self,
        results: &NgAnalysisResults,
        usage_stats: &mut HashMap<String, NgReferences>,
    ) {

        for component in &results.components {
            let element = NgElement::Component(component.clone());
            self.process_imports(&element, usage_stats);
        }

        for directive in &results.directives {
            let element = NgElement::Directive(directive.clone());
            self.process_imports(&element, usage_stats);
        }


        for pipe in &results.pipes {
            let element = NgElement::Pipe(pipe.clone());
            self.process_imports(&element, usage_stats);
        }

        for module in &results.modules {
            let element = NgElement::Module(module.clone());
            self.process_imports(&element, usage_stats);
        }


        for service in &results.services {
            let element = NgElement::Service(service.clone());
            self.process_imports(&element, usage_stats);
        }

        for other in &results.others {
            let element = NgElement::Other(other.clone());
            self.process_imports(&element, usage_stats);
        }


        for test_spec in &results.test_specs {
            let element = NgElement::TestSpec(test_spec.clone());
            self.process_imports(&element, usage_stats);
        }
    }

    fn find_dependencies_based_on_template_usages(
        &self,
        results: &NgAnalysisResults,
        usage_stats: &mut HashMap<String, NgReferences>,
    ) {
        for component in &results.components {
            let component_key = Self::get_base_key(&component.base);

            for component_usage in &component.template_usages.components {
                if let Some(used_component) = self
                    .selector_maps
                    .component_selector_map
                    .get(component_usage)
                {
                    let used_component_key = Self::get_base_key(&used_component.base);
                    if component_key != used_component_key {
                        usage_stats
                            .entry(used_component_key)
                            .or_insert_with(NgReferences::new)
                            .used_by_template
                            .push(component_key.clone());
                    }
                }
            }

            for directive_usage in &component.template_usages.directives {
                if let Some(used_directive) = self
                    .selector_maps
                    .directive_selector_map
                    .get(directive_usage)
                {
                    let used_directive_key = Self::get_base_key(&used_directive.base);
                    usage_stats
                        .entry(used_directive_key)
                        .or_insert_with(NgReferences::new)
                        .used_by_template
                        .push(component_key.clone());
                }
            }

            for pipe_usage in &component.template_usages.pipes {
                if let Some(used_pipe) = self.selector_maps.pipe_selector_map.get(pipe_usage) {
                    let used_pipe_key = Self::get_base_key(&used_pipe.base);
                    usage_stats
                        .entry(used_pipe_key)
                        .or_insert_with(NgReferences::new)
                        .used_by_template
                        .push(component_key.clone());
                }
            }
        }
    }

    fn assign_references(
        &self,
        results: &mut NgAnalysisResults,
        usage_stats: &HashMap<String, NgReferences>,
    ) {
        // Assign references to all Angular elements
        for component in &mut results.components {
            let key = Self::get_base_key(&component.base);
            if let Some(references) = usage_stats.get(&key) {
                component.base.references = references.clone();
            }
        }

        for directive in &mut results.directives {
            let key = Self::get_base_key(&directive.base);
            if let Some(references) = usage_stats.get(&key) {
                directive.base.references = references.clone();
            }
        }

        for pipe in &mut results.pipes {
            let key = Self::get_base_key(&pipe.base);
            if let Some(references) = usage_stats.get(&key) {
                pipe.base.references = references.clone();
            }
        }

        for module in &mut results.modules {
            let key = Self::get_base_key(&module.base);
            if let Some(references) = usage_stats.get(&key) {
                module.base.references = references.clone();
            }
        }

        for service in &mut results.services {
            let key = Self::get_base_key(&service.base);
            if let Some(references) = usage_stats.get(&key) {
                service.base.references = references.clone();
            }
        }

        for other in &mut results.others {
            let key = Self::get_base_key(&other.base);
            if let Some(references) = usage_stats.get(&key) {
                other.base.references = references.clone();
            }
        }

        for test_spec in &mut results.test_specs {
            let key = Self::get_base_key(&test_spec.base);
            if let Some(references) = usage_stats.get(&key) {
                test_spec.base.references = references.clone();
            }
        }
    }
}