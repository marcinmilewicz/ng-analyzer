use crate::analysis::models::import::{ImportKind, ImportedItem, ResolvedImport};
use crate::analysis::models::ts_config::TSConfig;
use crate::analysis::resolvers::import_resolver::ImportResolver;
use crate::ng::analyzers::component_analyzer::NgComponentAnalyzer;
use crate::ng::analyzers::decorator_analyzer::DecoratorAnalysisCache;
use crate::ng::analyzers::directive_analyzer::NgDirectiveAnalyzer;
use crate::ng::analyzers::module_analyzer::NgModuleAnalyzer;
use crate::ng::analyzers::pipe_analyzer::NgPipeAnalyzer;
use crate::ng::analyzers::service_analyzer::NgServiceAnalyzer;
use crate::ng::models::NgAnalysisResults;
use std::path::Path;
use swc_ecma_ast::{Decl, ImportDecl, Module, ModuleDecl, ModuleItem};
use swc_ecma_visit::Visit;

pub struct AngularVisitor<'a> {
    file_path: std::path::PathBuf,
    project_root: std::path::PathBuf,
    pub results: NgAnalysisResults,
    imports: Vec<ResolvedImport>,
    package_name: String,
    ts_config: TSConfig,
    import_resolver: &'a mut ImportResolver,
}

impl<'a> AngularVisitor<'a> {
    pub fn new(
        file_path: &Path,
        project_root: &Path,
        package_name: String,
        tsconfig: TSConfig,
        import_resolver: &'a mut ImportResolver,
    ) -> Self {
        Self {
            file_path: file_path.to_path_buf(),
            project_root: project_root.to_path_buf(),
            results: NgAnalysisResults::default(),
            imports: Vec::new(),
            package_name,
            ts_config: tsconfig,
            import_resolver,
        }
    }

    fn get_imports(&self) -> &[ResolvedImport] {
        &self.imports
    }

    fn process_decorator(&mut self, decorator: &swc_ecma_ast::Decorator, class_name: &str) {
        let decorator_cache = DecoratorAnalysisCache::new();

        if let Some(analysis) = decorator_cache.get_or_analyze(decorator, class_name) {
            match analysis.name.as_ref() {
                "Component" => {
                    if let Some(component) = NgComponentAnalyzer::analyze(
                        &analysis,
                        &self.file_path,
                        &self.project_root,
                        class_name,
                        &self.package_name,
                        self.imports.clone(),
                    ) {
                        self.results.components.push(component);
                    }
                }
                "Injectable" => {
                    if let Some(service) = NgServiceAnalyzer::analyze(
                        &analysis,
                        &self.file_path,
                        &self.project_root,
                        class_name,
                        &self.package_name,
                        &self.imports,
                    ) {
                        self.results.services.push(service);
                    }
                }
                "NgModule" => {
                    if let Some(module) = NgModuleAnalyzer::analyze(
                        &analysis,
                        &self.file_path,
                        &self.project_root,
                        class_name,
                        &self.package_name,
                    ) {
                        self.results.modules.push(module);
                    }
                }
                "Directive" => {
                    if let Some(directive) = NgDirectiveAnalyzer::analyze(
                        &analysis,
                        &self.file_path,
                        &self.project_root,
                        class_name,
                        &self.package_name,
                        self.imports.clone(),
                    ) {
                        self.results.directives.push(directive);
                    }
                }
                "Pipe" => {
                    if let Some(pipe) = NgPipeAnalyzer::analyze(
                        &analysis,
                        &self.file_path,
                        &self.project_root,
                        class_name,
                        &self.package_name,
                        self.imports.clone(),
                    ) {
                        self.results.pipes.push(pipe);
                    }
                }
                _ => {}
            }
        }
    }

    fn process_import_specifier(&mut self, specifier: &swc_ecma_ast::ImportSpecifier, src: &str) {
        let imported_item = match specifier {
            swc_ecma_ast::ImportSpecifier::Named(named) => ImportedItem {
                name: named.local.sym.to_string(),
                alias: named.imported.as_ref().map(|imported| match imported {
                    swc_ecma_ast::ModuleExportName::Ident(ident) => ident.sym.to_string(),
                    swc_ecma_ast::ModuleExportName::Str(str) => str.value.to_string(),
                }),
                import_kind: ImportKind::Named,
            },
            swc_ecma_ast::ImportSpecifier::Default(default) => ImportedItem {
                name: default.local.sym.to_string(),
                alias: None,
                import_kind: ImportKind::Default,
            },
            swc_ecma_ast::ImportSpecifier::Namespace(namespace) => ImportedItem {
                name: namespace.local.sym.to_string(),
                alias: None,
                import_kind: ImportKind::Namespace,
            },
        };

        let ts_paths = self
            .ts_config
            .compiler_options
            .as_ref()
            .and_then(|opts| opts.paths.as_ref())
            .cloned()
            .unwrap_or_default();

        if let Some(mut resolved_import) =
            self.import_resolver
                .resolve_import(src, &imported_item.name, &self.file_path, ts_paths)
        {
            resolved_import.imported_item = imported_item;
            self.imports.push(resolved_import);
        }
    }
}

impl<'a> Visit for AngularVisitor<'a> {
    fn visit_import_decl(&mut self, import_decl: &ImportDecl) {
        let src = import_decl.src.value.to_string();
        for specifier in &import_decl.specifiers {
            self.process_import_specifier(specifier, &src);
        }
    }

    fn visit_module(&mut self, module: &Module) {
        for item in &module.body {
            if let ModuleItem::ModuleDecl(ModuleDecl::Import(import_decl)) = item {
                self.visit_import_decl(import_decl);
            }
        }

        for item in &module.body {
            if let ModuleItem::ModuleDecl(ModuleDecl::ExportDecl(export_decl)) = item {
                if let Decl::Class(class_decl) = &export_decl.decl {
                    let class_name = &class_decl.ident.sym.to_string();
                    for decorator in &class_decl.class.decorators {
                        self.process_decorator(decorator, class_name);
                    }
                }
            }
        }
    }
}
