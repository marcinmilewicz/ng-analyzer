use crate::analysis::models::file_facts::{ExportInfo, ExportKind, FileFactsInfo, LocalReference};
use crate::analysis::models::import::{ImportKind, ImportedItem, ResolvedImport, UnresolvedImport};
use crate::analysis::models::react::{JsxUsageInfo, ReactComponentInfo};
use crate::analysis::models::ts_config::TSConfig;
use crate::analysis::resolvers::import_resolver::ImportResolver;
use crate::analysis::resolvers::resolver::classify_unresolved;
use crate::ng::analyzers::component_analyzer::NgComponentAnalyzer;
use crate::ng::analyzers::decorator_analyzer::DecoratorAnalyzer;
use crate::ng::analyzers::directive_analyzer::NgDirectiveAnalyzer;
use crate::ng::analyzers::module_analyzer::NgModuleAnalyzer;
use crate::ng::analyzers::pipe_analyzer::NgPipeAnalyzer;
use crate::ng::analyzers::service_analyzer::NgServiceAnalyzer;
use crate::ng::models::NgAnalysisResults;
use std::collections::HashSet;
use std::path::Path;
use swc_ecma_ast::{
    CallExpr, Callee, Class, ClassDecl, ClassMember, Decl, DefaultDecl, ExportDefaultDecl, Expr,
    ImportDecl, JSXAttrName, JSXAttrOrSpread, JSXElementName, JSXOpeningElement, Lit, MemberProp,
    Module, ModuleDecl, ModuleExportName, ModuleItem, PropName, Stmt,
};
use swc_ecma_visit::{Visit, VisitWith};

pub struct AngularVisitor<'a> {
    file_path: std::path::PathBuf,
    project_root: std::path::PathBuf,
    pub results: NgAnalysisResults,
    imports: Vec<ResolvedImport>,
    dynamic_imports: Vec<ResolvedImport>,
    unresolved_imports: Vec<UnresolvedImport>,
    exports: Vec<ExportInfo>,
    local_references: Vec<LocalReference>,
    used_idents: HashSet<String>,
    jsx_usages: Vec<JsxUsageInfo>,
    is_jsx_file: bool,
    package_name: String,
    ts_config: TSConfig,
    default_standalone: bool,
    import_resolver: &'a mut ImportResolver,
}

impl<'a> AngularVisitor<'a> {
    pub fn new(
        file_path: &Path,
        project_root: &Path,
        package_name: String,
        tsconfig: TSConfig,
        default_standalone: bool,
        import_resolver: &'a mut ImportResolver,
    ) -> Self {
        let is_jsx_file = file_path
            .extension()
            .map(|ext| ext == "tsx" || ext == "jsx")
            .unwrap_or(false);

        Self {
            file_path: file_path.to_path_buf(),
            project_root: project_root.to_path_buf(),
            results: NgAnalysisResults::default(),
            imports: Vec::new(),
            dynamic_imports: Vec::new(),
            unresolved_imports: Vec::new(),
            exports: Vec::new(),
            local_references: Vec::new(),
            used_idents: HashSet::new(),
            jsx_usages: Vec::new(),
            is_jsx_file,
            package_name,
            ts_config: tsconfig,
            default_standalone,
            import_resolver,
        }
    }

    fn process_decorated_class(&mut self, class: &Class, class_name: &str) {
        let (inputs, outputs) = Self::extract_inputs_outputs(class);
        for decorator in &class.decorators {
            self.process_decorator(decorator, class_name, inputs.clone(), outputs.clone());
        }
    }

    fn process_decorator(
        &mut self,
        decorator: &swc_ecma_ast::Decorator,
        class_name: &str,
        inputs: Vec<String>,
        outputs: Vec<String>,
    ) {
        let Some(analysis) = DecoratorAnalyzer::analyze(decorator) else {
            return;
        };

        match analysis.name.as_ref() {
            "Component" => {
                if let Some(component) = NgComponentAnalyzer::analyze(
                    &analysis,
                    &self.file_path,
                    &self.project_root,
                    class_name,
                    &self.package_name,
                    self.imports.clone(),
                    self.default_standalone,
                    inputs,
                    outputs,
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
                    self.default_standalone,
                    inputs,
                    outputs,
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
                    self.default_standalone,
                ) {
                    self.results.pipes.push(pipe);
                }
            }
            _ => {}
        }
    }

    /// `@Input()`/`@Output()` decorated properties and signal-based
    /// `input()`/`output()`/`model()` (including `input.required()`).
    fn extract_inputs_outputs(class: &Class) -> (Vec<String>, Vec<String>) {
        let mut inputs = Vec::new();
        let mut outputs = Vec::new();

        for member in &class.body {
            let ClassMember::ClassProp(prop) = member else {
                continue;
            };
            let PropName::Ident(key) = &prop.key else {
                continue;
            };
            let name = key.sym.to_string();

            for decorator in &prop.decorators {
                let ident = match &*decorator.expr {
                    Expr::Call(call) => match &call.callee {
                        Callee::Expr(callee) => match &**callee {
                            Expr::Ident(ident) => Some(ident.sym.to_string()),
                            _ => None,
                        },
                        _ => None,
                    },
                    Expr::Ident(ident) => Some(ident.sym.to_string()),
                    _ => None,
                };
                match ident.as_deref() {
                    Some("Input") => inputs.push(name.clone()),
                    Some("Output") => outputs.push(name.clone()),
                    _ => {}
                }
            }

            if let Some(value) = &prop.value {
                if let Expr::Call(call) = &**value {
                    if let Callee::Expr(callee) = &call.callee {
                        let base_fn = match &**callee {
                            Expr::Ident(ident) => Some(ident.sym.to_string()),
                            // input.required(...)
                            Expr::Member(member) => match (&*member.obj, &member.prop) {
                                (Expr::Ident(obj), MemberProp::Ident(_)) => {
                                    Some(obj.sym.to_string())
                                }
                                _ => None,
                            },
                            _ => None,
                        };
                        match base_fn.as_deref() {
                            Some("input") | Some("model") => inputs.push(name.clone()),
                            Some("output") => outputs.push(name.clone()),
                            _ => {}
                        }
                    }
                }
            }
        }

        inputs.dedup();
        outputs.dedup();
        (inputs, outputs)
    }

    /// Resolves a specifier and records it when resolution fails. A dropped
    /// edge is invisible in the graph, and an invisible edge is exactly how a
    /// live symbol ends up on the `unused` list — so every failure is kept.
    fn resolve_or_record(&mut self, src: &str, name: &str) -> Option<ResolvedImport> {
        let resolved =
            self.import_resolver
                .resolve_import(src, name, &self.file_path, &self.ts_config);
        if resolved.is_none() {
            self.record_unresolved(src);
        }
        resolved
    }

    fn record_unresolved(&mut self, src: &str) {
        let empty = std::collections::HashMap::new();
        let ts_paths = self
            .ts_config
            .compiler_options
            .as_ref()
            .and_then(|options| options.paths.as_ref())
            .unwrap_or(&empty);
        let scope = classify_unresolved(src, ts_paths);

        if !self
            .unresolved_imports
            .iter()
            .any(|unresolved| unresolved.specifier == src)
        {
            self.unresolved_imports.push(UnresolvedImport {
                specifier: src.to_string(),
                scope,
            });
        }
    }

    fn process_import_decl(&mut self, import_decl: &ImportDecl) {
        let src = import_decl.src.value.to_string();

        // `import './polyfills'` — no specifiers. It still executes the
        // module, so it is a real edge: without it the target looks orphaned.
        if import_decl.specifiers.is_empty() {
            if let Some(mut resolved) = self.resolve_or_record(&src, "*") {
                resolved.imported_item = ImportedItem {
                    name: String::new(),
                    alias: None,
                    import_kind: ImportKind::SideEffect,
                };
                self.imports.push(resolved);
            }
            return;
        }

        for specifier in &import_decl.specifiers {
            self.process_import_specifier(specifier, &src);
        }
    }

    fn process_import_specifier(&mut self, specifier: &swc_ecma_ast::ImportSpecifier, src: &str) {
        let imported_item = match specifier {
            swc_ecma_ast::ImportSpecifier::Named(named) => ImportedItem {
                name: named.local.sym.to_string(),
                alias: named.imported.as_ref().map(|imported| match imported {
                    ModuleExportName::Ident(ident) => ident.sym.to_string(),
                    ModuleExportName::Str(str) => str.value.to_string(),
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

        // For renamed imports (`import { A as B }`) the exporting file
        // declares the original name, not the local alias.
        let exported_name = imported_item
            .alias
            .as_deref()
            .unwrap_or(&imported_item.name)
            .to_string();

        if let Some(mut resolved_import) = self.resolve_or_record(src, &exported_name) {
            resolved_import.imported_item = imported_item;
            self.imports.push(resolved_import);
        }
    }

    fn export_name_to_string(name: &ModuleExportName) -> String {
        match name {
            ModuleExportName::Ident(ident) => ident.sym.to_string(),
            ModuleExportName::Str(str) => str.value.to_string(),
        }
    }

    /// First-pass collection of everything the module exports.
    fn collect_exports(&mut self, module: &Module) {
        for item in &module.body {
            let ModuleItem::ModuleDecl(decl) = item else {
                continue;
            };
            match decl {
                ModuleDecl::ExportDecl(export_decl) => match &export_decl.decl {
                    Decl::Class(class_decl) => {
                        self.push_export(class_decl.ident.sym.to_string(), ExportKind::Class)
                    }
                    Decl::Fn(fn_decl) => {
                        self.push_export(fn_decl.ident.sym.to_string(), ExportKind::Function)
                    }
                    Decl::Var(var_decl) => {
                        for var in &var_decl.decls {
                            if let swc_ecma_ast::Pat::Ident(ident) = &var.name {
                                self.push_export(ident.id.sym.to_string(), ExportKind::Variable);
                            }
                        }
                    }
                    Decl::TsInterface(interface) => {
                        self.push_export(interface.id.sym.to_string(), ExportKind::Interface)
                    }
                    Decl::TsTypeAlias(alias) => {
                        self.push_export(alias.id.sym.to_string(), ExportKind::TypeAlias)
                    }
                    Decl::TsEnum(ts_enum) => {
                        self.push_export(ts_enum.id.sym.to_string(), ExportKind::Enum)
                    }
                    _ => {}
                },
                ModuleDecl::ExportNamed(named) => {
                    for specifier in &named.specifiers {
                        match specifier {
                            swc_ecma_ast::ExportSpecifier::Named(spec) => {
                                let exported = spec
                                    .exported
                                    .as_ref()
                                    .map(Self::export_name_to_string)
                                    .unwrap_or_else(|| Self::export_name_to_string(&spec.orig));

                                // `export { X } from './y'` also creates a
                                // dependency edge on the source module.
                                match &named.src {
                                    Some(src) => {
                                        let source = src.value.to_string();
                                        self.push_reexport(exported, ExportKind::ReExport, &source);
                                        let original = Self::export_name_to_string(&spec.orig);
                                        self.resolve_or_record(&source, &original);
                                    }
                                    // `class X {}; export { X }` — declared here.
                                    None => self.push_export(exported, ExportKind::ReExport),
                                }
                            }
                            swc_ecma_ast::ExportSpecifier::Namespace(spec) => {
                                let exported = Self::export_name_to_string(&spec.name);
                                match &named.src {
                                    Some(src) => {
                                        let source = src.value.to_string();
                                        self.push_reexport(exported, ExportKind::ReExport, &source);
                                        self.resolve_or_record(&source, "*");
                                    }
                                    None => self.push_export(exported, ExportKind::ReExport),
                                }
                            }
                            swc_ecma_ast::ExportSpecifier::Default(_) => {
                                self.push_export("default".to_string(), ExportKind::Default);
                            }
                        }
                    }
                }
                ModuleDecl::ExportAll(export_all) => {
                    let source = export_all.src.value.to_string();
                    self.push_reexport(
                        format!("* from {}", source),
                        ExportKind::ReExportAll,
                        &source,
                    );
                    self.resolve_or_record(&source, "*");
                }
                ModuleDecl::ExportDefaultDecl(default_decl) => {
                    let name = match &default_decl.decl {
                        DefaultDecl::Class(class_expr) => {
                            class_expr.ident.as_ref().map(|ident| ident.sym.to_string())
                        }
                        DefaultDecl::Fn(fn_expr) => {
                            fn_expr.ident.as_ref().map(|ident| ident.sym.to_string())
                        }
                        _ => None,
                    };
                    self.push_export(
                        name.unwrap_or_else(|| "default".to_string()),
                        ExportKind::Default,
                    );
                }
                ModuleDecl::ExportDefaultExpr(_) => {
                    self.push_export("default".to_string(), ExportKind::Default);
                }
                _ => {}
            }
        }
    }

    fn push_export(&mut self, name: String, kind: ExportKind) {
        self.exports.push(ExportInfo {
            name,
            kind,
            from_module: None,
        });
    }

    /// `export { X } from './y'` / `export * from './y'` — carries the source,
    /// which is what separates a pass-through barrel from a file that declares
    /// its own symbols and merely exports them in a separate statement.
    fn push_reexport(&mut self, name: String, kind: ExportKind, from_module: &str) {
        self.exports.push(ExportInfo {
            name,
            kind,
            from_module: Some(from_module.to_string()),
        });
    }

    /// React function components: capitalized top-level functions or consts
    /// with an arrow/function initializer, optionally wrapped in
    /// `memo(...)` / `forwardRef(...)`. Only in .tsx/.jsx files.
    fn collect_react_components(&mut self, module: &Module) {
        let is_capitalized =
            |name: &str| name.chars().next().is_some_and(|c| c.is_ascii_uppercase());

        let push = |name: String, wrapped: bool, this: &mut Self| {
            this.results.react_components.push(ReactComponentInfo {
                name,
                source_path: this.file_path.clone(),
                package_name: this.package_name.clone(),
                wrapped,
            });
        };

        for item in &module.body {
            let decl = match item {
                ModuleItem::Stmt(Stmt::Decl(decl)) => decl,
                ModuleItem::ModuleDecl(ModuleDecl::ExportDecl(export)) => &export.decl,
                ModuleItem::ModuleDecl(ModuleDecl::ExportDefaultDecl(default)) => {
                    if let DefaultDecl::Fn(fn_expr) = &default.decl {
                        if let Some(ident) = &fn_expr.ident {
                            let name = ident.sym.to_string();
                            if is_capitalized(&name) {
                                push(name, false, self);
                            }
                        }
                    }
                    continue;
                }
                _ => continue,
            };

            match decl {
                Decl::Fn(fn_decl) => {
                    let name = fn_decl.ident.sym.to_string();
                    if is_capitalized(&name) {
                        push(name, false, self);
                    }
                }
                Decl::Var(var_decl) => {
                    for var in &var_decl.decls {
                        let swc_ecma_ast::Pat::Ident(ident) = &var.name else {
                            continue;
                        };
                        let name = ident.id.sym.to_string();
                        if !is_capitalized(&name) {
                            continue;
                        }
                        let Some(init) = &var.init else { continue };
                        match &**init {
                            Expr::Arrow(_) | Expr::Fn(_) => push(name, false, self),
                            Expr::Call(call) => {
                                let wrapper = match &call.callee {
                                    Callee::Expr(callee) => match &**callee {
                                        Expr::Ident(ident) => Some(ident.sym.to_string()),
                                        Expr::Member(member) => {
                                            match (&*member.obj, &member.prop) {
                                                // React.memo / React.forwardRef
                                                (Expr::Ident(_), MemberProp::Ident(prop)) => {
                                                    Some(prop.sym.to_string())
                                                }
                                                _ => None,
                                            }
                                        }
                                        _ => None,
                                    },
                                    _ => None,
                                };
                                if matches!(wrapper.as_deref(), Some("memo") | Some("forwardRef")) {
                                    push(name, true, self);
                                }
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }
    }

    /// Names a top-level item declares (empty for statements/imports).
    /// A default export with an ident declares that ident.
    fn top_level_declared_names(item: &ModuleItem) -> Vec<String> {
        let decl = match item {
            ModuleItem::Stmt(Stmt::Decl(decl)) => decl,
            ModuleItem::ModuleDecl(ModuleDecl::ExportDecl(export)) => &export.decl,
            ModuleItem::ModuleDecl(ModuleDecl::ExportDefaultDecl(default)) => {
                let ident = match &default.decl {
                    DefaultDecl::Class(class) => class.ident.as_ref(),
                    DefaultDecl::Fn(function) => function.ident.as_ref(),
                    _ => None,
                };
                return vec![ident
                    .map(|i| i.sym.to_string())
                    .unwrap_or_else(|| "default".to_string())];
            }
            ModuleItem::ModuleDecl(ModuleDecl::ExportDefaultExpr(_)) => {
                return vec!["default".to_string()];
            }
            _ => return Vec::new(),
        };
        match decl {
            Decl::Class(class) => vec![class.ident.sym.to_string()],
            Decl::Fn(function) => vec![function.ident.sym.to_string()],
            Decl::Var(var) => var
                .decls
                .iter()
                .filter_map(|d| match &d.name {
                    swc_ecma_ast::Pat::Ident(ident) => Some(ident.id.sym.to_string()),
                    _ => None,
                })
                .collect(),
            Decl::TsInterface(interface) => vec![interface.id.sym.to_string()],
            Decl::TsTypeAlias(alias) => vec![alias.id.sym.to_string()],
            Decl::TsEnum(ts_enum) => vec![ts_enum.id.sym.to_string()],
            _ => Vec::new(),
        }
    }

    /// Same-file references between top-level declarations: for every item,
    /// which OTHER top-level names its subtree mentions (value or type
    /// position — a union type referencing an interface counts). Top-level
    /// statements aggregate under `""`; they run whenever the module loads.
    fn collect_local_references(&mut self, module: &Module) {
        #[derive(Default)]
        struct IdentCollector {
            idents: HashSet<String>,
        }
        impl Visit for IdentCollector {
            fn visit_ident(&mut self, ident: &swc_ecma_ast::Ident) {
                self.idents.insert(ident.sym.to_string());
            }
        }

        let top_names: HashSet<String> = module
            .body
            .iter()
            .flat_map(Self::top_level_declared_names)
            .collect();
        if top_names.is_empty() {
            return;
        }

        let mut statement_refs: Vec<String> = Vec::new();
        for item in &module.body {
            // Import/export-from statements reference nothing local; a bare
            // `export { A }` is the export itself, not a usage of A.
            if matches!(
                item,
                ModuleItem::ModuleDecl(
                    ModuleDecl::Import(_) | ModuleDecl::ExportNamed(_) | ModuleDecl::ExportAll(_)
                )
            ) {
                continue;
            }

            let declared = Self::top_level_declared_names(item);
            let mut collector = IdentCollector::default();
            item.visit_with(&mut collector);
            let mut refs: Vec<String> = collector
                .idents
                .into_iter()
                .filter(|name| top_names.contains(name) && !declared.contains(name))
                .collect();
            refs.sort();
            if refs.is_empty() {
                continue;
            }

            if declared.is_empty() {
                statement_refs.extend(refs);
            } else {
                for from in declared {
                    self.local_references.push(LocalReference {
                        from,
                        to: refs.clone(),
                    });
                }
            }
        }

        if !statement_refs.is_empty() {
            statement_refs.sort();
            statement_refs.dedup();
            self.local_references.push(LocalReference {
                from: String::new(),
                to: statement_refs,
            });
        }
    }

    fn assemble_file_facts(&mut self) {
        let mut used_import_names: Vec<String> = self
            .imports
            .iter()
            .map(|import| import.imported_item.name.clone())
            .filter(|name| self.used_idents.contains(name))
            .collect();
        used_import_names.sort();
        used_import_names.dedup();

        let mut unresolved_imports = std::mem::take(&mut self.unresolved_imports);
        unresolved_imports.sort();

        self.results.source_files.push(FileFactsInfo {
            path: self.file_path.clone(),
            package_name: self.package_name.clone(),
            exports: std::mem::take(&mut self.exports),
            imports: self.imports.clone(),
            dynamic_imports: std::mem::take(&mut self.dynamic_imports),
            used_import_names,
            unresolved_imports,
            jsx_usages: std::mem::take(&mut self.jsx_usages),
            local_references: std::mem::take(&mut self.local_references),
        });
    }
}

impl<'a> Visit for AngularVisitor<'a> {
    fn visit_module(&mut self, module: &Module) {
        // Imports first: decorated classes below reference them.
        for item in &module.body {
            if let ModuleItem::ModuleDecl(ModuleDecl::Import(import_decl)) = item {
                self.process_import_decl(import_decl);
            }
        }

        self.collect_exports(module);
        self.collect_local_references(module);

        if self.is_jsx_file {
            self.collect_react_components(module);
        }

        // Walk the whole module: classes in every position, dynamic imports,
        // identifier usage, JSX elements.
        module.visit_children_with(self);

        self.assemble_file_facts();
    }

    /// Import declarations were processed manually — skipping them here keeps
    /// their local names out of `used_idents`.
    fn visit_import_decl(&mut self, _import_decl: &ImportDecl) {}

    fn visit_class_decl(&mut self, class_decl: &ClassDecl) {
        let class_name = class_decl.ident.sym.to_string();
        self.process_decorated_class(&class_decl.class, &class_name);
        class_decl.visit_children_with(self);
    }

    fn visit_export_default_decl(&mut self, export_default: &ExportDefaultDecl) {
        if let DefaultDecl::Class(class_expr) = &export_default.decl {
            let class_name = class_expr
                .ident
                .as_ref()
                .map(|ident| ident.sym.to_string())
                .unwrap_or_else(|| "default".to_string());
            self.process_decorated_class(&class_expr.class, &class_name);
        }
        export_default.visit_children_with(self);
    }

    fn visit_call_expr(&mut self, call: &CallExpr) {
        // Dynamic `import('...')` — lazy loading edge (Angular routes,
        // React.lazy).
        if matches!(call.callee, Callee::Import(_)) {
            if let Some(arg) = call.args.first() {
                if let Expr::Lit(Lit::Str(src)) = &*arg.expr {
                    if let Some(mut resolved) = self.resolve_or_record(&src.value, "*") {
                        resolved.imported_item = ImportedItem {
                            name: "*".to_string(),
                            alias: None,
                            import_kind: ImportKind::Namespace,
                        };
                        self.dynamic_imports.push(resolved);
                    }
                }
            }
        }
        call.visit_children_with(self);
    }

    fn visit_ident(&mut self, ident: &swc_ecma_ast::Ident) {
        self.used_idents.insert(ident.sym.to_string());
    }

    fn visit_jsx_opening_element(&mut self, element: &JSXOpeningElement) {
        if let JSXElementName::Ident(ident) = &element.name {
            let name = ident.sym.to_string();
            // Capitalized tags are components; lowercase are DOM elements.
            if name.chars().next().is_some_and(|c| c.is_ascii_uppercase()) {
                let props = element
                    .attrs
                    .iter()
                    .filter_map(|attr| match attr {
                        JSXAttrOrSpread::JSXAttr(attr) => match &attr.name {
                            JSXAttrName::Ident(ident) => Some(ident.sym.to_string()),
                            _ => None,
                        },
                        _ => None,
                    })
                    .collect();
                self.jsx_usages.push(JsxUsageInfo {
                    component: name,
                    props,
                });
            }
        }
        element.visit_children_with(self);
    }
}
