use crate::analysis::models::import::ResolvedImport;
use crate::ng::analyzers::decorator_analyzer::{DecoratorAnalysis, DecoratorAnalyzer};
use crate::ng::models::ng_pipe::NgPipeInfo;
use std::path::Path;

pub struct NgPipeAnalyzer;

impl NgPipeAnalyzer {
    pub fn analyze(
        analysis: &DecoratorAnalysis,
        file_path: &Path,
        project_root: &Path,
        class_name: &str,
        package_name: &String,
        imports: Vec<ResolvedImport>,
    ) -> Option<NgPipeInfo> {
        if analysis.name != "Pipe" {
            return None;
        }

        let props = analysis.raw_props.as_ref()?;

        let pipe = NgPipeInfo::new(
            class_name.to_string(),
            DecoratorAnalyzer::get_string_prop(props, "name").unwrap_or_default(),
            DecoratorAnalyzer::get_bool_prop(props, "pure").unwrap_or(true),
            DecoratorAnalyzer::get_bool_prop(props, "standalone").unwrap_or(false),
            imports,
            file_path.to_path_buf(),
            crate::analysis::utils::path_utils::get_relative_path(file_path, project_root),
            package_name.clone(),
        );

        Some(pipe)
    }
}
