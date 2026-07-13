use crate::analysis::processor::context::AnalysisContext;
use crate::analysis::resolvers::import_resolver::ImportResolver;
use std::path::Path;

pub trait AnalysisCollector: Default + Send {
    fn extend(&mut self, other: Self);
    fn process_file(
        path: &Path,
        resolver: &mut ImportResolver,
        context: &AnalysisContext,
    ) -> Result<Self, Box<dyn std::error::Error>>;
}
