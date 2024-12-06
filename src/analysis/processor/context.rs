use crate::analysis::models::ts_config::TSConfig;
use crate::file_cache_reader::CachedFileReader;
use std::path::PathBuf;
use std::sync::Arc;
use swc_common::SourceMap;

pub struct AnalysisContext {
    pub project_path: PathBuf,
    pub project_name: Arc<String>,
    pub project_ts_config: TSConfig,
    pub source_map: Arc<SourceMap>,
    pub file_reader: CachedFileReader,
}
