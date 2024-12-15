use crate::ng::models::NgAnalysisResults;
use swc_ecma_visit::VisitWith;

use crate::analysis::models::ts_config::TSConfig;
use crate::ng::visitors::visitor::AngularVisitor;

use crate::analysis::resolvers::import_resolver::ImportResolver;
use crate::file_cache_reader::CachedFileReader;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use swc_common::input::StringInput;
use swc_common::SourceMap;
use swc_ecma_parser::lexer::Lexer;
use swc_ecma_parser::{Parser, Syntax, TsSyntax};

mod visitor;

pub fn analyze_file(
    file_path: &Path,
    base_path: PathBuf,
    source_map: &SourceMap,
    project_name: String,
    tsconfig: TSConfig,
    import_resolver: &mut ImportResolver,
    file_reader: &CachedFileReader,
) -> Result<NgAnalysisResults, Box<dyn std::error::Error>> {
    let source = file_reader.read_file(file_path)?;
    let source_file = source_map.new_source_file(
        Arc::from(swc_common::FileName::Real(file_path.to_path_buf())),
        source,
    );

    let lexer = Lexer::new(
        Syntax::Typescript(TsSyntax {
            tsx: false,
            decorators: true,
            ..Default::default()
        }),
        Default::default(),
        StringInput::from(&*source_file),
        None,
    );

    let mut parser = Parser::new_from(lexer);
    match parser.parse_module() {
        Ok(module) => {
            let mut visitor = AngularVisitor::new(
                file_path,
                base_path,
                project_name,
                tsconfig.clone(),
                import_resolver,
                file_reader,
            );
            module.visit_with(&mut visitor);

            visitor.results.build_maps();

            Ok(visitor.results)
        }
        Err(_) => Ok(NgAnalysisResults::default()),
    }
}
