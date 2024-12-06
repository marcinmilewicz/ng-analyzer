use clap::Parser;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
mod analysis;
mod file_cache_reader;
mod ng;
mod nx;

use crate::nx::NxWorkspace;
use analysis::timing::TimingMetrics;
use ng::models::NgAnalysisResults;
use std::sync::Arc;
use swc_common::SourceMap;

use crate::analysis::processor::file_processor::ProjectProcessor;
use crate::analysis::resolvers::cache::ImportCache;
use crate::analysis::resolvers::import_graph::ImportGraph;
use crate::file_cache_reader::CachedFileReader;
use crate::ng::ng_reporter::NgReporter;
use crate::nx::nx_project::NxProject;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the project directory
    #[arg(short = 'd', long, default_value = ".")]
    project_path: PathBuf,

    /// Cache duration in seconds
    #[arg(short, long, default_value = "300")]
    cache_duration: u64,

    /// Output file for analysis results
    #[arg(short, long, default_value = "angular-analysis.json")]
    output_file: PathBuf,

    /// Enable verbose output
    #[arg(short = 'v', long)]
    verbose: bool,

    /// Filter specific projects (comma-separated)
    #[arg(short = 'p', long)]
    projects: Option<String>,

    /// Exclude node_modules
    #[arg(short = 'n', long, default_value = "true")]
    exclude_node_modules: bool,

    /// Filter TypeScript files only
    #[arg(short = 't', long, default_value = "true")]
    typescript_only: bool,
}

fn process_workspace(args: &Args) -> Result<(), Box<dyn std::error::Error>> {
    let total_start = Instant::now();
    let mut metrics = TimingMetrics::new();

    if args.verbose {
        println!("🔍 Starting analysis with following parameters:");
        println!("Project path: {:?}", args.project_path);
        println!("Cache duration: {} seconds", args.cache_duration);
        println!("Output file: {:?}", args.output_file);
    }

    let source_map = Arc::new(SourceMap::default());
    let project_path = Path::new(&args.project_path);

    println!("🔍 Loading NX Workspace configuration...");
    let mut results = NgAnalysisResults::default();

    let workspace_start = Instant::now();
    let mut nx_workspace = NxWorkspace::new(project_path.to_path_buf());
    nx_workspace.load_configuration()?;
    metrics.workspace_load_time = workspace_start.elapsed();

    let file_reader = CachedFileReader::new(Duration::from_secs(args.cache_duration));
    let shared_cache = ImportCache::new();
    let shared_file_reader = Arc::new(file_reader);
    let import_graph = Arc::new(ImportGraph::new());

    let projects: Vec<(PathBuf, NxProject)> = if let Some(project_filter) = &args.projects {
        let project_names: Vec<&str> = project_filter.split(',').collect();
        nx_workspace
            .get_projects()
            .into_iter()
            .filter(|(_, project)| project_names.contains(&project.name.as_str()))
            .map(|(path, project)| (path.clone(), project.clone()))
            .collect()
    } else {
        nx_workspace
            .get_projects()
            .into_iter()
            .map(|(path, project)| (path.clone(), project.clone()))
            .collect()
    };

    for (project_path, project) in &projects {
        let mut processor = ProjectProcessor::new(
            project_path.clone(),
            project.name.clone(),
            project.ts_config.clone(),
            shared_cache.clone_cache(),
            shared_file_reader.clone_cache(),
            Arc::clone(&source_map),
            Arc::clone(&import_graph),
        );

        if args.exclude_node_modules {
            processor = processor.filter_node_modules();
        }

        if args.typescript_only {
            processor = processor.filter_ts_files();
        }

        processor.process_files(&mut results);

        println!("📦 Project {} has been processed", project.name);
    }

    metrics.total_time = total_start.elapsed();

    if args.verbose {
        NgReporter::print_analysis(&results);
        metrics.print_summary();
        println!("{} projects have been processed", projects.len());
    }

    let json = serde_json::to_string_pretty(&results)?;
    std::fs::write(&args.output_file, json)?;
    println!("✅ Analysis results saved to {:?}", args.output_file);

    Ok(())
}

fn main() {
    let args = Args::parse();

    if let Err(e) = process_workspace(&args) {
        eprintln!("Error during analysis: {}", e);
        std::process::exit(1);
    }
}