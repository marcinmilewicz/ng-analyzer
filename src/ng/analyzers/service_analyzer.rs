use crate::analysis::models::import::ResolvedImport;
use crate::ng::analyzers::decorator_analyzer::{DecoratorAnalysis, DecoratorAnalyzer};
use crate::ng::models::ng_base::NgBaseInfo;
use crate::ng::models::NgServiceInfo;
use std::path::Path;

pub struct NgServiceAnalyzer;

impl NgServiceAnalyzer {
    pub fn analyze(
        analysis: &DecoratorAnalysis,
        file_path: &Path,
        project_root: &Path,
        class_name: &str,
        package_name: &str,
        imports: &[ResolvedImport],
    ) -> Option<NgServiceInfo> {
        if analysis.name != "Injectable" {
            return None;
        }

        // `@Injectable()` without an argument object is perfectly valid —
        // the service is provided via a providers array elsewhere.
        let provided_in = analysis
            .raw_props
            .as_ref()
            .and_then(|props| DecoratorAnalyzer::get_string_prop(props, "providedIn"));

        Some(NgServiceInfo {
            base: NgBaseInfo::new(
                class_name.to_string(),
                imports.to_vec(),
                file_path.to_path_buf(),
                crate::analysis::utils::path_utils::get_relative_path(file_path, project_root),
                package_name.to_string(),
            ),
            provided_in,
        })
    }
}
