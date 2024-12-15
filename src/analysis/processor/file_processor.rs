use crate::analysis::models::ts_config::TSConfig;
use crate::analysis::processor::collector::AnalysisCollector;
use crate::analysis::processor::context::AnalysisContext;
use crate::analysis::resolvers::cache::ImportCache;
use crate::analysis::resolvers::import_graph::ImportGraph;
use crate::analysis::resolvers::import_resolver::ImportResolver;
use crate::file_cache_reader::CachedFileReader;
use rayon::prelude::*;
use std::path::PathBuf;
use std::sync::Arc;
use swc_common::SourceMap;
use walkdir::WalkDir;

pub struct ProjectProcessor {
    context: AnalysisContext,
    shared_cache: ImportCache,
    import_graph: Arc<ImportGraph>,
}

impl ProjectProcessor {
    pub fn new(
        base_path: PathBuf,
        project_path:PathBuf,
        project_name: String,
        project_ts_config: TSConfig,
        shared_cache: ImportCache,
        shared_file_reader: CachedFileReader,
        source_map: Arc<SourceMap>,
        import_graph: Arc<ImportGraph>,
    ) -> Self {
        let context = AnalysisContext {
            base_path,
            project_path,
            project_name: Arc::new(project_name),
            project_ts_config,
            source_map,
            file_reader: shared_file_reader,
        };

        ProjectProcessor {
            context,
            shared_cache,
            import_graph,
        }
    }

    pub fn filter_node_modules(self) -> Self {
        self.filter(|entry| !crate::analysis::utils::path_utils::is_node_modules(entry.path()))
    }

    pub fn filter_ts_files(self) -> Self {
        self.filter(|entry| {
            entry.file_type().is_file() && entry.path().extension().map_or(false, |ext| ext == "ts")
        })
    }

    pub fn filter<P>(self, predicate: P) -> Self
    where
        P: Fn(&walkdir::DirEntry) -> bool + Send + Sync + 'static,
    {
        Self {
            context: self.context,
            shared_cache: self.shared_cache,
            import_graph: self.import_graph,
        }
    }

    pub fn collect_paths(&self) -> Vec<PathBuf> {
        WalkDir::new(&self.context.project_path)
            .into_iter()
            .filter_entry(|entry| !crate::analysis::utils::path_utils::is_node_modules(entry.path()))
            .filter_map(|entry| entry.ok())
            .filter(|entry| {
                entry.file_type().is_file() && entry.path().extension().map_or(false, |ext| ext == "ts")
            })
            .map(|e| e.path().to_path_buf())
            .collect()
    }

    pub fn process_files<T: AnalysisCollector>(&self, results: &mut T) {
        let files = self.collect_paths();
        let context = &self.context;

        let chunk_results: Vec<T> = files
            .par_chunks(10)
            .map(|chunk| {
                let mut local_resolver = ImportResolver::new(
                    context.base_path.clone(),
                    context.project_path.clone(),
                    Some(self.shared_cache.clone_cache()),
                    Arc::clone(&self.import_graph),
                );

                let mut chunk_results = T::default();

                for file_path in chunk {
                    if let Ok(file_results) = T::process_file(file_path, &mut local_resolver, context) {
                        chunk_results.extend(file_results);
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
