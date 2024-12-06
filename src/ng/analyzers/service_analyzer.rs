use crate::analysis::models::import::ResolvedImport;
use crate::ng::analyzers::decorator_analyzer::{DecoratorAnalysis, DecoratorAnalyzer};
use crate::ng::models::NgServiceInfo;
use std::path::Path;

pub struct NgServiceAnalyzer;

impl NgServiceAnalyzer {
    pub fn analyze(
        analysis: &DecoratorAnalysis,
        file_path: &Path,
        project_root: &Path,
        class_name: &str,
        package_name: &String,
        imports: &Vec<ResolvedImport>,
    ) -> Option<NgServiceInfo> {
        if analysis.name != "Injectable" {
            return None;
        }
        let props = analysis.raw_props.as_ref()?;

        let service = NgServiceInfo::new(
            class_name.to_string(),
            DecoratorAnalyzer::get_string_prop(props, "providedIn")
                .unwrap_or_else(|| "root".to_string()),
            imports.clone(),
            file_path.to_path_buf(),
            crate::analysis::utils::path_utils::get_relative_path(file_path, project_root),
            package_name.clone(),
        );

        Some(service)
    }
}
