use crate::report::FullReport;
use std::fmt::Write;

/// Mermaid renders natively in GitHub/GitLab markdown — a project graph in a
/// PR description with zero infrastructure.
pub fn project_graph_mermaid(report: &FullReport) -> String {
    let mut out = String::from("graph LR\n");
    for project in &report.analysis.stats.projects {
        let _ = writeln!(out, "  {}[\"{}\"]", node_id(&project.name), project.name);
    }
    for dep in &report.analysis.stats.dependencies {
        let arrow = if dep.lazy { "-. lazy .->" } else { "-->" };
        let _ = writeln!(
            out,
            "  {} {}|{}| {}",
            node_id(&dep.from),
            arrow,
            dep.count,
            node_id(&dep.to)
        );
    }
    out
}

pub fn project_graph_dot(report: &FullReport) -> String {
    let mut out = String::from("digraph workspace {\n  rankdir=LR;\n  node [shape=box];\n");
    for project in &report.analysis.stats.projects {
        let _ = writeln!(out, "  \"{}\";", project.name);
    }
    for dep in &report.analysis.stats.dependencies {
        let style = if dep.lazy { ", style=dashed" } else { "" };
        let _ = writeln!(
            out,
            "  \"{}\" -> \"{}\" [label=\"{}\"{}];",
            dep.from, dep.to, dep.count, style
        );
    }
    out.push_str("}\n");
    out
}

/// File-level graph is only offered as DOT — too large for readable Mermaid.
pub fn file_graph_dot(report: &FullReport) -> String {
    let mut out = String::from("digraph files {\n  rankdir=LR;\n  node [shape=box, fontsize=9];\n");
    for edge in &report.import_graph.edges {
        for target in &edge.to {
            let _ = writeln!(
                out,
                "  \"{}\" -> \"{}\";",
                edge.from.display(),
                target.display()
            );
        }
    }
    out.push_str("}\n");
    out
}

/// Mermaid node ids must be alphanumeric-ish.
fn node_id(name: &str) -> String {
    name.chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect()
}
