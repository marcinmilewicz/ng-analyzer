use crate::ng::models::NgAnalysisResults;
use serde::Serialize;
use std::collections::BTreeMap;
use std::path::PathBuf;

/// Usage analytics for React components — how often each component is
/// rendered and which props are actually used (design-system adoption data,
/// the react-scanner use case).
#[derive(Debug, Serialize)]
pub struct ReactComponentUsage {
    pub component: String,
    pub file: PathBuf,
    pub package_name: String,
    pub usage_count: usize,
    pub props: Vec<PropUseCount>,
}

#[derive(Debug, Serialize)]
pub struct PropUseCount {
    pub name: String,
    pub count: usize,
}

pub fn analyze_react_usage(results: &NgAnalysisResults) -> Vec<ReactComponentUsage> {
    if results.react_components.is_empty() {
        return Vec::new();
    }

    // (component file, name) -> (usage count, prop -> count)
    let mut stats: BTreeMap<(PathBuf, String), (usize, BTreeMap<String, usize>)> = BTreeMap::new();

    for file in &results.source_files {
        for usage in &file.jsx_usages {
            // Resolve the JSX tag: imported name → declaring file, otherwise
            // a component defined in the same file.
            let target: Option<PathBuf> = file
                .imports
                .iter()
                .find(|import| import.imported_item.name == usage.component)
                .map(|import| import.resolved_path.clone())
                .or_else(|| {
                    results
                        .react_components
                        .iter()
                        .find(|component| {
                            component.name == usage.component && component.source_path == file.path
                        })
                        .map(|component| component.source_path.clone())
                });

            let Some(target) = target else { continue };

            let entry = stats
                .entry((target, usage.component.clone()))
                .or_insert_with(|| (0, BTreeMap::new()));
            entry.0 += 1;
            for prop in &usage.props {
                *entry.1.entry(prop.clone()).or_insert(0) += 1;
            }
        }
    }

    results
        .react_components
        .iter()
        .map(|component| {
            let key = (component.source_path.clone(), component.name.clone());
            let (usage_count, props) = stats.get(&key).cloned().unwrap_or_default();
            ReactComponentUsage {
                component: component.name.clone(),
                file: component.source_path.clone(),
                package_name: component.package_name.clone(),
                usage_count,
                props: props
                    .into_iter()
                    .map(|(name, count)| PropUseCount { name, count })
                    .collect(),
            }
        })
        .collect()
}
