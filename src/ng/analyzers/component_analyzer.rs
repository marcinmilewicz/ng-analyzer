use crate::analysis::models::import::ResolvedImport;
use crate::file_cache_reader::CachedFileReader;
use crate::ng::analyzers::decorator_analyzer::{DecoratorAnalysis, DecoratorAnalyzer};
use crate::ng::analyzers::template_analyzer::TemplateParser;
use crate::ng::models::NgComponentInfo;
use std::path::Path;

pub struct NgComponentAnalyzer;

impl NgComponentAnalyzer {
    pub fn analyze(
        analysis: &DecoratorAnalysis,
        file_path: &Path,
        project_root: &Path,
        class_name: &str,
        package_name: &String,
        imports: Vec<ResolvedImport>,
        file_reader: &CachedFileReader,
    ) -> Option<NgComponentInfo> {
        if analysis.name != "Component" {
            return None;
        }
        let props = analysis.raw_props.as_ref()?;
        let template_path =
            DecoratorAnalyzer::get_string_prop(props, "templateUrl").unwrap_or_default();

        let mut template_parser = TemplateParser::new();

        let mut component = NgComponentInfo::new(
            class_name.to_string(),
            DecoratorAnalyzer::get_string_prop(props, "selector").unwrap_or_default(),
            DecoratorAnalyzer::get_string_prop(props, "templateUrl").unwrap_or_default(),
            DecoratorAnalyzer::get_string_array_prop(props, "styleUrls"),
            DecoratorAnalyzer::get_bool_prop(props, "standalone").unwrap_or(false),
            imports,
            file_path.to_path_buf(),
            crate::analysis::utils::path_utils::get_relative_path(file_path, project_root),
            package_name.clone(),
        );

        if !template_path.is_empty() {
            let template_file_path = file_path.parent().unwrap().join(&template_path);
            if let Ok(template_content) = file_reader.read_file(&template_file_path) {
                component.template_usages = template_parser.parse_template(&template_content);
            }
        }

        Some(component)
    }
}
