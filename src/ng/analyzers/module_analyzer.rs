use crate::ng::analyzers::decorator_analyzer::DecoratorAnalysis;
use crate::ng::models::NgModuleInfo;
use std::path::Path;
use swc_ecma_ast::{Expr, PropName};

pub struct NgModuleAnalyzer;

impl NgModuleAnalyzer {
    pub fn analyze(
        analysis: &DecoratorAnalysis,
        file_path: &Path,
        project_root: &Path,
        class_name: &str,
        package_name: &String,
    ) -> Option<NgModuleInfo> {
        if analysis.name != "NgModule" {
            return None;
        }

        let props = analysis.raw_props.as_ref()?;

        let declarations = Self::extract_string_array_identifiers(props, "declarations");
        let exports = Self::extract_string_array_identifiers(props, "exports");
        let providers = Self::extract_string_array_identifiers(props, "providers");

        let module = NgModuleInfo::new(
            class_name.to_string(),
            declarations,
            exports,
            providers,
            file_path.to_path_buf(),
            crate::analysis::utils::path_utils::get_relative_path(file_path, project_root),
            package_name.clone(),
        );

        Some(module)
    }

    fn extract_string_array_identifiers(
        obj: &swc_ecma_ast::ObjectLit,
        prop_name: &str,
    ) -> Vec<String> {
        for prop in &obj.props {
            if let swc_ecma_ast::PropOrSpread::Prop(prop) = prop {
                if let swc_ecma_ast::Prop::KeyValue(kv) = &**prop {
                    if let PropName::Ident(key) = &kv.key {
                        if key.sym == *prop_name {
                            if let Expr::Array(arr) = &*kv.value {
                                return arr
                                    .elems
                                    .iter()
                                    .flatten()
                                    .filter_map(|elem| {
                                        if let Expr::Ident(ident) = &*elem.expr {
                                            Some(ident.sym.to_string())
                                        } else {
                                            None
                                        }
                                    })
                                    .collect();
                            }
                        }
                    }
                }
            }
        }
        Vec::new()
    }
}
