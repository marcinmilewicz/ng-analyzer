use crate::analysis::models::import::ResolvedImport;
use crate::ng::analyzers::decorator_analyzer::{DecoratorAnalysis, DecoratorAnalyzer};
use crate::ng::models::ng_base::NgBaseInfo;
use crate::ng::models::ng_pipe::NgPipeInfo;
use std::path::Path;

pub struct NgPipeAnalyzer;

impl NgPipeAnalyzer {
    #[allow(clippy::too_many_arguments)]
    pub fn analyze(
        analysis: &DecoratorAnalysis,
        file_path: &Path,
        project_root: &Path,
        class_name: &str,
        package_name: &str,
        imports: Vec<ResolvedImport>,
        default_standalone: bool,
    ) -> Option<NgPipeInfo> {
        if analysis.name != "Pipe" {
            return None;
        }

        let props = analysis.raw_props.as_ref()?;

        Some(NgPipeInfo {
            base: NgBaseInfo::new(
                class_name.to_string(),
                imports,
                file_path.to_path_buf(),
                crate::analysis::utils::path_utils::get_relative_path(file_path, project_root),
                package_name.to_string(),
            ),
            name: DecoratorAnalyzer::get_string_prop(props, "name").unwrap_or_default(),
            pure: DecoratorAnalyzer::get_bool_prop(props, "pure").unwrap_or(true),
            standalone: DecoratorAnalyzer::get_bool_prop(props, "standalone")
                .unwrap_or(default_standalone),
        })
    }
}
