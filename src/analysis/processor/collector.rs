use crate::analysis::processor::context::AnalysisContext;
use crate::analysis::resolvers::import_resolver::ImportResolver;
use std::path::PathBuf;

pub trait AnalysisCollector: Default + Send {
    fn extend(&mut self, other: Self);
    fn process_file(
        path: &PathBuf,
        resolver: &mut ImportResolver,
        context: &AnalysisContext,
    ) -> Result<Self, Box<dyn std::error::Error>>;
}
