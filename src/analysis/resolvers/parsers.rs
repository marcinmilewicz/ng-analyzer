use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use swc_common::input::StringInput;
use swc_common::{FileName, SourceMap};
use swc_ecma_ast::{ExportSpecifier, ModuleDecl, ModuleExportName, ModuleItem};
use swc_ecma_parser::lexer::Lexer;
use swc_ecma_parser::{Parser, Syntax, TsSyntax};

pub struct ImportParser;

impl ImportParser {
    pub fn find_export_declaration(&self, path: &Path, target_name: &str) -> Option<PathBuf> {
        let mut visited_paths = std::collections::HashSet::new();
        self.find_export_declaration_recursive(path, target_name, &mut visited_paths)
    }

    fn find_export_declaration_recursive(
        &self,
        path_from_export: &Path,
        target_name: &str,
        visited_paths: &mut std::collections::HashSet<PathBuf>,
    ) -> Option<PathBuf> {
        let paths = self.resolve_path_with_extensions(path_from_export)?;
        let path = paths.as_path();
        if !visited_paths.insert(path.to_path_buf()) {
            return None;
        }

        let source = fs::read_to_string(&path).ok()?;
        let cm = SourceMap::default();
        let fm = cm.new_source_file(Arc::from(FileName::Real(path.to_path_buf())), source);

        let lexer = Lexer::new(
            Syntax::Typescript(TsSyntax {
                tsx: false,
                decorators: true,
                ..Default::default()
            }),
            Default::default(),
            StringInput::from(&*fm),
            None,
        );

        let mut parser = Parser::new_from(lexer);
        let module = parser.parse_module().ok()?;

        for item in &module.body {
            if let ModuleItem::ModuleDecl(ModuleDecl::ExportDecl(export_decl)) = item {
                let found_name = match &export_decl.decl {
                    swc_ecma_ast::Decl::Class(class_decl) => Some(class_decl.ident.sym.to_string()),
                    swc_ecma_ast::Decl::Fn(fn_decl) => Some(fn_decl.ident.sym.to_string()),
                    swc_ecma_ast::Decl::Var(var_decl) => var_decl.decls.iter().find_map(|decl| {
                        if let swc_ecma_ast::Pat::Ident(ident) = &decl.name {
                            Some(ident.id.sym.to_string())
                        } else {
                            None
                        }
                    }),
                    swc_ecma_ast::Decl::TsInterface(ts_interface) => {
                        Some(ts_interface.id.sym.to_string())
                    }
                    swc_ecma_ast::Decl::TsTypeAlias(ts_type_alias) => {
                        Some(ts_type_alias.id.sym.to_string())
                    }
                    swc_ecma_ast::Decl::TsEnum(ts_enum) => Some(ts_enum.id.sym.to_string()),
                    swc_ecma_ast::Decl::TsModule(ts_module) => match &ts_module.id {
                        swc_ecma_ast::TsModuleName::Ident(ident) => Some(ident.sym.to_string()),
                        swc_ecma_ast::TsModuleName::Str(str) => Some(str.value.to_string()),
                    },
                    _ => None,
                };

                if let Some(name) = found_name {
                    if name == target_name {
                        return Some(path.to_path_buf());
                    }
                }
            }
        }

        for item in &module.body {
            match item {
                ModuleItem::ModuleDecl(ModuleDecl::ExportAll(export_all)) => {
                    let relative_path = export_all.src.value.to_string();

                    if let Some(new_path) = self.resolve_export_path(&relative_path, path) {
                        if let Some(result) = self.find_export_declaration_recursive(
                            &new_path,
                            target_name,
                            visited_paths,
                        ) {
                            return Some(result);
                        }
                    }
                }
                ModuleItem::ModuleDecl(ModuleDecl::ExportNamed(export_named)) => {
                    for specifier in &export_named.specifiers {
                        if let ExportSpecifier::Named(named_export) = specifier {
                            let export_name = match &named_export.exported {
                                Some(exported) => match exported {
                                    ModuleExportName::Ident(i) => i.sym.to_string(),
                                    ModuleExportName::Str(s) => s.value.to_string(),
                                },
                                None => match &named_export.orig {
                                    ModuleExportName::Ident(i) => i.sym.to_string(),
                                    ModuleExportName::Str(s) => s.value.to_string(),
                                },
                            };

                            if export_name == target_name {
                                if let Some(src) = &export_named.src {
                                    if let Some(new_path) =
                                        self.resolve_export_path(&src.value, path)
                                    {
                                        if let Some(result) = self
                                            .find_export_declaration_recursive(
                                                &new_path,
                                                &match &named_export.orig {
                                                    ModuleExportName::Ident(i) => i.sym.to_string(),
                                                    ModuleExportName::Str(s) => s.value.to_string(),
                                                },
                                                visited_paths,
                                            )
                                        {
                                            return Some(result);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                _ => continue,
            }
        }

        None
    }

    fn resolve_path_with_extensions(&self, path: &Path) -> Option<PathBuf> {
        if path.exists() {
            return Some(path.to_path_buf());
        }

        let path_str = path.to_string_lossy();

        for ext in &[".ts", ".tsx", ".js", ".d.ts"] {
            let with_ext = PathBuf::from(format!("{}{}", path_str, ext));
            if with_ext.exists() {
                return Some(with_ext);
            }
        }

        let as_dir = path.join("index");
        for ext in &[".ts", ".tsx", ".js", ".d.ts"] {
            let with_ext = PathBuf::from(format!("{}{}", as_dir.display(), ext));
            if with_ext.exists() {
                return Some(with_ext);
            }
        }

        None
    }

    fn resolve_export_path(&self, relative_path: &str, current_file: &Path) -> Option<PathBuf> {
        let parent = current_file.parent()?;
        let mut resolved = parent.join(relative_path);

        if !resolved.extension().is_some() {
            for ext in &[".ts", ".tsx", ".d.ts"] {
                let with_ext = PathBuf::from(format!("{}{}", resolved.display(), ext));
                if with_ext.exists() {
                    return Some(with_ext);
                }
            }

            resolved = resolved.join("index");
            for ext in &[".ts", ".tsx", ".d.ts"] {
                let with_ext = PathBuf::from(format!("{}{}", resolved.display(), ext));
                if with_ext.exists() {
                    return Some(with_ext);
                }
            }
        }

        Some(resolved)
    }
}
