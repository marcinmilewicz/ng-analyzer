use crate::analyses::project_map::ProjectCatalog;
use crate::analysis::models::import::ImportKind;
use crate::ng::models::NgAnalysisResults;
use crate::ng::templates::TemplateUsageInfo;
use petgraph::algo::tarjan_scc;
use petgraph::graph::{DiGraph, NodeIndex};
use serde::Serialize;
use std::collections::{BTreeMap, HashMap, HashSet};

#[derive(Debug, Serialize)]
pub struct StatsReport {
    pub projects: Vec<ProjectStatsInfo>,
    pub dependencies: Vec<ProjectDependencyInfo>,
    pub project_cycles: Vec<Vec<String>>,
}

#[derive(Debug, Serialize)]
pub struct ProjectStatsInfo {
    pub name: String,
    pub project_type: String,
    pub tags: Vec<String>,
    pub files: usize,
    pub exports: usize,
    /// Projects that depend on this one (afferent coupling, Ca).
    pub afferent: usize,
    /// Projects this one depends on (efferent coupling, Ce).
    pub efferent: usize,
    /// I = Ce / (Ca + Ce); 0 for isolated projects.
    pub instability: f64,
}

#[derive(Debug, Serialize)]
pub struct ProjectDependencyInfo {
    pub from: String,
    pub to: String,
    /// Total references (imports + template usages + lazy loads).
    pub count: usize,
    /// True when any of the references is a dynamic `import()`.
    pub lazy: bool,
    pub symbols: Vec<SymbolUseCount>,
}

#[derive(Debug, Serialize)]
pub struct SymbolUseCount {
    pub name: String,
    pub count: usize,
}

pub fn build_stats(
    results: &NgAnalysisResults,
    template_usages: &[TemplateUsageInfo],
    catalog: &ProjectCatalog,
) -> StatsReport {
    // (from_project, to_project) -> (symbol -> count, lazy)
    let mut edges: BTreeMap<(String, String), (BTreeMap<String, usize>, bool)> = BTreeMap::new();

    let mut record = |from: &str, to: &str, symbol: String, lazy: bool| {
        let entry = edges
            .entry((from.to_string(), to.to_string()))
            .or_insert_with(|| (BTreeMap::new(), false));
        *entry.0.entry(symbol).or_insert(0) += 1;
        entry.1 |= lazy;
    };

    for file in &results.source_files {
        let Some(from) = catalog.project_of(&file.path) else {
            continue;
        };
        for import in &file.imports {
            if let Some(to) = catalog.project_of(&import.resolved_path) {
                if to.name != from.name {
                    // `import './x'` binds no name: the dependency is on the
                    // whole module, spelled "*" as for dynamic imports.
                    let exported = if import.imported_item.import_kind == ImportKind::SideEffect {
                        "*".to_string()
                    } else {
                        import
                            .imported_item
                            .alias
                            .clone()
                            .unwrap_or_else(|| import.imported_item.name.clone())
                    };
                    record(&from.name, &to.name, exported, false);
                }
            }
        }
        for import in &file.dynamic_imports {
            if let Some(to) = catalog.project_of(&import.resolved_path) {
                if to.name != from.name {
                    record(&from.name, &to.name, "*".to_string(), true);
                }
            }
        }
    }

    for usage in template_usages {
        let (Some(from), Some(to)) = (
            catalog.project_of(&usage.component_path),
            catalog.project_of(&usage.target_path),
        ) else {
            continue;
        };
        if from.name != to.name {
            record(&from.name, &to.name, usage.target.clone(), false);
        }
    }

    let dependencies: Vec<ProjectDependencyInfo> = edges
        .iter()
        .map(|((from, to), (symbols, lazy))| ProjectDependencyInfo {
            from: from.clone(),
            to: to.clone(),
            count: symbols.values().sum(),
            lazy: *lazy,
            symbols: symbols
                .iter()
                .map(|(name, count)| SymbolUseCount {
                    name: name.clone(),
                    count: *count,
                })
                .collect(),
        })
        .collect();

    // Per-project aggregates.
    let mut afferent: HashMap<&str, HashSet<&str>> = HashMap::new();
    let mut efferent: HashMap<&str, HashSet<&str>> = HashMap::new();
    for dep in &dependencies {
        efferent.entry(&dep.from).or_default().insert(&dep.to);
        afferent.entry(&dep.to).or_default().insert(&dep.from);
    }

    let mut files_per_project: HashMap<&str, usize> = HashMap::new();
    let mut exports_per_project: HashMap<&str, usize> = HashMap::new();
    for file in &results.source_files {
        if let Some(project) = catalog.project_of(&file.path) {
            *files_per_project.entry(project.name.as_str()).or_insert(0) += 1;
            *exports_per_project
                .entry(project.name.as_str())
                .or_insert(0) += file.exports.len();
        }
    }

    let mut projects: Vec<ProjectStatsInfo> = catalog
        .projects()
        .map(|project| {
            let ca = afferent.get(project.name.as_str()).map_or(0, |s| s.len());
            let ce = efferent.get(project.name.as_str()).map_or(0, |s| s.len());
            let instability = if ca + ce == 0 {
                0.0
            } else {
                ce as f64 / (ca + ce) as f64
            };
            ProjectStatsInfo {
                name: project.name.clone(),
                project_type: project.project_type.clone(),
                tags: project.tags.clone(),
                files: files_per_project
                    .get(project.name.as_str())
                    .copied()
                    .unwrap_or(0),
                exports: exports_per_project
                    .get(project.name.as_str())
                    .copied()
                    .unwrap_or(0),
                afferent: ca,
                efferent: ce,
                instability,
            }
        })
        .collect();
    projects.sort_by(|a, b| a.name.cmp(&b.name));

    StatsReport {
        project_cycles: project_cycles(&dependencies),
        projects,
        dependencies,
    }
}

fn project_cycles(dependencies: &[ProjectDependencyInfo]) -> Vec<Vec<String>> {
    let mut graph: DiGraph<String, ()> = DiGraph::new();
    let mut nodes: HashMap<&str, NodeIndex> = HashMap::new();

    for dep in dependencies {
        let from = *nodes
            .entry(&dep.from)
            .or_insert_with(|| graph.add_node(dep.from.clone()));
        let to = *nodes
            .entry(&dep.to)
            .or_insert_with(|| graph.add_node(dep.to.clone()));
        graph.add_edge(from, to, ());
    }

    let mut cycles: Vec<Vec<String>> = tarjan_scc(&graph)
        .into_iter()
        .filter(|scc| {
            scc.len() > 1
                || scc
                    .first()
                    .map(|&n| graph.contains_edge(n, n))
                    .unwrap_or(false)
        })
        .map(|scc| {
            let mut names: Vec<String> = scc.into_iter().map(|n| graph[n].clone()).collect();
            names.sort();
            names
        })
        .collect();
    cycles.sort();
    cycles
}
