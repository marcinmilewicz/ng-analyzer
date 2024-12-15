use std::path::Path;
use swc_ecma_ast::{Expr, Lit, PropName};

use crate::analysis::models::import::ResolvedImport;
use crate::analysis::utils::path_utils::get_relative_path;
use crate::ng::analysis::decorator_analyzer::{DecoratorAnalysis, DecoratorAnalyzer};
use crate::ng::models::ng_directive::NgDirectiveInfo;

pub struct NgDirectiveAnalyzer;

impl NgDirectiveAnalyzer {
    pub fn analyze(
        analysis: &DecoratorAnalysis,
        file_path: &Path,
        base_path: &Path,
        class_name: &str,
        package_name: &String,
        imports: Vec<ResolvedImport>,
    ) -> Option<NgDirectiveInfo> {
        if analysis.name != "Directive" {
            return None;
        }

        let props = analysis.raw_props.as_ref()?;

        let mut directive = NgDirectiveInfo::new(
            class_name.to_string(),
            DecoratorAnalyzer::get_string_prop(props, "selector").unwrap_or_default(),
            DecoratorAnalyzer::get_bool_prop(props, "standalone").unwrap_or(false),
            Vec::new(),
            Vec::new(),
            imports,
            file_path.to_path_buf(),
            get_relative_path(file_path, base_path),
            package_name.clone(),
        );

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
