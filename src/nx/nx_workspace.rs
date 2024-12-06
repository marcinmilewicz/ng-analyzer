use crate::analysis::models::ts_config::TSConfig;
use crate::nx::config::NxProjectConfig;
use crate::nx::nx_project::NxProject;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug)]
pub struct NxWorkspace {
    projects: HashMap<PathBuf, NxProject>,
    workspace_root: PathBuf,
}

impl NxWorkspace {
    pub fn new(workspace_root: PathBuf) -> Self {
        Self {
            projects: HashMap::new(),
            workspace_root,
        }
    }

    pub fn get_projects(&self) -> &HashMap<PathBuf, NxProject> {
        &self.projects
    }

    pub fn load_configuration(&mut self) -> std::io::Result<()> {
        self.load_projects()?;
        Ok(())
    }

    fn collect_project_files(&self, project_root: &Path) -> HashSet<PathBuf> {
        WalkDir::new(project_root)
            .into_iter()
            .filter_entry(|e| !is_node_modules(e))
            .filter_map(|entry| {
                entry.ok().and_then(|e| {
                    if e.file_type().is_file() {
                        Some(e.path().to_path_buf())
                    } else {
                        None
                    }
                })
            })
            .collect()
    }

    fn load_projects(&mut self) -> std::io::Result<()> {
        for entry in WalkDir::new(&self.workspace_root)
            .into_iter()
            .filter_entry(|e| !is_node_modules(e))
        {
            let entry = entry?;
            if entry.file_name() == "project.json" {
                match self.parse_project_config(entry.path()) {
                    Ok(project_config) => {
                        let project_root = entry.path().parent().unwrap().to_path_buf();

                        let sibling = get_sibling(entry, "tsconfig.json");

                        if let Some(ref sibling_path) = sibling {
                            match self.parse_tsconfig(sibling_path.as_path()) {
                                Ok(tsconfig) => {
                                    let resolved_config =
                                        self.resolve_extended_tsconfig(&tsconfig, sibling_path)?;
                                    let files = self.collect_project_files(&project_root);

                                    let project = NxProject::with_files(
                                        project_config.name.clone(),
                                        project_config,
                                        resolved_config,
                                        files,
                                    );

                                    self.projects.insert(project_root, project);
                                }
                                Err(e) => {
                                    eprintln!(
                                        "Error parsing tsconfig at {:?}: {}",
                                        sibling_path, e
                                    );
                                }
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Error parsing project.json at {:?}: {}", entry.path(), e);
                    }
                }
            }
        }
        Ok(())
    }

    fn parse_project_config(&self, path: &Path) -> std::io::Result<NxProjectConfig> {
        let content = fs::read_to_string(path)?;
        let config: NxProjectConfig = serde_json::from_str(&content)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        Ok(config)
    }

    fn parse_tsconfig(&self, path: &Path) -> std::io::Result<TSConfig> {
        let content = fs::read_to_string(path)?;
        let config: TSConfig = serde_json::from_str(&content)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        Ok(config)
    }

    fn resolve_extended_tsconfig(
        &self,
        config: &TSConfig,
        config_path: &Path,
    ) -> std::io::Result<TSConfig> {
        let mut resolved_config = config.clone();

        if let Some(extends_path) = &config.extends {
            let base_path = config_path.parent().unwrap();
            let extended_path = base_path.join(extends_path);

            if extended_path.exists() {
                let extended_config = self.parse_tsconfig(&extended_path)?;

                if let Some(extended_options) = extended_config.compiler_options {
                    if let Some(current_options) = &mut resolved_config.compiler_options {
                        if let Some(extended_paths) = extended_options.paths {
                            if let Some(current_paths) = &mut current_options.paths {
                                for (key, value) in extended_paths {
                                    if !current_paths.contains_key(&key) {
                                        current_paths.insert(key, value);
                                    }
                                }
                            } else {
                                current_options.paths = Some(extended_paths);
                            }
                        }

                        if current_options.base_url.is_none() {
                            current_options.base_url = extended_options.base_url;
                        }
                    } else {
                        resolved_config.compiler_options = Some(extended_options);
                    }
                }
            }
        }

        Ok(resolved_config)
    }
}

fn is_node_modules(entry: &walkdir::DirEntry) -> bool {
    entry
        .path()
        .components()
        .any(|c| c.as_os_str() == "node_modules")
}

fn get_sibling(entry: walkdir::DirEntry, target: &str) -> Option<PathBuf> {
    let parent = entry.path().parent()?;
    let sibling = parent.join(target);

    if sibling.exists() {
        Some(sibling)
    } else {
        None
    }
}
