use crate::ng::analyzers::decorator_analyzer::{DecoratorAnalysis, DecoratorAnalyzer};
use crate::ng::models::ng_base::NgBaseInfo;
use crate::ng::models::NgModuleInfo;
use std::path::Path;

pub struct NgModuleAnalyzer;

impl NgModuleAnalyzer {
    pub fn analyze(
        analysis: &DecoratorAnalysis,
        file_path: &Path,
        project_root: &Path,
        class_name: &str,
        package_name: &str,
    ) -> Option<NgModuleInfo> {
        if analysis.name != "NgModule" {
            return None;
        }

        let props = analysis.raw_props.as_ref()?;

        Some(NgModuleInfo {
            base: NgBaseInfo::new(
                class_name.to_string(),
                Vec::new(),
                file_path.to_path_buf(),
                crate::analysis::utils::path_utils::get_relative_path(file_path, project_root),
                package_name.to_string(),
            ),
            declarations: DecoratorAnalyzer::get_ident_array_prop(props, "declarations"),
            imports_idents: DecoratorAnalyzer::get_ident_array_prop(props, "imports"),
            exports: DecoratorAnalyzer::get_ident_array_prop(props, "exports"),
            providers: DecoratorAnalyzer::get_ident_array_prop(props, "providers"),
            bootstrap: DecoratorAnalyzer::get_ident_array_prop(props, "bootstrap"),
        })
    }
}
