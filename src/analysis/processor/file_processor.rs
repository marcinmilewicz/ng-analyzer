use crate::analysis::models::ts_config::TSConfig;
use crate::analysis::processor::collector::AnalysisCollector;
use crate::analysis::processor::context::AnalysisContext;
use crate::analysis::resolvers::cache::ImportCache;
use crate::analysis::resolvers::import_graph::ImportGraph;
use crate::analysis::resolvers::import_resolver::ImportResolver;
use crate::analysis::resolvers::parsers::ModuleCache;
use crate::file_cache_reader::CachedFileReader;
use rayon::prelude::*;
use std::path::PathBuf;
use std::sync::Arc;
use swc_common::SourceMap;
use walkdir::WalkDir;

type EntryPredicate = Box<dyn Fn(&walkdir::DirEntry) -> bool + Send + Sync>;

/// Caches and graph shared by every project processor in a run.
#[derive(Clone)]
pub struct SharedAnalysisState {
    pub import_cache: ImportCache,
    pub module_cache: ModuleCache,
    pub import_graph: Arc<ImportGraph>,
    pub file_reader: CachedFileReader,
}

impl SharedAnalysisState {
    pub fn new() -> Self {
        Self {
            import_cache: ImportCache::new(),
            module_cache: ModuleCache::new(),
            import_graph: Arc::new(ImportGraph::new()),
            file_reader: CachedFileReader::new(),
        }
    }
}

pub struct ProjectProcessor {
    context: AnalysisContext,
    shared: SharedAnalysisState,
    exclude_node_modules: bool,
    excluded_roots: Vec<PathBuf>,
    file_filters: Vec<EntryPredicate>,
}

impl ProjectProcessor {
    pub fn new(
        workspace_root: PathBuf,
        project_path: PathBuf,
        project_name: String,
        project_ts_config: TSConfig,
        shared: SharedAnalysisState,
        source_map: Arc<SourceMap>,
        default_standalone: bool,
    ) -> Self {
        let context = AnalysisContext {
            workspace_root,
            project_path,
            project_name: Arc::new(project_name),
            project_ts_config,
            source_map,
            file_reader: shared.file_reader.clone(),
            default_standalone,
        };

        ProjectProcessor {
            context,
            shared,
            exclude_node_modules: false,
            excluded_roots: Vec::new(),
            file_filters: Vec::new(),
        }
    }

    /// Prunes node_modules directories from the walk entirely.
    pub fn filter_node_modules(mut self) -> Self {
        self.exclude_node_modules = true;
        self
    }

    /// Prunes roots of OTHER projects nested inside this one — their files
    /// belong to the nested project and must not be processed twice.
    pub fn exclude_nested_roots(mut self, roots: Vec<PathBuf>) -> Self {
        self.excluded_roots = roots;
        self
    }

    /// Restricts analysis to TypeScript files (.ts/.tsx).
    pub fn filter_ts_files(self) -> Self {
        self.filter(|entry| {
            entry
                .path()
                .extension()
                .is_some_and(|ext| ext == "ts" || ext == "tsx")
        })
    }

    /// Restricts analysis to any parseable script files.
    pub fn filter_script_files(self) -> Self {
        self.filter(|entry| {
            entry.path().extension().is_some_and(|ext| {
                matches!(
                    ext.to_string_lossy().as_ref(),
                    "ts" | "tsx" | "js" | "jsx" | "mjs" | "cjs" | "mts" | "cts"
                )
            })
        })
    }

    pub fn filter<P>(mut self, predicate: P) -> Self
    where
        P: Fn(&walkdir::DirEntry) -> bool + Send + Sync + 'static,
    {
        self.file_filters.push(Box::new(predicate));
        self
    }

    pub fn collect_paths(&self) -> Vec<PathBuf> {
        let exclude_node_modules = self.exclude_node_modules;
        let excluded_roots = self.excluded_roots.clone();
        WalkDir::new(crate::nx::nx_workspace::walkable_root(
            &self.context.project_path,
        ))
        .into_iter()
        .filter_entry(move |entry| {
            if exclude_node_modules
                && entry.depth() > 0
                && crate::analysis::utils::path_utils::is_ignored_dir_component(entry.file_name())
            {
                return false;
            }
            // Excluded roots are normalized project roots; the walk yields
            // raw paths (`./apps/x` under `-d .`) — compare like with like.
            let entry_path = crate::analysis::utils::path_utils::normalize_path(entry.path());
            !excluded_roots.contains(&entry_path)
        })
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry.file_type().is_file()
                && self.file_filters.iter().all(|predicate| predicate(entry))
        })
        // Normalized: file paths are join keys against normalized resolved
        // import paths (see unused/usages analyses).
        .map(|e| crate::analysis::utils::path_utils::normalize_path(e.path()))
        .collect()
    }

    pub fn process_files<T: AnalysisCollector>(&self, results: &mut T) {
        let files = self.collect_paths();
        let context = &self.context;

        let chunk_results: Vec<T> = files
            .par_chunks(10)
            .map(|chunk| {
                let mut local_resolver = ImportResolver::new(
                    &context.workspace_root,
                    Some(self.shared.import_cache.clone()),
                    self.shared.module_cache.clone(),
                    Arc::clone(&self.shared.import_graph),
                );

                let mut chunk_results = T::default();

                for path in chunk {
                    match T::process_file(path, &mut local_resolver, context) {
                        Ok(file_results) => chunk_results.extend(file_results),
                        Err(e) => eprintln!("⚠️ Failed to process {:?}: {}", path, e),
                    }
                }

                chunk_results
            })
            .collect();

        for chunk_result in chunk_results {
            results.extend(chunk_result);
        }
    }
}
