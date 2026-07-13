use clap::{Parser, Subcommand, ValueEnum};
use std::path::{Path, PathBuf};
use std::time::Instant;
mod analyses;
mod analysis;
mod file_cache_reader;
mod ng;
mod nx;
mod report;

use crate::analyses::project_map::{ProjectCatalog, ProjectInfo};
use crate::analysis::processor::file_processor::{ProjectProcessor, SharedAnalysisState};
use crate::ng::ng_reporter::NgReporter;
use crate::nx::nx_project::NxProject;
use crate::nx::NxWorkspace;
use crate::report::FullReport;
use analysis::timing::TimingMetrics;
use ng::models::NgAnalysisResults;
use std::sync::Arc;
use swc_common::SourceMap;

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "Static analyzer for TypeScript NX workspaces (Angular, React, Next.js)"
)]
struct Args {
    /// Path to the workspace directory
    #[arg(short = 'd', long, default_value = ".", global = true)]
    project_path: PathBuf,

    /// Enable verbose output
    #[arg(short = 'v', long, global = true)]
    verbose: bool,

    /// Filter specific projects (comma-separated)
    #[arg(short = 'p', long, global = true)]
    projects: Option<String>,

    /// Exclude node_modules (disable with --exclude-node-modules false)
    #[arg(short = 'n', long, default_value_t = true, action = clap::ArgAction::Set, global = true)]
    exclude_node_modules: bool,

    /// Analyze TypeScript files only (disable with --typescript-only false)
    #[arg(short = 't', long, default_value_t = true, action = clap::ArgAction::Set, global = true)]
    typescript_only: bool,

    /// Baseline file — only findings NOT in the baseline are reported/failed on
    #[arg(long, global = true)]
    baseline: Option<PathBuf>,

    /// Exit 3 when any import inside the workspace fails to resolve. Each one
    /// is an edge missing from the graph, so dead-code findings cannot be
    /// trusted until they are zero — gate CI on this before --fail-on unused.
    #[arg(long, global = true)]
    strict: bool,

    /// Fail (exit 2) when any new finding exists. Repeat or comma-separate:
    /// unused, cycles, boundaries, all
    #[arg(long, global = true, value_delimiter = ',')]
    fail_on: Vec<FailCategory>,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Full analysis, JSON report to a file (default command)
    Analyze {
        /// Output file for the JSON report
        #[arg(short, long, default_value = "angular-analysis.json")]
        output_file: PathBuf,
    },
    /// Package statistics: coupling, dependency matrix, project cycles
    Stats {
        /// Show only rows involving this project
        #[arg(long)]
        project: Option<String>,
    },
    /// Unused exports, never-rendered entities, orphan files
    Unused {
        /// Show only findings in this project
        #[arg(long)]
        project: Option<String>,
        /// Show only these kinds (component, service, pipe, directive,
        /// class, function, variable, interface, enum) — comma-separated
        #[arg(long, value_delimiter = ',')]
        kind: Vec<String>,
    },
    /// File-level and project-level dependency cycles
    Cycles,
    /// Symbols worth moving to the only project that uses them
    MoveCandidates {
        /// Show only candidates from or into this project
        #[arg(long)]
        project: Option<String>,
    },
    /// Where and how a symbol/component is used (imports, templates, JSX, lazy)
    Usages {
        /// Symbol name, e.g. UiButtonComponent or formatPrice
        symbol: String,
        /// Count only usages originating in this project
        #[arg(long)]
        from: Option<String>,
        /// Print JSON instead of the human-readable summary
        #[arg(long)]
        json: bool,
    },
    /// NX tag boundary rule violations (rules in nx-analyzer.json)
    Boundaries,
    /// Export the dependency graph
    Graph {
        #[arg(long, value_enum, default_value_t = GraphFormat::Mermaid)]
        format: GraphFormat,
        #[arg(long, value_enum, default_value_t = GraphLevel::Project)]
        level: GraphLevel,
    },
    /// Self-contained interactive HTML report
    Html {
        #[arg(short, long, default_value = "nx-analyzer-report.html")]
        output_file: PathBuf,
    },
    /// SARIF 2.1.0 output (GitHub code scanning)
    Sarif {
        #[arg(short, long, default_value = "nx-analyzer.sarif")]
        output_file: PathBuf,
    },
    /// Write the current findings as a baseline file
    Baseline {
        #[arg(short, long, default_value = "nx-analyzer-baseline.json")]
        output_file: PathBuf,
    },
}

#[derive(ValueEnum, Clone, Debug, PartialEq)]
enum FailCategory {
    Unused,
    Cycles,
    Boundaries,
    All,
}

#[derive(ValueEnum, Clone, Debug)]
enum GraphFormat {
    Mermaid,
    Dot,
    Json,
}

#[derive(ValueEnum, Clone, Debug)]
enum GraphLevel {
    Project,
    File,
}

/// Angular >= 19 treats components without an explicit `standalone:` flag as
/// standalone. Detect the major version from the workspace package.json.
fn detect_default_standalone(workspace_root: &Path) -> bool {
    let package_json = workspace_root.join("package.json");
    let Ok(content) = std::fs::read_to_string(&package_json) else {
        return false;
    };
    let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) else {
        return false;
    };

    let version = ["dependencies", "devDependencies"]
        .iter()
        .find_map(|section| json[section]["@angular/core"].as_str());

    let Some(version) = version else {
        return false;
    };

    let major: Option<u32> = version
        .chars()
        .skip_while(|c| !c.is_ascii_digit())
        .take_while(|c| c.is_ascii_digit())
        .collect::<String>()
        .parse()
        .ok();

    major.is_some_and(|major| major >= 19)
}

/// expo-router uses file-based routing: EVERY script file under the app
/// directory is a route consumed by the framework. Detected via the
/// project-level package.json.
fn detect_file_routing_dirs(project_root: &Path) -> Vec<PathBuf> {
    let package_json = project_root.join("package.json");
    let Ok(content) = std::fs::read_to_string(&package_json) else {
        return Vec::new();
    };
    if !content.contains("\"expo-router\"") {
        return Vec::new();
    }

    ["src/app", "app"]
        .iter()
        .map(|dir| project_root.join(dir))
        .filter(|dir| dir.is_dir())
        .collect()
}

/// Runs the whole pipeline: workspace discovery → per-project extraction →
/// template analysis → derived analyses.
fn run_pipeline(args: &Args) -> Result<(FullReport, ProjectCatalog), Box<dyn std::error::Error>> {
    let total_start = Instant::now();
    let mut metrics = TimingMetrics::new();

    let source_map = Arc::new(SourceMap::default());
    let project_path = Path::new(&args.project_path);

    let mut results = NgAnalysisResults::default();

    let workspace_start = Instant::now();
    let mut nx_workspace = NxWorkspace::new(project_path.to_path_buf());
    nx_workspace.load_configuration()?;
    metrics.workspace_load_time = workspace_start.elapsed();

    let shared = SharedAnalysisState::new();
    let workspace_root = nx_workspace.workspace_root().to_path_buf();
    let default_standalone = detect_default_standalone(&workspace_root);

    let projects: Vec<(PathBuf, NxProject)> = if let Some(project_filter) = &args.projects {
        let project_names: Vec<&str> = project_filter.split(',').collect();
        nx_workspace
            .get_projects()
            .iter()
            .filter(|(_, project)| project_names.contains(&project.name.as_str()))
            .map(|(path, project)| (path.clone(), project.clone()))
            .collect()
    } else {
        nx_workspace
            .get_projects()
            .iter()
            .map(|(path, project)| (path.clone(), project.clone()))
            .collect()
    };

    for (project_path, project) in &projects {
        let project_start = Instant::now();

        // Other projects rooted inside this one own their files.
        let nested_roots: Vec<PathBuf> = projects
            .iter()
            .map(|(other_root, _)| other_root)
            .filter(|other_root| {
                *other_root != project_path && other_root.starts_with(project_path)
            })
            .cloned()
            .collect();

        let mut processor = ProjectProcessor::new(
            workspace_root.clone(),
            project_path.clone(),
            project.name.clone(),
            project.ts_config.clone(),
            shared.clone(),
            Arc::clone(&source_map),
            default_standalone,
        )
        .exclude_nested_roots(nested_roots);

        if args.exclude_node_modules {
            processor = processor.filter_node_modules();
        }

        processor = if args.typescript_only {
            processor.filter_ts_files()
        } else {
            processor.filter_script_files()
        };

        processor.process_files(&mut results);
        metrics
            .file_analysis_times
            .push((project.name.clone(), project_start.elapsed()));

        if args.verbose {
            println!("📦 Project {} has been processed", project.name);
        }
    }

    metrics.total_time = total_start.elapsed();

    results.sort_deterministic();

    // Template usages add edges to the import graph — the snapshot must be
    // taken afterwards.
    let template_usages = ng::templates::analyze_templates(&results, &shared.import_graph);

    let catalog = ProjectCatalog::new(
        projects
            .iter()
            .map(|(root, project)| ProjectInfo {
                name: project.name.clone(),
                root: root.clone(),
                tags: project.config.tags.clone().unwrap_or_default(),
                project_type: project
                    .config
                    .project_type
                    .clone()
                    .unwrap_or_else(|| "library".to_string()),
                entry_dirs: detect_file_routing_dirs(root),
            })
            .collect(),
    );

    let analysis = analyses::run_analyses(
        &results,
        &template_usages,
        &shared.import_graph,
        &catalog,
        &workspace_root,
    );

    if args.verbose {
        metrics.print_summary();
        println!("{} projects have been processed", projects.len());
    }

    Ok((
        FullReport {
            results,
            template_usages,
            import_graph: shared.import_graph.snapshot(),
            analysis,
        },
        catalog,
    ))
}

/// Applies the baseline and --fail-on policy. Returns the process exit code.
fn enforce_policy(
    args: &Args,
    full_report: &FullReport,
) -> Result<i32, Box<dyn std::error::Error>> {
    // An incomplete graph invalidates every finding below it, so this gate
    // comes first and has its own exit code.
    let resolution = &full_report.analysis.resolution;
    if args.strict && !resolution.is_trustworthy() {
        eprintln!(
            "\n❌ --strict: {} unresolved import(s) inside the workspace.",
            resolution.unresolved_internal.len()
        );
        return Ok(3);
    }

    let new_findings = match &args.baseline {
        Some(path) => {
            let baseline = report::baseline::load(path)?;
            report::baseline::new_findings(full_report, &baseline)
        }
        None => report::baseline::finding_keys(full_report),
    };

    if args.baseline.is_some() && !new_findings.is_empty() {
        println!("\n🆕 New findings vs baseline ({}):", new_findings.len());
        for key in &new_findings {
            println!("  {}", key);
        }
    }

    if args.fail_on.is_empty() {
        return Ok(0);
    }

    let matches_category = |key: &str| {
        args.fail_on.iter().any(|category| match category {
            FailCategory::All => true,
            FailCategory::Unused => {
                key.starts_with("unused:")
                    || key.starts_with("unused-import:")
                    || key.starts_with("not-rendered:")
                    || key.starts_with("orphan:")
            }
            FailCategory::Cycles => key.starts_with("cycle:") || key.starts_with("project-cycle:"),
            FailCategory::Boundaries => key.starts_with("boundary:"),
        })
    };

    let failing: Vec<&String> = new_findings
        .iter()
        .filter(|key| matches_category(key))
        .collect();
    if failing.is_empty() {
        Ok(0)
    } else {
        eprintln!("\n❌ --fail-on: {} finding(s):", failing.len());
        for key in failing {
            eprintln!("  {}", key);
        }
        Ok(2)
    }
}

fn run(args: &Args) -> Result<i32, Box<dyn std::error::Error>> {
    let (full_report, catalog) = run_pipeline(args)?;

    // Goes to stderr, so it never pollutes JSON on stdout. Silent when the
    // graph is complete.
    report::terminal::print_resolution_warning(&full_report);

    match args.command.as_ref() {
        None | Some(Command::Analyze { .. }) => {
            let output_file = match args.command.as_ref() {
                Some(Command::Analyze { output_file }) => output_file.clone(),
                _ => PathBuf::from("angular-analysis.json"),
            };
            if args.verbose {
                NgReporter::print_analysis(&full_report.results);
            }
            let json = serde_json::to_string_pretty(&full_report)?;
            std::fs::write(&output_file, json)?;
            println!("✅ Analysis results saved to {:?}", output_file);
        }
        Some(Command::Stats { project }) => {
            report::terminal::print_stats(&full_report, project.as_deref())
        }
        Some(Command::Unused { project, kind }) => {
            report::terminal::print_unused(&full_report, project.as_deref(), kind)
        }
        Some(Command::Cycles) => report::terminal::print_cycles(&full_report),
        Some(Command::MoveCandidates { project }) => {
            report::terminal::print_move_candidates(&full_report, project.as_deref())
        }
        Some(Command::Boundaries) => report::terminal::print_boundaries(&full_report),
        Some(Command::Usages { symbol, from, json }) => {
            let usage_report =
                report::usages::symbol_usages(&full_report, &catalog, symbol, from.as_deref());
            if *json {
                println!("{}", serde_json::to_string_pretty(&usage_report)?);
            } else {
                report::usages::print_symbol_usages(&usage_report);
            }
        }
        Some(Command::Graph { format, level }) => {
            let output = match (format, level) {
                (GraphFormat::Mermaid, GraphLevel::Project) => {
                    report::graph_export::project_graph_mermaid(&full_report)
                }
                (GraphFormat::Dot, GraphLevel::Project) => {
                    report::graph_export::project_graph_dot(&full_report)
                }
                (GraphFormat::Dot, GraphLevel::File) => {
                    report::graph_export::file_graph_dot(&full_report)
                }
                (GraphFormat::Json, GraphLevel::Project) => {
                    serde_json::to_string_pretty(&full_report.analysis.stats.dependencies)?
                }
                (GraphFormat::Json, GraphLevel::File) => {
                    serde_json::to_string_pretty(&full_report.import_graph)?
                }
                (GraphFormat::Mermaid, GraphLevel::File) => return Err(
                    "file-level graph is not supported for mermaid (too large) — use dot or json"
                        .into(),
                ),
            };
            println!("{}", output);
        }
        Some(Command::Html { output_file }) => {
            std::fs::write(output_file, report::html::to_html(&full_report)?)?;
            println!("✅ HTML report saved to {:?}", output_file);
        }
        Some(Command::Sarif { output_file }) => {
            let sarif = report::sarif::to_sarif(&full_report);
            std::fs::write(output_file, serde_json::to_string_pretty(&sarif)?)?;
            println!("✅ SARIF report saved to {:?}", output_file);
        }
        Some(Command::Baseline { output_file }) => {
            let baseline = report::baseline::Baseline {
                findings: report::baseline::finding_keys(&full_report),
            };
            std::fs::write(output_file, serde_json::to_string_pretty(&baseline)?)?;
            println!(
                "✅ Baseline with {} findings saved to {:?}",
                baseline.findings.len(),
                output_file
            );
            return Ok(0); // Writing a baseline never fails the build.
        }
    }

    enforce_policy(args, &full_report)
}

fn main() {
    let args = Args::parse();

    match run(&args) {
        Ok(code) => std::process::exit(code),
        Err(e) => {
            eprintln!("Error during analysis: {}", e);
            std::process::exit(1);
        }
    }
}
