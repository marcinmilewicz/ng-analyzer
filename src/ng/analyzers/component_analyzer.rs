use crate::analysis::models::import::ResolvedImport;
use crate::ng::analyzers::decorator_analyzer::{DecoratorAnalysis, DecoratorAnalyzer};
use crate::ng::models::ng_base::NgBaseInfo;
use crate::ng::models::NgComponentInfo;
use std::path::Path;

pub struct NgComponentAnalyzer;

impl NgComponentAnalyzer {
    #[allow(clippy::too_many_arguments)]
    pub fn analyze(
        analysis: &DecoratorAnalysis,
        file_path: &Path,
        project_root: &Path,
        class_name: &str,
        package_name: &str,
        imports: Vec<ResolvedImport>,
        default_standalone: bool,
        inputs: Vec<String>,
        outputs: Vec<String>,
    ) -> Option<NgComponentInfo> {
        if analysis.name != "Component" {
            return None;
        }
        let props = analysis.raw_props.as_ref()?;

        let mut style_paths = DecoratorAnalyzer::get_string_array_prop(props, "styleUrls");
        if let Some(style_url) = DecoratorAnalyzer::get_string_prop(props, "styleUrl") {
            style_paths.push(style_url);
        }

        Some(NgComponentInfo {
            base: NgBaseInfo::new(
                class_name.to_string(),
                imports,
                file_path.to_path_buf(),
                crate::analysis::utils::path_utils::get_relative_path(file_path, project_root),
                package_name.to_string(),
            ),
            selector: DecoratorAnalyzer::get_string_prop(props, "selector").unwrap_or_default(),
            template_path: DecoratorAnalyzer::get_string_prop(props, "templateUrl")
                .unwrap_or_default(),
            template_inline: DecoratorAnalyzer::get_string_prop(props, "template"),
            style_paths,
            standalone: DecoratorAnalyzer::get_bool_prop(props, "standalone")
                .unwrap_or(default_standalone),
            standalone_imports: DecoratorAnalyzer::get_ident_array_prop(props, "imports"),
            providers: DecoratorAnalyzer::get_ident_array_prop(props, "providers"),
            inputs,
            outputs,
        })
    }
}
