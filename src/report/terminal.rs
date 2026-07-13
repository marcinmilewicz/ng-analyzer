use crate::report::FullReport;

/// `project` narrows the output to rows involving that project.
pub fn print_stats(report: &FullReport, project: Option<&str>) {
    println!("📊 Projects:");
    println!(
        "{:<28} {:>6} {:>8} {:>4} {:>4} {:>6}  type/tags",
        "project", "files", "exports", "Ca", "Ce", "I"
    );
    for stats in &report.analysis.stats.projects {
        if project.is_some_and(|name| name != stats.name) {
            continue;
        }
        println!(
            "{:<28} {:>6} {:>8} {:>4} {:>4} {:>6.2}  {} {}",
            stats.name,
            stats.files,
            stats.exports,
            stats.afferent,
            stats.efferent,
            stats.instability,
            stats.project_type,
            stats.tags.join(",")
        );
    }

    println!("\n🔗 Dependencies (package → package):");
    for dep in &report.analysis.stats.dependencies {
        if project.is_some_and(|name| name != dep.from && name != dep.to) {
            continue;
        }
        let lazy = if dep.lazy { " [lazy]" } else { "" };
        println!("  {} → {} ({} refs){}", dep.from, dep.to, dep.count, lazy);
        for symbol in &dep.symbols {
            println!("      {} ×{}", symbol.name, symbol.count);
        }
    }

    if !report.analysis.stats.project_cycles.is_empty() {
        println!("\n🔄 Project cycles:");
        for cycle in &report.analysis.stats.project_cycles {
            println!("  {}", cycle.join(" ⇄ "));
        }
    }
}

/// `project` and `kinds` narrow the output (kinds match case-insensitively:
/// component, service, pipe, directive, class, function, variable, ...).
/// Every unresolved internal specifier is an edge missing from the graph, and
/// a missing edge is how a live symbol lands on the dead list. Surfacing this
/// above the findings is the difference between a work list and a guess.
pub fn print_resolution_warning(report: &FullReport) {
    let resolution = &report.analysis.resolution;
    if resolution.is_trustworthy() {
        return;
    }

    eprintln!(
        "⚠️  {} import(s) inside the workspace could not be resolved — the symbol graph is\n\
         incomplete, so findings below may contain false positives. Fix these first:",
        resolution.unresolved_internal.len()
    );
    for unresolved in resolution.unresolved_internal.iter().take(10) {
        eprintln!(
            "   {} → {}",
            unresolved.file.display(),
            unresolved.specifier
        );
    }
    if resolution.unresolved_internal.len() > 10 {
        eprintln!(
            "   … and {} more",
            resolution.unresolved_internal.len() - 10
        );
    }
    eprintln!();
}

pub fn print_unused(report: &FullReport, project: Option<&str>, kinds: &[String]) {
    let unused = &report.analysis.unused;

    let matches = |symbol: &crate::analyses::unused::UnusedSymbol| {
        if project.is_some_and(|name| name != symbol.project) {
            return false;
        }
        if !kinds.is_empty()
            && !kinds
                .iter()
                .any(|kind| kind.eq_ignore_ascii_case(&symbol.kind))
        {
            return false;
        }
        true
    };

    let unused_exports: Vec<_> = unused
        .unused_exports
        .iter()
        .filter(|s| matches(s))
        .collect();
    let not_rendered: Vec<_> = unused
        .declared_not_rendered
        .iter()
        .filter(|s| matches(s))
        .collect();
    let test_only: Vec<_> = unused
        .test_only_exports
        .iter()
        .filter(|s| matches(s))
        .collect();

    println!("🪦 Unused exports ({}):", unused_exports.len());
    for symbol in &unused_exports {
        println!(
            "  {} [{}] — {} ({})",
            symbol.name,
            symbol.kind,
            symbol.file.display(),
            symbol.project
        );
    }

    println!("\n🧟 Declared but never rendered ({}):", not_rendered.len());
    for symbol in &not_rendered {
        println!(
            "  {} [{}] — {}",
            symbol.name,
            symbol.kind,
            symbol.file.display()
        );
    }

    println!("\n🧪 Used only in tests ({}):", test_only.len());
    for symbol in &test_only {
        println!("  {} — {}", symbol.name, symbol.file.display());
    }

    let export_only: Vec<_> = unused.export_only.iter().filter(|s| matches(s)).collect();
    println!(
        "\n📦 Used in own file only — `export` may be unnecessary ({}):",
        export_only.len()
    );
    for symbol in &export_only {
        println!(
            "  {} [{}] — {}",
            symbol.name,
            symbol.kind,
            symbol.file.display()
        );
    }

    // Import statements and orphan files have no symbol kind — filter by
    // project only.
    if kinds.is_empty() {
        let dead_imports: Vec<_> = unused
            .unused_imports
            .iter()
            .filter(|import| project.is_none_or(|name| name == import.project))
            .collect();
        println!("\n🧹 Unused import statements ({}):", dead_imports.len());
        for import in dead_imports {
            println!(
                "  {} from '{}' — {}",
                import.name,
                import.specifier,
                import.file.display()
            );
        }

        let orphans: Vec<_> = unused
            .orphan_files
            .iter()
            .filter(|_| project.is_none())
            .collect();
        println!("\n👻 Orphan files ({}):", orphans.len());
        for file in orphans {
            println!("  {}", file.display());
        }
    }
}

pub fn print_cycles(report: &FullReport) {
    println!(
        "🔄 Project cycles ({}):",
        report.analysis.stats.project_cycles.len()
    );
    for cycle in &report.analysis.stats.project_cycles {
        println!("  {}", cycle.join(" ⇄ "));
    }

    println!(
        "\n🔁 File cycles ({}):",
        report.import_graph.circular_dependencies.len()
    );
    for cycle in &report.import_graph.circular_dependencies {
        let joined: Vec<String> = cycle.iter().map(|p| p.display().to_string()).collect();
        println!("  {}", joined.join(" → "));
    }
}

/// `project` narrows to candidates from or into that project.
pub fn print_move_candidates(report: &FullReport, project: Option<&str>) {
    let candidates: Vec<_> = report
        .analysis
        .move_candidates
        .iter()
        .filter(|candidate| {
            project
                .is_none_or(|name| candidate.from_project == name || candidate.to_project == name)
        })
        .collect();

    println!("📦 Move candidates ({}):", candidates.len());
    for candidate in candidates {
        println!(
            "  {} : {} → {} ({} uses, 0 internal) — {}",
            candidate.symbol,
            candidate.from_project,
            candidate.to_project,
            candidate.external_usages,
            candidate.file.display()
        );
    }
}

pub fn print_boundaries(report: &FullReport) {
    println!(
        "🚧 Boundary violations ({}):",
        report.analysis.boundary_violations.len()
    );
    for violation in &report.analysis.boundary_violations {
        println!(
            "  {} → {} — tag `{}` allows only [{}], target has [{}]",
            violation.from,
            violation.to,
            violation.source_tag,
            violation.allowed_tags.join(", "),
            violation.to_tags.join(", ")
        );
    }
}
