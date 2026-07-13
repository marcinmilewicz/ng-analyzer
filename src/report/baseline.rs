use crate::report::FullReport;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::path::Path;

/// A baseline is the set of finding keys accepted at a point in time.
/// `--baseline file` filters those out so CI only fails on NEW findings —
/// the standard brownfield-adoption mechanism.
#[derive(Serialize, Deserialize, Default)]
pub struct Baseline {
    pub findings: BTreeSet<String>,
}

pub fn finding_keys(report: &FullReport) -> BTreeSet<String> {
    let mut keys = BTreeSet::new();

    for symbol in &report.analysis.unused.unused_exports {
        keys.insert(format!("unused:{}:{}", symbol.file.display(), symbol.name));
    }
    for symbol in &report.analysis.unused.test_only_exports {
        keys.insert(format!(
            "test-only:{}:{}",
            symbol.file.display(),
            symbol.name
        ));
    }
    for symbol in &report.analysis.unused.export_only {
        keys.insert(format!(
            "export-only:{}:{}",
            symbol.file.display(),
            symbol.name
        ));
    }
    for symbol in &report.analysis.unused.declared_not_rendered {
        keys.insert(format!(
            "not-rendered:{}:{}",
            symbol.file.display(),
            symbol.name
        ));
    }
    for import in &report.analysis.unused.unused_imports {
        keys.insert(format!(
            "unused-import:{}:{}:{}",
            import.file.display(),
            import.name,
            import.specifier
        ));
    }
    for file in &report.analysis.unused.orphan_files {
        keys.insert(format!("orphan:{}", file.display()));
    }
    for cycle in &report.import_graph.circular_dependencies {
        let joined: Vec<String> = cycle.iter().map(|p| p.display().to_string()).collect();
        keys.insert(format!("cycle:{}", joined.join("->")));
    }
    for cycle in &report.analysis.stats.project_cycles {
        keys.insert(format!("project-cycle:{}", cycle.join("->")));
    }
    for violation in &report.analysis.boundary_violations {
        keys.insert(format!(
            "boundary:{}->{}:{}",
            violation.from, violation.to, violation.source_tag
        ));
    }

    keys
}

pub fn load(path: &Path) -> Result<Baseline, Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string(path)?;
    Ok(serde_json::from_str(&content)?)
}

/// Returns the finding keys NOT present in the baseline.
pub fn new_findings(report: &FullReport, baseline: &Baseline) -> BTreeSet<String> {
    finding_keys(report)
        .difference(&baseline.findings)
        .cloned()
        .collect()
}
