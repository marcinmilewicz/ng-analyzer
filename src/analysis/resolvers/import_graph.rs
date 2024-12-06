use dashmap::DashMap;
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;

pub struct ImportGraph {
    dependencies: Arc<DashMap<PathBuf, HashSet<PathBuf>>>,
    reverse_dependencies: Arc<DashMap<PathBuf, HashSet<PathBuf>>>,
}

impl ImportGraph {
    pub fn new() -> Self {
        Self {
            dependencies: Arc::new(DashMap::new()),
            reverse_dependencies: Arc::new(DashMap::new()),
        }
    }

    pub fn add_dependency(&self, source: PathBuf, target: PathBuf) {
        if let Some(mut set) = self.dependencies.get_mut(&source) {
            set.insert(target.clone());
        } else {
            let mut set = HashSet::new();
            set.insert(target.clone());
            self.dependencies.insert(source.clone(), set);
        }

        if let Some(mut set) = self.reverse_dependencies.get_mut(&target) {
            set.insert(source);
        } else {
            let mut set = HashSet::new();
            set.insert(source);
            self.reverse_dependencies.insert(target, set);
        }
    }

    pub fn get_dependencies(&self, file: &PathBuf) -> Option<HashSet<PathBuf>> {
        self.dependencies.get(file).map(|deps| deps.clone())
    }

    pub fn get_dependents(&self, file: &PathBuf) -> Option<HashSet<PathBuf>> {
        self.reverse_dependencies.get(file).map(|deps| deps.clone())
    }

    pub fn get_all_dependencies(&self, file: &PathBuf) -> HashSet<PathBuf> {
        let mut all_deps = HashSet::new();
        let mut to_process = vec![file.clone()];

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

    pub fn analyze_circular_dependencies(&self) -> Vec<Vec<PathBuf>> {
        let mut circular_deps = Vec::new();
        let mut visited = HashSet::new();
        let mut path = Vec::new();

        for entry in self.dependencies.iter() {
            let file = entry.key();
            if !visited.contains(file) {
                self.find_cycles(file, &mut visited, &mut path, &mut circular_deps);
            }
        }

        circular_deps
    }

    fn find_cycles(
        &self,
        current: &PathBuf,
        visited: &mut HashSet<PathBuf>,
        path: &mut Vec<PathBuf>,
        cycles: &mut Vec<Vec<PathBuf>>,
    ) {
        if path.contains(current) {
            let cycle_start = path.iter().position(|x| x == current).unwrap();
            cycles.push(path[cycle_start..].to_vec());
            return;
        }

        if !visited.insert(current.clone()) {
            return;
        }

        path.push(current.clone());

        if let Some(deps) = self.get_dependencies(current) {
            for dep in deps {
                self.find_cycles(&dep, visited, path, cycles);
            }
        }

        path.pop();
    }
}
