use crate::analyses::project_map::{is_test_file, ProjectCatalog};
use crate::report::FullReport;
use serde::Serialize;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// Full usage picture of a single symbol: every place it is referenced,
/// classified by mechanism — the "who uses this component and how" question.
#[derive(Debug, Serialize)]
pub struct SymbolUsageReport {
    pub symbol: String,
    pub declarations: Vec<SymbolDeclaration>,
}

#[derive(Debug, Serialize)]
pub struct SymbolDeclaration {
    pub file: PathBuf,
    pub project: String,
    pub kind: String,
    pub total_usages: usize,
    pub usages: Vec<SymbolUsage>,
    /// Usage count per using project.
    pub by_project: BTreeMap<String, usize>,
}

#[derive(Debug, Serialize)]
pub struct SymbolUsage {
    pub file: PathBuf,
    pub project: String,
    pub via: UsageVia,
    pub test: bool,
}

#[derive(Debug, Serialize, PartialEq, Eq, Clone)]
pub enum UsageVia {
    Import,
    Template,
    Jsx,
    LazyLoad,
}

/// Builds the usage report for `symbol`. `from_project` narrows the usages to
/// those originating in one project.
pub fn symbol_usages(
    report: &FullReport,
    catalog: &ProjectCatalog,
    symbol: &str,
    from_project: Option<&str>,
) -> SymbolUsageReport {
    let project_of = |path: &Path| {
        catalog
            .project_of(path)
            .map(|p| p.name.clone())
            .unwrap_or_default()
    };

    // Declarations: exported symbols with this name + Angular/React entities.
    let mut declarations: BTreeMap<PathBuf, String> = BTreeMap::new();
    for file in &report.results.source_files {
        for export in &file.exports {
            if export.name == symbol {
                declarations
                    .entry(file.path.clone())
                    .or_insert_with(|| format!("{:?}", export.kind));
            }
        }
    }
    for component in &report.results.components {
        if component.base.name == symbol {
            declarations.insert(component.base.source_path.clone(), "Component".into());
        }
    }
    for directive in &report.results.directives {
        if directive.base.name == symbol {
            declarations.insert(directive.base.source_path.clone(), "Directive".into());
        }
    }
    for pipe in &report.results.pipes {
        if pipe.base.name == symbol {
            declarations.insert(pipe.base.source_path.clone(), "Pipe".into());
        }
    }
    for service in &report.results.services {
        if service.base.name == symbol {
            declarations.insert(service.base.source_path.clone(), "Service".into());
        }
    }
    for component in &report.results.react_components {
        if component.name == symbol {
            declarations.insert(component.source_path.clone(), "ReactComponent".into());
        }
    }

    let mut result = Vec::new();

    for (decl_file, kind) in declarations {
        let mut usages: Vec<SymbolUsage> = Vec::new();

        for file in &report.results.source_files {
            // Static imports of this symbol resolved to the declaring file.
            for import in &file.imports {
                let exported = import
                    .imported_item
                    .alias
                    .as_deref()
                    .unwrap_or(&import.imported_item.name);
                if exported == symbol && import.resolved_path == decl_file {
                    usages.push(SymbolUsage {
                        file: file.path.clone(),
                        project: project_of(&file.path),
                        via: UsageVia::Import,
                        test: is_test_file(&file.path),
                    });
                }
            }
            // Lazy loads of the declaring file itself.
            for import in &file.dynamic_imports {
                if import.resolved_path == decl_file {
                    usages.push(SymbolUsage {
                        file: file.path.clone(),
                        project: project_of(&file.path),
                        via: UsageVia::LazyLoad,
                        test: is_test_file(&file.path),
                    });
                }
            }
            // JSX renders, resolved through this file's imports.
            for jsx in &file.jsx_usages {
                if jsx.component == symbol {
                    let resolved_here = file.imports.iter().any(|import| {
                        import.imported_item.name == symbol && import.resolved_path == decl_file
                    }) || file.path == decl_file;
                    if resolved_here {
                        usages.push(SymbolUsage {
                            file: file.path.clone(),
                            project: project_of(&file.path),
                            via: UsageVia::Jsx,
                            test: is_test_file(&file.path),
                        });
                    }
                }
            }
        }

        // Angular template renders.
        for usage in &report.template_usages {
            if usage.target == symbol && usage.target_path == decl_file {
                usages.push(SymbolUsage {
                    file: usage.component_path.clone(),
                    project: project_of(&usage.component_path),
                    via: UsageVia::Template,
                    test: false,
                });
            }
        }

        if let Some(from) = from_project {
            usages.retain(|usage| usage.project == from);
        }
        usages.sort_by(|a, b| (&a.file, a.via.clone() as u8).cmp(&(&b.file, b.via.clone() as u8)));

        let mut by_project: BTreeMap<String, usize> = BTreeMap::new();
        for usage in &usages {
            *by_project.entry(usage.project.clone()).or_insert(0) += 1;
        }

        result.push(SymbolDeclaration {
            project: project_of(&decl_file),
            file: decl_file,
            kind,
            total_usages: usages.len(),
            by_project,
            usages,
        });
    }

    SymbolUsageReport {
        symbol: symbol.to_string(),
        declarations: result,
    }
}

pub fn print_symbol_usages(report: &SymbolUsageReport) {
    if report.declarations.is_empty() {
        println!("❓ Symbol `{}` not found in the workspace.", report.symbol);
        return;
    }

    for declaration in &report.declarations {
        println!(
            "🔎 {} [{}] — declared in {} ({})",
            report.symbol,
            declaration.kind,
            declaration.file.display(),
            declaration.project
        );
        println!("   Total usages: {}", declaration.total_usages);

        if !declaration.by_project.is_empty() {
            println!("   By project:");
            for (project, count) in &declaration.by_project {
                println!("     {} ×{}", project, count);
            }
        }

        if declaration.usages.is_empty() {
            println!("   ⚠️ No usages found — candidate for removal.");
        } else {
            println!("   Usages:");
            for usage in &declaration.usages {
                let via = match usage.via {
                    UsageVia::Import => "import",
                    UsageVia::Template => "template",
                    UsageVia::Jsx => "jsx",
                    UsageVia::LazyLoad => "lazy",
                };
                let test = if usage.test { " [test]" } else { "" };
                println!("     [{}]{} {}", via, test, usage.file.display());
            }
        }
        println!();
    }
}
