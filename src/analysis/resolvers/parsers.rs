use dashmap::DashMap;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use swc_common::input::StringInput;
use swc_common::{FileName, SourceMap};
use swc_ecma_ast::{ExportSpecifier, Module, ModuleDecl, ModuleExportName, ModuleItem};
use swc_ecma_parser::lexer::Lexer;
use swc_ecma_parser::Parser;

/// Everything barrel-following needs to know about one file, as owned strings.
#[derive(Debug, Default)]
pub struct ExportTable {
    /// `export default …` is present.
    pub has_default: bool,
    /// Names both declared and exported here (`export class X`, and the
    /// `class X {}; export { X }` spelling) — the search ends at this file.
    pub declared: HashSet<String>,
    /// `export { orig as exported } from 'src'` → exported → (orig, src).
    pub forwarded: HashMap<String, (String, String)>,
    /// `export * from 'src'` sources, in source order.
    pub star: Vec<String>,
}

/// Shared cache of per-file export tables. Barrels (`index.ts`) are consulted
/// for every symbol imported through them; without a cache they would be
/// re-parsed once per lookup.
///
/// It caches the extracted TABLE, never the AST. An earlier version stored
/// `Arc<Module>`: swc interns identifiers in a thread-local pool, so an AST
/// parsed on one worker and read on another corrupts that pool — and the
/// damage lands on unrelated files, which silently lose identifiers. That is
/// how `used_import_names` (and with it the decision whether an import counts
/// as a usage) came out different on every run. Owned `String`s cross threads
/// safely; the AST is built, drained and dropped on the thread that parsed it.
#[derive(Clone, Default)]
pub struct ModuleCache {
    // None caches parse failures so broken files are not retried.
    tables: Arc<DashMap<PathBuf, Option<Arc<ExportTable>>>>,
}

impl ModuleCache {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn exports_of(&self, path: &Path) -> Option<Arc<ExportTable>> {
        if let Some(cached) = self.tables.get(path) {
            return cached.clone();
        }

        let table = Self::parse(path)
            .map(|module| Arc::new(Self::extract(&module)))
            .or(None);
        self.tables.insert(path.to_path_buf(), table.clone());
        table
    }

    /// Drains the AST into owned strings. Runs on the parsing thread.
    fn extract(module: &Module) -> ExportTable {
        let mut table = ExportTable::default();

        for item in &module.body {
            let ModuleItem::ModuleDecl(decl) = item else {
                continue;
            };
            match decl {
                ModuleDecl::ExportDefaultDecl(_) | ModuleDecl::ExportDefaultExpr(_) => {
                    table.has_default = true;
                }
                ModuleDecl::ExportDecl(export_decl) => {
                    if let Some(name) = declared_name(&export_decl.decl) {
                        table.declared.insert(name);
                    }
                }
                ModuleDecl::ExportAll(export_all) => {
                    table.star.push(export_all.src.value.to_string());
                }
                ModuleDecl::ExportNamed(export_named) => {
                    for specifier in &export_named.specifiers {
                        let ExportSpecifier::Named(named) = specifier else {
                            continue;
                        };
                        let original = export_name(&named.orig);
                        let exported = named
                            .exported
                            .as_ref()
                            .map(export_name)
                            .unwrap_or_else(|| original.clone());

                        match &export_named.src {
                            Some(src) => {
                                table
                                    .forwarded
                                    .insert(exported, (original, src.value.to_string()));
                            }
                            // `export { X }` with no source — declared here.
                            None => {
                                table.declared.insert(exported);
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        table
    }

    fn parse(path: &Path) -> Option<Module> {
        let source = fs::read_to_string(path).ok()?;
        let cm = SourceMap::default();
        let fm = cm.new_source_file(Arc::from(FileName::Real(path.to_path_buf())), source);

        let lexer = Lexer::new(
            crate::ng::visitors::syntax_for(path),
            Default::default(),
            StringInput::from(&*fm),
            None,
        );

        Parser::new_from(lexer).parse_module().ok()
    }
}

fn export_name(name: &ModuleExportName) -> String {
    match name {
        ModuleExportName::Ident(ident) => ident.sym.to_string(),
        ModuleExportName::Str(str) => str.value.to_string(),
    }
}

fn declared_name(decl: &swc_ecma_ast::Decl) -> Option<String> {
    match decl {
        swc_ecma_ast::Decl::Class(class_decl) => Some(class_decl.ident.sym.to_string()),
        swc_ecma_ast::Decl::Fn(fn_decl) => Some(fn_decl.ident.sym.to_string()),
        swc_ecma_ast::Decl::Var(var_decl) => var_decl.decls.iter().find_map(|declarator| {
            if let swc_ecma_ast::Pat::Ident(ident) = &declarator.name {
                Some(ident.id.sym.to_string())
            } else {
                None
            }
        }),
        swc_ecma_ast::Decl::TsInterface(ts_interface) => Some(ts_interface.id.sym.to_string()),
        swc_ecma_ast::Decl::TsTypeAlias(ts_type_alias) => Some(ts_type_alias.id.sym.to_string()),
        swc_ecma_ast::Decl::TsEnum(ts_enum) => Some(ts_enum.id.sym.to_string()),
        swc_ecma_ast::Decl::TsModule(ts_module) => match &ts_module.id {
            swc_ecma_ast::TsModuleName::Ident(ident) => Some(ident.sym.to_string()),
            swc_ecma_ast::TsModuleName::Str(str) => Some(str.value.to_string()),
        },
        _ => None,
    }
}

pub struct ImportParser {
    module_cache: ModuleCache,
}

impl ImportParser {
    pub fn new(module_cache: ModuleCache) -> Self {
        Self { module_cache }
    }

    /// `resolve_specifier` resolves a NON-relative re-export source
    /// (`export { X } from '@org/y'`) exactly like an import — tsconfig
    /// paths, then node_modules/workspace packages.
    pub fn find_export_declaration(
        &self,
        path: &Path,
        target_name: &str,
        resolve_specifier: &dyn Fn(&str, &Path) -> Option<PathBuf>,
    ) -> Option<PathBuf> {
        let mut visited_paths = std::collections::HashSet::new();
        self.find_export_declaration_recursive(
            path,
            target_name,
            resolve_specifier,
            &mut visited_paths,
        )
    }

    fn find_export_declaration_recursive(
        &self,
        path_from_export: &Path,
        target_name: &str,
        resolve_specifier: &dyn Fn(&str, &Path) -> Option<PathBuf>,
        visited_paths: &mut std::collections::HashSet<PathBuf>,
    ) -> Option<PathBuf> {
        let paths = self.resolve_path_with_extensions(path_from_export)?;
        let path = paths.as_path();
        if !visited_paths.insert(path.to_path_buf()) {
            return None;
        }

        let exports = self.module_cache.exports_of(path)?;

        // Barrel `export { default as X } from './y'` recurses here looking
        // for the target file's DEFAULT export.
        if target_name == "default" && exports.has_default {
            return Some(path.to_path_buf());
        }

        // Declared here (`export class X`, or `class X {}; export { X }`).
        if exports.declared.contains(target_name) {
            return Some(path.to_path_buf());
        }

        // An explicit `export { X } from './y'` names the source precisely, so
        // it is followed before any `export *` — which is also what TypeScript
        // does: a named re-export shadows a star.
        if let Some((original, source)) = exports.forwarded.get(target_name) {
            if let Some(new_path) = self.resolve_export_path(source, path, resolve_specifier) {
                if let Some(result) = self.find_export_declaration_recursive(
                    &new_path,
                    original,
                    resolve_specifier,
                    visited_paths,
                ) {
                    return Some(result);
                }
            }
        }

        for source in &exports.star {
            if let Some(new_path) = self.resolve_export_path(source, path, resolve_specifier) {
                if let Some(result) = self.find_export_declaration_recursive(
                    &new_path,
                    target_name,
                    resolve_specifier,
                    visited_paths,
                ) {
                    return Some(result);
                }
            }
        }

        None
    }

    fn resolve_path_with_extensions(&self, path: &Path) -> Option<PathBuf> {
        if path.is_file() {
            return Some(path.to_path_buf());
        }

        // `export * from './x.js'` (NodeNext style) → x.ts
        if let Some(mapped) = crate::analysis::resolvers::resolver::map_js_specifier_to_ts(path) {
            return Some(mapped);
        }

        let path_str = path.to_string_lossy();

        for ext in &[".ts", ".tsx", ".js", ".jsx", ".d.ts"] {
            let with_ext = PathBuf::from(format!("{}{}", path_str, ext));
            if with_ext.is_file() {
                return Some(with_ext);
            }
        }

        let as_dir = path.join("index");
        for ext in &[".ts", ".tsx", ".js", ".jsx", ".d.ts"] {
            let with_ext = PathBuf::from(format!("{}{}", as_dir.display(), ext));
            if with_ext.is_file() {
                return Some(with_ext);
            }
        }

        None
    }

    fn resolve_export_path(
        &self,
        specifier: &str,
        current_file: &Path,
        resolve_specifier: &dyn Fn(&str, &Path) -> Option<PathBuf>,
    ) -> Option<PathBuf> {
        if specifier.starts_with("./") || specifier.starts_with("../") {
            let parent = current_file.parent()?;
            return self.resolve_path_with_extensions(&parent.join(specifier));
        }
        // Re-export through a tsconfig alias or a (workspace) package —
        // resolved like any import.
        resolve_specifier(specifier, current_file)
    }
}
