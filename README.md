# ngAnalyzer - Angular NX Project Analysis Tool

A Rust-based static analysis tool for Angular projects in NX workspaces. This tool analyzes Angular components, services, modules, directives, and pipes to provide insights into project structure and dependencies.

## Features

- **NX Workspace Support**: Automatically detects and processes NX workspace projects
- **Angular-specific files analysis**:
    - Components analysis 
    - Services analysis 
    - NgModules analysis 
    - Directives analysis
    - Pipes analysis (name, pure/impure status)
- **Import Analysis**: Resolves and tracks imports across files
- **Import Graph**: Dependency tracking between files
- **JSON Output**: Results exported in structured JSON format

## Command Line Options

```bash
USAGE:
    angular-analysis [OPTIONS]

OPTIONS:
    -d, --project-path <PATH>     Path to the project directory [default: .]
    -o, --output-file <PATH>      Output file for analysis results [default: angular-analysis.json]
    -v, --verbose                 Enable verbose output
    -p, --projects <PROJECTS>     Filter specific projects (comma-separated)
    -n, --exclude-node-modules    Exclude node_modules [default: true]
    -t, --typescript-only         Filter TypeScript files only [default: true]
```

## Project Structure

```
src/
├── analysis/           # Core analysis functionality
├── ng/                # Angular-specific analysis
└──  nx/                # NX workspace handling
```

## Needed Core Enhancements (under development)

1. **Template Analysis**
    - Parse Angular HTML templates
    - Track component references in templates


2. **Dependency Usage Analysis**
    - Track internal library usage within NX workspace
    - Analyze external dependency usage
    - Generate dependency usage reports
    - Detect unused dependencies


3. **Test Coverage**
    - Unit tests for core functionality
    - Integration tests for file processing
    - Test fixtures for different Angular patterns
    - Performance benchmarking tests

## Usage Example

```rust
let mut results = NgAnalysisResults::default();
let workspace = NxWorkspace::new(path);
let processor = ProjectProcessor::new(
    project_path,
    project_name,
    ts_config,
    cache,
    file_reader,
    source_map,
    import_graph
);

let mut results = NgAnalysisResults::default();
processor.process_files(&mut results);
```

## Installation

```bash
# Clone the repository
git clone [repository-url]

# Build the project
cargo build --release

# Run the analysis
./target/release/ng-analyzer -d /path/to/nx/workspace -v
```

## Contributing

Please wait yet for more stable version. Project is in early development stage and the directions are not established.
Project is live only for educational purposes.

## License

MIT