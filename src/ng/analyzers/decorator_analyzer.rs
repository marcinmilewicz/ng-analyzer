use dashmap::DashMap;
use std::sync::Arc;
use swc_ecma_ast::{Decorator, Expr, Lit, PropName};

#[derive(Clone)]
pub struct DecoratorAnalysisCache {
    cache: Arc<DashMap<String, DecoratorAnalysis>>,
}

#[derive(Clone)]
pub struct DecoratorAnalysis {
    pub name: String,
    pub raw_props: Option<swc_ecma_ast::ObjectLit>,
}

impl DecoratorAnalysisCache {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(DashMap::new()),
        }
    }

    pub fn get_or_analyze(
        &self,
        decorator: &Decorator,
        class_name: &str,
    ) -> Option<DecoratorAnalysis> {
        let cache_key = format!("{}_{}", class_name, self.get_decorator_hash(decorator));

        if let Some(analysis) = self.cache.get(&cache_key) {
            return Some(analysis.clone());
        }

        let analysis = self.analyze_decorator(decorator)?;
        self.cache.insert(cache_key, analysis.clone());
        Some(analysis)
    }

    fn get_decorator_hash(&self, decorator: &Decorator) -> String {
        format!("{:p}", decorator)
    }

    fn analyze_decorator(&self, decorator: &Decorator) -> Option<DecoratorAnalysis> {
        if let Expr::Call(call) = &*decorator.expr {
            if let swc_ecma_ast::Callee::Expr(expr) = &call.callee {
                if let Expr::Ident(ident) = expr.as_ref() {
                    let name = ident.sym.to_string();

                    let mut raw_props = None;

                    if let Some(first_arg) = call.args.first() {
                        if let Expr::Object(obj) = &*first_arg.expr {
                            raw_props = Some(obj.clone());
                        }
                    }

                    return Some(DecoratorAnalysis { name, raw_props });
                }
            }
        }
        None
    }
}

pub struct DecoratorAnalyzer;

impl DecoratorAnalyzer {


    pub fn get_string_prop(obj: &swc_ecma_ast::ObjectLit, prop_name: &str) -> Option<String> {
        for prop in &obj.props {
            if let swc_ecma_ast::PropOrSpread::Prop(prop) = prop {
                if let swc_ecma_ast::Prop::KeyValue(kv) = &**prop {
                    if let PropName::Ident(key) = &kv.key {
                        if key.sym == *prop_name {
                            if let Expr::Lit(Lit::Str(str_lit)) = &*kv.value {
                                return Some(str_lit.value.to_string());
                            }
                        }
                    }
                }
            }
        }
        None
    }

    pub fn get_bool_prop(obj: &swc_ecma_ast::ObjectLit, prop_name: &str) -> Option<bool> {
        for prop in &obj.props {
            if let swc_ecma_ast::PropOrSpread::Prop(prop) = prop {
                if let swc_ecma_ast::Prop::KeyValue(kv) = &**prop {
                    if let PropName::Ident(key) = &kv.key {
                        if key.sym == *prop_name {
                            if let Expr::Lit(Lit::Bool(bool_lit)) = &*kv.value {
                                return Some(bool_lit.value);
                            }
                        }
                    }
                }
            }
        }
        None
    }

    pub fn get_string_array_prop(obj: &swc_ecma_ast::ObjectLit, prop_name: &str) -> Vec<String> {
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
                                        if let Expr::Lit(Lit::Str(str_lit)) = &*elem.expr {
                                            Some(str_lit.value.to_string())
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
