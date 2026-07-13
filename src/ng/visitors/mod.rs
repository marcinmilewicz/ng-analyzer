use crate::ng::models::NgAnalysisResults;
use swc_ecma_visit::VisitWith;

use crate::analysis::models::ts_config::TSConfig;
use crate::ng::visitors::visitor::AngularVisitor;

use crate::analysis::resolvers::import_resolver::ImportResolver;
use crate::file_cache_reader::CachedFileReader;
use std::path::Path;
use std::sync::Arc;
use swc_common::input::StringInput;
use swc_common::SourceMap;
use swc_ecma_parser::lexer::Lexer;
use swc_ecma_parser::{Parser, Syntax, TsSyntax};

mod visitor;

/// JSX syntax for .tsx/.jsx files, plain TS otherwise.
pub fn syntax_for(path: &Path) -> Syntax {
    let tsx = path
        .extension()
        .map(|ext| ext == "tsx" || ext == "jsx")
        .unwrap_or(false);
    Syntax::Typescript(TsSyntax {
        tsx,
        decorators: true,
        ..Default::default()
    })
}

#[allow(clippy::too_many_arguments)]
pub fn analyze_file(
    path: &Path,
    project_root: &Path,
    source_map: &SourceMap,
    project_name: String,
    tsconfig: TSConfig,
    default_standalone: bool,
    import_resolver: &mut ImportResolver,
    file_reader: &CachedFileReader,
) -> Result<NgAnalysisResults, Box<dyn std::error::Error>> {
    let source = file_reader.read_file(path)?;
    let source_file = source_map.new_source_file(
        Arc::from(swc_common::FileName::Real(path.to_path_buf())),
        source,
    );

    let lexer = Lexer::new(
        syntax_for(path),
        Default::default(),
        StringInput::from(&*source_file),
        None,
    );

    let mut parser = Parser::new_from(lexer);
    match parser.parse_module() {
        Ok(module) => {
            let mut visitor = AngularVisitor::new(
                path,
                project_root,
                project_name,
                tsconfig.clone(),
                default_standalone,
                import_resolver,
            );
            module.visit_with(&mut visitor);

            Ok(visitor.results)
        }
        Err(e) => {
            // Keep the analysis resilient, but never swallow errors silently.
            eprintln!("⚠️ Failed to parse {:?}: {:?}", path, e);
            Ok(NgAnalysisResults::default())
        }
    }
}
