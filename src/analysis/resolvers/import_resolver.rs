use crate::analysis::models::import::{ImportKind, ImportType, ImportedItem, ResolvedImport};
use crate::analysis::models::ts_config::TSConfig;
use crate::analysis::resolvers::cache::ImportCache;
use crate::analysis::resolvers::import_graph::ImportGraph;
use crate::analysis::resolvers::parsers::{ImportParser, ModuleCache};
use crate::analysis::resolvers::resolver::ImportPathResolver;
use crate::analysis::utils::path_utils::normalize_path;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub struct ImportResolver {
    cache: ImportCache,
    import_parser: ImportParser,
    import_path_resolver: ImportPathResolver,
    import_graph: Arc<ImportGraph>,
}

impl ImportResolver {
    pub fn new(
        workspace_root: &Path,
        shared_cache: Option<ImportCache>,
        module_cache: ModuleCache,
        import_graph: Arc<ImportGraph>,
    ) -> Self {
        Self {
            cache: shared_cache.unwrap_or_else(ImportCache::new),
            import_parser: ImportParser::new(module_cache),
            import_path_resolver: ImportPathResolver::new(workspace_root.to_path_buf()),
            import_graph,
        }
    }

    pub fn resolve_import(
        &mut self,
        import_path: &str,
        name: &str,
        current_file: &Path,
        ts_config: &TSConfig,
    ) -> Option<ResolvedImport> {
        if let Some(cached) = self.cache.get(import_path, current_file, name) {
            self.import_graph
                .add_dependency(current_file.to_path_buf(), cached.resolved_path.clone());
            return Some(cached);
        }

        let empty_paths = HashMap::new();
        let (ts_paths, base_url) = match &ts_config.compiler_options {
            Some(opts) => (
                opts.paths.as_ref().unwrap_or(&empty_paths),
                opts.base_url.as_deref(),
            ),
            None => (&empty_paths, None),
        };

        let (resolved_path, import_type) =
            self.import_path_resolver
                .resolve_import(import_path, current_file, ts_paths, base_url);

        let resolved_path = resolved_path?;

        // Follow barrel files (index.ts re-exports) to the declaring file.
        // Re-exports through aliases/packages resolve like imports do.
        let path_resolver = &self.import_path_resolver;
        let resolve_specifier = |specifier: &str, from: &Path| {
            path_resolver
                .resolve_import(specifier, from, ts_paths, base_url)
                .0
        };
        let final_path = self
            .import_parser
            .find_export_declaration(&resolved_path, name, &resolve_specifier)
            .unwrap_or(resolved_path);

        let resolved_import =
            self.create_resolved_import(import_path, name, final_path, import_type);

        self.cache.insert(
            import_path,
            current_file,
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
    ) -> ResolvedImport {
        ResolvedImport {
            source: source.to_string(),
            resolved_path: normalize_path(path),
            import_type,
            imported_item: ImportedItem {
                name: name.to_string(),
                alias: None,
                import_kind: ImportKind::Named,
            },
        }
    }
}
