use crate::ng::analysis::ng_results::NgAnalysisResults;
use crate::ng::models::ng_directive::NgDirectiveInfo;
use crate::ng::models::ng_element::NgElement;
use crate::ng::models::ng_pipe::NgPipeInfo;
use crate::ng::models::NgComponentInfo;
use std::collections::HashMap;

#[derive(Default)]
pub struct NgSelectorMaps {
    pub component_selector_map: HashMap<String, NgComponentInfo>,
    pub directive_selector_map: HashMap<String, NgDirectiveInfo>,
    pub pipe_selector_map: HashMap<String, NgPipeInfo>,
}

impl NgSelectorMaps {
    pub fn build_from_results(results: &NgAnalysisResults) -> Self {
        let mut maps = Self::default();
        maps.component_selector_map = results
            .components
            .iter()
            .map(|comp| (comp.selector.clone(), comp.clone()))
            .collect();

        maps.directive_selector_map = results
            .directives
            .iter()
            .map(|dir| (dir.selector.clone(), dir.clone()))
            .collect();

        maps.pipe_selector_map = results
            .pipes
            .iter()
            .map(|pipe| (pipe.name.clone(), pipe.clone()))
            .collect();

        maps
    }
}

pub struct NgPathMap {
    pub path_map: HashMap<String, NgElement>,
}

impl NgPathMap {
    pub fn build_from_results(results: &NgAnalysisResults) -> Self {
        let mut path_map = HashMap::new();

        for component in results.components.iter() {
            path_map.insert(
                component.base.relative_path.clone(),
                NgElement::Component(component.clone()),
            );
        }

        for directive in results.directives.iter() {
            path_map.insert(
                directive.base.relative_path.clone(),
                NgElement::Directive(directive.clone()),
            );
        }

        for pipe in results.pipes.iter() {
            path_map.insert(
                pipe.base.relative_path.clone(),
                NgElement::Pipe(pipe.clone()),
            );
        }

        for module in results.modules.iter() {
            path_map.insert(
                module.base.relative_path.clone(),
                NgElement::Module(module.clone()),
            );
        }

        for service in results.services.iter() {
            path_map.insert(
                service.base.relative_path.clone(),
                NgElement::Service(service.clone()),
            );
        }

        Self { path_map }
    }
}
