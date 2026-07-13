use swc_ecma_ast::{Decorator, Expr, Lit, PropName};

#[derive(Clone)]
pub struct DecoratorAnalysis {
    pub name: String,
    pub raw_props: Option<swc_ecma_ast::ObjectLit>,
}

pub struct DecoratorAnalyzer;

impl DecoratorAnalyzer {
    /// Extracts the decorator name and its first object-literal argument,
    /// e.g. `@Component({ selector: ... })` → ("Component", { ... }).
    pub fn analyze(decorator: &Decorator) -> Option<DecoratorAnalysis> {
        let Expr::Call(call) = &*decorator.expr else {
            return None;
        };
        let swc_ecma_ast::Callee::Expr(callee) = &call.callee else {
            return None;
        };
        let Expr::Ident(ident) = callee.as_ref() else {
            return None;
        };

        let raw_props = call.args.first().and_then(|arg| {
            if let Expr::Object(obj) = &*arg.expr {
                Some(obj.clone())
            } else {
                None
            }
        });

        Some(DecoratorAnalysis {
            name: ident.sym.to_string(),
            raw_props,
        })
    }

    fn find_prop<'a>(
        obj: &'a swc_ecma_ast::ObjectLit,
        prop_name: &str,
    ) -> Option<&'a swc_ecma_ast::Expr> {
        obj.props.iter().find_map(|prop| {
            if let swc_ecma_ast::PropOrSpread::Prop(prop) = prop {
                if let swc_ecma_ast::Prop::KeyValue(kv) = &**prop {
                    if let PropName::Ident(key) = &kv.key {
                        if key.sym == *prop_name {
                            return Some(&*kv.value);
                        }
                    }
                }
            }
            None
        })
    }

    pub fn get_string_prop(obj: &swc_ecma_ast::ObjectLit, prop_name: &str) -> Option<String> {
        match Self::find_prop(obj, prop_name)? {
            Expr::Lit(Lit::Str(str_lit)) => Some(str_lit.value.to_string()),
            _ => None,
        }
    }

    pub fn get_bool_prop(obj: &swc_ecma_ast::ObjectLit, prop_name: &str) -> Option<bool> {
        match Self::find_prop(obj, prop_name)? {
            Expr::Lit(Lit::Bool(bool_lit)) => Some(bool_lit.value),
            _ => None,
        }
    }

    /// Array of identifiers, e.g. `imports: [CommonModule, UiButtonComponent]`.
    pub fn get_ident_array_prop(obj: &swc_ecma_ast::ObjectLit, prop_name: &str) -> Vec<String> {
        match Self::find_prop(obj, prop_name) {
            Some(Expr::Array(arr)) => arr
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
                .collect(),
            _ => Vec::new(),
        }
    }

    pub fn get_string_array_prop(obj: &swc_ecma_ast::ObjectLit, prop_name: &str) -> Vec<String> {
        match Self::find_prop(obj, prop_name) {
            Some(Expr::Array(arr)) => arr
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
                .collect(),
            _ => Vec::new(),
        }
    }
}
