use dashmap::DashMap;
use petgraph::algo::tarjan_scc;
use petgraph::graph::{DiGraph, NodeIndex};
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub struct ImportGraph {
    dependencies: Arc<DashMap<PathBuf, HashSet<PathBuf>>>,
    reverse_dependencies: Arc<DashMap<PathBuf, HashSet<PathBuf>>>,
}

/// Serializable snapshot of the graph, sorted for deterministic output.
#[derive(Debug, Serialize)]
pub struct ImportGraphSnapshot {
    pub edges: Vec<ImportGraphEdge>,
    pub circular_dependencies: Vec<Vec<PathBuf>>,
}

#[derive(Debug, Serialize)]
pub struct ImportGraphEdge {
    pub from: PathBuf,
    pub to: Vec<PathBuf>,
}

impl ImportGraph {
    pub fn new() -> Self {
        Self {
            dependencies: Arc::new(DashMap::new()),
            reverse_dependencies: Arc::new(DashMap::new()),
        }
    }

    pub fn add_dependency(&self, source: PathBuf, target: PathBuf) {
        self.dependencies
            .entry(source.clone())
            .or_default()
            .insert(target.clone());

        self.reverse_dependencies
            .entry(target)
            .or_default()
            .insert(source);
    }

    #[allow(dead_code)] // query API for upcoming analyses (unused/stats)
    pub fn get_dependencies(&self, file: &Path) -> Option<HashSet<PathBuf>> {
        self.dependencies.get(file).map(|deps| deps.clone())
    }

    #[allow(dead_code)] // query API for upcoming analyses (unused/stats)
    pub fn get_dependents(&self, file: &Path) -> Option<HashSet<PathBuf>> {
        self.reverse_dependencies.get(file).map(|deps| deps.clone())
    }

    #[allow(dead_code)] // query API for upcoming analyses (unused/stats)
    pub fn get_all_dependencies(&self, file: &Path) -> HashSet<PathBuf> {
        let mut all_deps = HashSet::new();
        let mut to_process = vec![file.to_path_buf()];

        while let Some(current) = to_process.pop() {
            if let Some(deps) = self.get_dependencies(&current) {
                for dep in deps {
                    if all_deps.insert(dep.clone()) {
                        to_process.push(dep);
                    }
                }
            }
        }

        all_deps
    }

    /// Finds circular dependencies as strongly connected components
    /// (Tarjan, iterative — no recursion, no missed cycles).
    /// Every returned group has at least 2 files, or is a self-loop.
    pub fn analyze_circular_dependencies(&self) -> Vec<Vec<PathBuf>> {
        let mut graph: DiGraph<PathBuf, ()> = DiGraph::new();
        let mut node_indices: HashMap<PathBuf, NodeIndex> = HashMap::new();

        let mut node_of = |graph: &mut DiGraph<PathBuf, ()>, path: &PathBuf| -> NodeIndex {
            if let Some(&idx) = node_indices.get(path) {
                return idx;
            }
            let idx = graph.add_node(path.clone());
            node_indices.insert(path.clone(), idx);
            idx
        };

        for entry in self.dependencies.iter() {
            let from = node_of(&mut graph, entry.key());
            for target in entry.value() {
                let to = node_of(&mut graph, target);
                graph.add_edge(from, to, ());
            }
        }

        let mut cycles: Vec<Vec<PathBuf>> = tarjan_scc(&graph)
            .into_iter()
            .filter(|scc| {
                scc.len() > 1
                    || scc
                        .first()
                        .map(|&n| graph.contains_edge(n, n))
                        .unwrap_or(false)
            })
            .map(|scc| {
                let mut files: Vec<PathBuf> = scc.into_iter().map(|n| graph[n].clone()).collect();
                files.sort();
                files
            })
            .collect();

        cycles.sort();
        cycles
    }

    /// Deterministic, serializable view of the whole graph.
    pub fn snapshot(&self) -> ImportGraphSnapshot {
        let mut edges: Vec<ImportGraphEdge> = self
            .dependencies
            .iter()
            .map(|entry| {
                let mut targets: Vec<PathBuf> = entry.value().iter().cloned().collect();
                targets.sort();
                ImportGraphEdge {
                    from: entry.key().clone(),
                    to: targets,
                }
            })
            .collect();
        edges.sort_by(|a, b| a.from.cmp(&b.from));

        ImportGraphSnapshot {
            edges,
            circular_dependencies: self.analyze_circular_dependencies(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(s: &str) -> PathBuf {
        PathBuf::from(s)
    }

    #[test]
    fn detects_simple_cycle() {
        let graph = ImportGraph::new();
        graph.add_dependency(p("a"), p("b"));
        graph.add_dependency(p("b"), p("c"));
        graph.add_dependency(p("c"), p("a"));

        let cycles = graph.analyze_circular_dependencies();
        assert_eq!(cycles.len(), 1);
        assert_eq!(cycles[0], vec![p("a"), p("b"), p("c")]);
    }

    #[test]
    fn detects_cycle_through_shared_visited_node() {
        // Two cycles sharing node "b" — the old single-visited-set DFS
        // missed the second one.
        let graph = ImportGraph::new();
        graph.add_dependency(p("a"), p("b"));
        graph.add_dependency(p("b"), p("a"));
        graph.add_dependency(p("c"), p("b"));
        graph.add_dependency(p("b"), p("c"));

        let cycles = graph.analyze_circular_dependencies();
        // a-b-c form one SCC (a↔b, b↔c)
        assert_eq!(cycles.len(), 1);
        assert_eq!(cycles[0], vec![p("a"), p("b"), p("c")]);
    }

    #[test]
    fn no_cycles_in_dag() {
        let graph = ImportGraph::new();
        graph.add_dependency(p("a"), p("b"));
        graph.add_dependency(p("b"), p("c"));
        graph.add_dependency(p("a"), p("c"));

        assert!(graph.analyze_circular_dependencies().is_empty());
    }

    #[test]
    fn detects_self_loop() {
        let graph = ImportGraph::new();
        graph.add_dependency(p("a"), p("a"));

        let cycles = graph.analyze_circular_dependencies();
        assert_eq!(cycles.len(), 1);
        assert_eq!(cycles[0], vec![p("a")]);
    }
}
