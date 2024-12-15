use crate::ng::analysis::dependency::analyzer::NgDependencyAnalyzer;
use crate::ng::analysis::dependency::maps::{NgPathMap, NgSelectorMaps};
use crate::ng::models::NgAnalysisResults;

pub struct NgAnalysisProcessor<'a> {
    results: &'a mut NgAnalysisResults,
}

impl<'a> NgAnalysisProcessor<'a> {
    pub fn new(results: &'a mut NgAnalysisResults) -> Self {
        Self { results }
    }

    pub fn build_maps(results: &NgAnalysisResults) -> (NgSelectorMaps, NgPathMap) {
        let selector_maps = NgSelectorMaps::build_from_results(&results);
        let path_map = NgPathMap::build_from_results(&results);

        (selector_maps, path_map)
    }

    pub fn analyze_dependencies(&mut self) {
        let (selector_maps, path_map) = Self::build_maps(&self.results);

        let analyzer = NgDependencyAnalyzer::new(selector_maps, path_map);
        analyzer.analyze_dependencies(&mut self.results);
    }
}
