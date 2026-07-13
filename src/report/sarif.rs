use crate::report::FullReport;
use serde_json::{json, Value};

/// Minimal valid SARIF 2.1.0 for GitHub code scanning: unused code, cycles
/// and boundary violations as warnings.
pub fn to_sarif(report: &FullReport) -> Value {
    let mut results = Vec::new();

    for symbol in &report.analysis.unused.unused_exports {
        results.push(result(
            "unused-export",
            &format!(
                "Unused export `{}` in project `{}` — nothing imports, renders or lazy-loads it.",
                symbol.name, symbol.project
            ),
            &symbol.file.display().to_string(),
        ));
    }
    for symbol in &report.analysis.unused.export_only {
        results.push(result(
            "export-only",
            &format!(
                "`{}` is only used inside its own file — the `export` keyword may be unnecessary.",
                symbol.name
            ),
            &symbol.file.display().to_string(),
        ));
    }
    for symbol in &report.analysis.unused.declared_not_rendered {
        results.push(result(
            "declared-not-rendered",
            &format!(
                "`{}` is wired up (declarations/imports) but never appears in any template.",
                symbol.name
            ),
            &symbol.file.display().to_string(),
        ));
    }
    for file in &report.analysis.unused.orphan_files {
        results.push(result(
            "orphan-file",
            "File has no incoming dependencies.",
            &file.display().to_string(),
        ));
    }
    for cycle in &report.import_graph.circular_dependencies {
        let joined: Vec<String> = cycle.iter().map(|p| p.display().to_string()).collect();
        results.push(result(
            "circular-dependency",
            &format!("Circular file dependency: {}", joined.join(" -> ")),
            &joined[0],
        ));
    }
    for violation in &report.analysis.boundary_violations {
        results.push(result(
            "boundary-violation",
            &format!(
                "Project `{}` (tag `{}`) must not depend on `{}` (tags: {}).",
                violation.from,
                violation.source_tag,
                violation.to,
                violation.to_tags.join(", ")
            ),
            &violation.from,
        ));
    }

    json!({
        "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json",
        "version": "2.1.0",
        "runs": [{
            "tool": {
                "driver": {
                    "name": "nx-analyzer",
                    "informationUri": "https://github.com/nx-analyzer/nx-analyzer",
                    "rules": [
                        rule("unused-export", "Exported symbol is never used"),
                        rule("export-only", "Symbol used in its own file only — export may be unnecessary"),
                        rule("declared-not-rendered", "Angular entity wired up but never rendered"),
                        rule("orphan-file", "File with no incoming dependencies"),
                        rule("circular-dependency", "Circular dependency between files"),
                        rule("boundary-violation", "NX tag boundary rule violation"),
                    ]
                }
            },
            "results": results
        }]
    })
}

fn rule(id: &str, description: &str) -> Value {
    json!({ "id": id, "shortDescription": { "text": description } })
}

fn result(rule_id: &str, message: &str, uri: &str) -> Value {
    json!({
        "ruleId": rule_id,
        "level": "warning",
        "message": { "text": message },
        "locations": [{
            "physicalLocation": {
                "artifactLocation": { "uri": uri }
            }
        }]
    })
}
