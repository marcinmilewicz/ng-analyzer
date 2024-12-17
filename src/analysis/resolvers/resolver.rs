use crate::analysis::models::import::ImportType;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub struct ImportPathResolver {
    base_path: PathBuf,
}

impl ImportPathResolver {
    pub fn new(base_path: PathBuf) -> Self {
        Self { base_path }
    }

    pub fn resolve_import(
        &self,
        import_path: &str,
        current_file: &Path,
        ts_paths: HashMap<String, Vec<String>>,
    ) -> (Option<PathBuf>, ImportType) {
        let import_type = self.determine_import_type(import_path);

        println!("Import type: {:?}", import_type);

        match import_type {
            ImportType::Relative => (
                self.resolve_relative_import(import_path, current_file),
                ImportType::Relative,
            ),
            ImportType::Absolute => (
                self.resolve_absolute_import(import_path),
                ImportType::Absolute,
            ),
            ImportType::Package => (
                self.resolve_package_import(import_path, ts_paths),
                ImportType::Package,
            ),
            ImportType::NodeModule => (
                self.resolve_node_module_import(import_path),
                ImportType::NodeModule,
            ),
        }
    }

    pub fn resolve_relative_import(
        &self,
        import_path: &str,
        current_file: &Path,
    ) -> Option<PathBuf> {
        let parent = current_file.parent()?;
        let resolved = parent.join(import_path);

        if !resolved.exists() {
            for ext in &[".ts", ".tsx", ".d.ts"] {
                let with_ext = PathBuf::from(format!("{}{}", resolved.display(), ext));
                if with_ext.exists() {
                    return Some(with_ext);
                }
            }
        }

        Some(resolved)
    }

    pub fn resolve_absolute_import(&self, import_path: &str) -> Option<PathBuf> {
        let path = self.base_path.join(import_path.trim_start_matches('/'));
        Some(path)
    }

    pub fn resolve_package_import(
        &self,
        import_path: &str,
        ts_paths: HashMap<String, Vec<String>>,
    ) -> Option<PathBuf> {
        for (alias, paths) in &ts_paths {
            if import_path.starts_with(alias) {
                for path in paths {
                    let resolved = self
                        .base_path
                        .join(path.replace("*", &import_path[alias.len()..]));
                    if resolved.exists() {
                        return Some(resolved);
                    }
                }
            }
        }
        // todo resolve_node_module_import shoudl work somehow
        //self.resolve_node_module_import(import_path)
        return None;
    }

    pub fn resolve_node_module_import(&self, import_path: &str) -> Option<PathBuf> {
        let mut current = self.base_path.clone();
        while current.parent().is_some() {
            let node_modules = current.join("node_modules").join(import_path);
            if node_modules.exists() {
                return Some(node_modules);
            }
            current = current.parent()?.to_path_buf();
        }
        None
    }

    fn determine_import_type(&self, import_path: &str) -> ImportType {
        if import_path.starts_with("./") || import_path.starts_with("../") {
            ImportType::Relative
        } else if import_path.starts_with('/') {
            ImportType::Absolute
        } else if import_path.starts_with('@') {
            ImportType::Package
        } else {
            ImportType::NodeModule
        }
    }
}
