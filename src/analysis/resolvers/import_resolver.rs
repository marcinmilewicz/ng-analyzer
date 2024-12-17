use crate::analysis::models::import::{ImportKind, ImportType, ImportedItem, ResolvedImport};
use crate::analysis::resolvers::cache::ImportCache;
use crate::analysis::resolvers::import_graph::ImportGraph;
use crate::analysis::resolvers::parsers::ImportParser;
use crate::analysis::resolvers::resolver::ImportPathResolver;
use crate::analysis::utils::path_utils::{get_relative_path, normalize_path};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use ImportKind::Named;

pub struct ImportResolver {
    base_path: PathBuf,
    cache: ImportCache,
    import_parser: ImportParser,
    import_path_resolver: ImportPathResolver,
    import_graph: Arc<ImportGraph>,
}

impl ImportResolver {
    pub fn new(
        base_path: PathBuf,
        project_path: PathBuf,
        shared_cache: Option<ImportCache>,
        import_graph: Arc<ImportGraph>,
    ) -> Self {
        let import_path_resolver= ImportPathResolver::new(base_path.clone());
        Self {
            base_path,
            cache: shared_cache.unwrap_or_else(ImportCache::new),
            import_parser: ImportParser {},
            import_path_resolver,
            import_graph,
        }
    }

    pub fn resolve_import(
        &mut self,
        import_path: &str,
        name: &str,
        current_file: &Path,
        ts_paths: HashMap<String, Vec<String>>,
    ) -> Option<ResolvedImport> {
        if let Some(path) = self.cache.get(import_path, name) {
            self.import_graph
                .add_dependency(current_file.to_path_buf(), path.resolved_path.clone());

            return Some(path.clone());
        }

        println!("Import path: {}", import_path);

        let (resolved_path, import_type) =
            self.import_path_resolver
                .resolve_import(import_path, current_file, ts_paths);

        println!("Resolve path: {:?}", resolved_path);

        let final_path = self
            .import_parser
            .find_export_declaration(&resolved_path.unwrap_or_default(), name)?;

        println!("Final path: {:?}", final_path);

        let resolved_import =
            self.create_resolved_import(import_path, name, final_path, import_type, self.base_path.clone());

        self.cache.insert(
            String::from(import_path),
            name.to_string(),
            resolved_import.clone(),
        );

        self.import_graph.add_dependency(
            current_file.to_path_buf(),
            resolved_import.resolved_path.clone(),
        );

        Some(resolved_import)


    }

    fn create_resolved_import(
        &self,
        source: &str,
        name: &str,
        path: PathBuf,
        import_type: ImportType,
        base_path: PathBuf,
    ) -> ResolvedImport {
        let resolved_path = normalize_path(&path);
        let relative_path = get_relative_path(resolved_path.as_path(), base_path.as_path());
        ResolvedImport {
            source: source.to_string(),
            resolved_path,
            relative_path,
            import_type,
            imported_item: ImportedItem {
                name: name.to_string(),
                alias: None,
                import_kind: Named,
            },
        }
    }
}
