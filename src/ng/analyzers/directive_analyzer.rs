use std::path::Path;
use swc_ecma_ast::{Expr, Lit, PropName};

use crate::analysis::models::import::ResolvedImport;
use crate::ng::analyzers::decorator_analyzer::{DecoratorAnalysis, DecoratorAnalyzer};
use crate::ng::models::ng_base::NgBaseInfo;
use crate::ng::models::ng_directive::NgDirectiveInfo;

pub struct NgDirectiveAnalyzer;

impl NgDirectiveAnalyzer {
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
    ) -> Option<NgDirectiveInfo> {
        if analysis.name != "Directive" {
            return None;
        }

        let props = analysis.raw_props.as_ref()?;

        let mut directive = NgDirectiveInfo {
            base: NgBaseInfo::new(
                class_name.to_string(),
                imports,
                file_path.to_path_buf(),
                crate::analysis::utils::path_utils::get_relative_path(file_path, project_root),
                package_name.to_string(),
            ),
            selector: DecoratorAnalyzer::get_string_prop(props, "selector").unwrap_or_default(),
            standalone: DecoratorAnalyzer::get_bool_prop(props, "standalone")
                .unwrap_or(default_standalone),
            host_bindings: Vec::new(),
            host_listeners: Vec::new(),
            inputs,
            outputs,
        };

        if let Some(host_props) = props.props.iter().find_map(|p| {
            if let swc_ecma_ast::PropOrSpread::Prop(p) = p {
                if let swc_ecma_ast::Prop::KeyValue(kv) = &**p {
                    if let PropName::Ident(key) = &kv.key {
                        if key.sym == *"host" {
                            if let Expr::Object(obj) = &*kv.value {
                                return Some(obj);
                            }
                        }
                    }
                }
            }
            None
        }) {
            for prop in &host_props.props {
                if let swc_ecma_ast::PropOrSpread::Prop(p) = prop {
                    if let swc_ecma_ast::Prop::KeyValue(kv) = &**p {
                        if let Expr::Lit(Lit::Str(str_lit)) = &*kv.value {
                            directive.host_bindings.push(str_lit.value.to_string());
                        }
                    }
                }
            }
        }

        Some(directive)
    }
}
