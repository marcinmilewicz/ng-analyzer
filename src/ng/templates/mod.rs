pub mod scanner;
pub mod selector;

use crate::analysis::resolvers::import_graph::ImportGraph;
use crate::ng::models::NgAnalysisResults;
use selector::SimpleSelector;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// A usage of an Angular entity inside a component template.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TemplateUsageInfo {
    pub component: String,
    pub component_path: PathBuf,
    pub target: String,
    pub target_path: PathBuf,
    pub target_kind: TemplateTargetKind,
    pub via: TemplateUsageVia,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum TemplateTargetKind {
    Component,
    Directive,
    Pipe,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum TemplateUsageVia {
    Selector,
    Pipe,
}

struct SelectorEntry {
    name: String,
    path: PathBuf,
    kind: TemplateTargetKind,
    selectors: Vec<SimpleSelector>,
}

/// Matches every component template (external or inline) against the
/// workspace-wide registry of selectors and pipe names. Template usages are
/// also recorded as dependency edges — a component used only in HTML is not
/// dead code.
pub fn analyze_templates(
    results: &NgAnalysisResults,
    import_graph: &ImportGraph,
) -> Vec<TemplateUsageInfo> {
    let mut registry: Vec<SelectorEntry> = Vec::new();

    for component in &results.components {
        if component.selector.is_empty() {
            continue;
        }
        registry.push(SelectorEntry {
            name: component.base.name.clone(),
            path: component.base.source_path.clone(),
            kind: TemplateTargetKind::Component,
            selectors: selector::parse_selector(&component.selector),
        });
    }
    for directive in &results.directives {
        if directive.selector.is_empty() {
            continue;
        }
        registry.push(SelectorEntry {
            name: directive.base.name.clone(),
            path: directive.base.source_path.clone(),
            kind: TemplateTargetKind::Directive,
            selectors: selector::parse_selector(&directive.selector),
        });
    }

    let pipes: Vec<(&str, &PathBuf, &str)> = results
        .pipes
        .iter()
        .filter(|pipe| !pipe.name.is_empty())
        .map(|pipe| {
            (
                pipe.name.as_str(),
                &pipe.base.source_path,
                pipe.base.name.as_str(),
            )
        })
        .collect();

    let mut usages = Vec::new();

    for component in &results.components {
        let template = match &component.template_inline {
            Some(inline) => Some(inline.clone()),
            None if !component.template_path.is_empty() => {
                let template_file = component
                    .base
                    .source_path
                    .parent()
                    .map(|dir| dir.join(&component.template_path));
                match template_file {
                    Some(path) if path.exists() => std::fs::read_to_string(&path).ok(),
                    Some(path) => {
                        eprintln!(
                            "⚠️ Template {:?} of {} not found",
                            path, component.base.name
                        );
                        None
                    }
                    None => None,
                }
            }
            None => None,
        };

        let Some(template) = template else {
            continue;
        };

        let scan = scanner::scan_template(&template);

        for entry in &registry {
            let matched = scan
                .elements
                .iter()
                .any(|element| selector::matches(&entry.selectors, element));
            if matched {
                import_graph.add_dependency(component.base.source_path.clone(), entry.path.clone());
                usages.push(TemplateUsageInfo {
                    component: component.base.name.clone(),
                    component_path: component.base.source_path.clone(),
                    target: entry.name.clone(),
                    target_path: entry.path.clone(),
                    target_kind: entry.kind.clone(),
                    via: TemplateUsageVia::Selector,
                });
            }
        }

        for (pipe_name, pipe_path, pipe_class) in &pipes {
            if scan.pipes.contains(*pipe_name) {
                import_graph
                    .add_dependency(component.base.source_path.clone(), (*pipe_path).clone());
                usages.push(TemplateUsageInfo {
                    component: component.base.name.clone(),
                    component_path: component.base.source_path.clone(),
                    target: pipe_class.to_string(),
                    target_path: (*pipe_path).clone(),
                    target_kind: TemplateTargetKind::Pipe,
                    via: TemplateUsageVia::Pipe,
                });
            }
        }
    }

    usages.sort_by(|a, b| {
        (&a.component_path, &a.target_path, &a.target).cmp(&(
            &b.component_path,
            &b.target_path,
            &b.target,
        ))
    });
    usages
}
