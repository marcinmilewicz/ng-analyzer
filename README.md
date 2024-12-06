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

## Example Output
```bash
🔍 Loading NX Workspace configuration...
📦 Project api-layer has been processed
📦 Project course-catalog-feature has been processed
📦 Project employee-list-feature has been processed
📦 Project course-details-feature has been processed
📦 Project course-details-data-access has been processed
📦 Project shared-ui has been processed
📦 Project ddd-hrm has been processed
📦 Project course-model-shared has been processed
📦 Project course-catalog-data-access has been processed
📦 Project course-shared has been processed
📦 Project employee-list-data-access has been processed
📦 Project employee-profile-data-access has been processed
📦 Project logger has been processed
📦 Project employee-profile-feature has been processed
📦 Project employee-model-shared has been processed
🔍 Analyzing Angular project...


📊 Analysis Results:
  - CourseCatalogFeatureComponent (src/lib/course-catalog-feature/course-catalog-feature.component.ts)
    Selector: ddd-hrm-course-catalog-feature
    Package: course-catalog-feature
    Template: ./course-catalog-feature.component.html
  - EmploymentListFeatureComponent (src/lib/employment-list-feature/employment-list-feature.component.ts)
    Selector: ddd-hrm-employment-list-feature
    Package: employee-list-feature
    Template: ./employment-list-feature.component.html
  - CourseDetailsFeatureComponent (src/lib/course-details-feature/course-details-feature.component.ts)
    Selector: ddd-hrm-course-details-feature
    Package: course-details-feature
    Template: ./course-details-feature.component.html
  - CardComponent (src/lib/card/card.component.ts)
    Selector: ddd-hrm-card
    Package: shared-ui
    Template: ./card.component.html
    Imports:
      - ImportedItem { name: "BadgeComponent", alias: None, import_kind: Named } from ../badge/badge.component [resolved: ../ddd-hrm/libs/shared/ui/src/lib/badge/badge.component.ts]
  - CardFieldComponent (src/lib/card-field/card-field.component.ts)
    Selector: ddd-hrm-card-field
    Package: shared-ui
    Template: ./card-field.component.html
  - BadgeComponent (src/lib/badge/badge.component.ts)
    Selector: ddd-hrm-badge
    Package: shared-ui
    Template: ./badge.component.html
  - AppComponent (src/app/app.component.ts)
    Selector: ddd-hrm-root
    Package: ddd-hrm
    Template: ./app.component.html
  - SelectEmployeeForLearningComponent (src/lib/select-employee-for-learning/select-employee-for-learning.component.ts)
    Selector: ddd-hrm-select-employee-for-learning
    Package: course-shared
    Template: ./select-employee-for-learning.component.html
  - EmployeeProfileFeatureComponent (src/lib/employee-profile-feature/employee-profile-feature.component.ts)
    Selector: ddd-hrm-employee-profile-feature
    Package: employee-profile-feature
    Template: ./employee-profile-feature.component.html
  - CourseDetailsApiService (src/lib/course-details-api.service.ts)
    Package: course-details-data-access
    Imports:
      - ImportedItem { name: "CourseDetails", alias: None, import_kind: Named } from ./course-details.model [resolved: ../ddd-hrm/libs/learning-management/course-details-data-access/src/lib/course-details.model.ts]
      - ImportedItem { name: "EmployeeAssignmentForCourse", alias: None, import_kind: Named } from ./course-details.model [resolved: ../ddd-hrm/libs/learning-management/course-details-data-access/src/lib/course-details.model.ts]
  - CourseCatalogApiService (src/lib/course-catalog-api.service.ts)
    Package: course-catalog-data-access
    Imports:
      - ImportedItem { name: "CourseListItem", alias: None, import_kind: Named } from ./course-catalog.model [resolved: ../ddd-hrm/libs/learning-management/course-catalog-data-access/src/lib/course-catalog.model.ts]
  - EmployeeListApiService (src/lib/employee-list-api.service.ts)
    Package: employee-list-data-access
  - EmployeeProfileApiService (src/lib/services/employee-profile-api.service.ts)
    Package: employee-profile-data-access
    Imports:
      - ImportedItem { name: "CourseAssignmentForEmployee", alias: None, import_kind: Named } from ../models/employee-profile.model [resolved: ../ddd-hrm/libs/employee-management/employee-profile-data-access/src/lib/models/employee-profile.model.ts]
      - ImportedItem { name: "EmployeeDetails", alias: None, import_kind: Named } from ../models/employee-profile.model [resolved: ../ddd-hrm/libs/employee-management/employee-profile-data-access/src/lib/models/employee-profile.model.ts]
  - LoggerService (src/lib/logger.service.ts)
    Package: logger
Components found: 9

Services found: 5

Modules found: 0

Directives found: 0

Pipes found: 0

⏱️ Timing Analysis:
Workspace load time: 56.165666ms
Total analysis time: 0ns
Total execution time: 68.205709ms
15 projects have been processed

```

## Contributing

Please wait yet for more stable version. Project is in early development stage and the directions are not established.
Project is live only for educational purposes.

## License

MIT