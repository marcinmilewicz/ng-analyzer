use crate::analyses::project_map::{is_entry_file, is_test_file, ProjectCatalog};
use crate::analysis::models::file_facts::{ExportInfo, ExportKind, LocalReference};
use crate::analysis::models::import::ImportKind;
use crate::analysis::resolvers::import_graph::ImportGraph;
use crate::ng::models::NgAnalysisResults;
use crate::ng::templates::TemplateUsageInfo;
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize)]
pub struct UnusedReport {
    /// Exported symbols nobody imports, renders or lazy-loads.
    pub unused_exports: Vec<UnusedSymbol>,
    /// Symbols used exclusively from test files.
    pub test_only_exports: Vec<UnusedSymbol>,
    /// Symbols never imported but referenced by live code in their own file
    /// (e.g. a union member of an exported, used union type) — the symbol is
    /// alive; only the `export` keyword may be unnecessary.
    pub export_only: Vec<UnusedSymbol>,
    /// Angular entities that are wired up (declared/imported) but never
    /// appear in any template, route or bootstrap.
    pub declared_not_rendered: Vec<UnusedSymbol>,
    /// Import statements whose local binding is never referenced in the file.
    /// Removable on their own — and, more importantly, they do NOT keep their
    /// target alive, which is what lets dead code hold dead code up.
    pub unused_imports: Vec<UnusedImport>,
    /// Files with no incoming edges at all (and not entry/test files).
    pub orphan_files: Vec<PathBuf>,
}

#[derive(Debug, Serialize)]
pub struct UnusedImport {
    /// Local binding introduced by the statement.
    pub name: String,
    /// Specifier it was imported from.
    pub specifier: String,
    pub file: PathBuf,
    pub project: String,
}

#[derive(Debug, Serialize)]
pub struct UnusedSymbol {
    pub name: String,
    pub kind: String,
    pub file: PathBuf,
    pub project: String,
    pub confidence: Confidence,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub enum Confidence {
    High,
    Medium,
}

/// Where a symbol is referenced from.
struct SymbolUsages {
    production_files: Vec<PathBuf>,
    test_files: Vec<PathBuf>,
}

pub fn find_unused(
    results: &NgAnalysisResults,
    template_usages: &[TemplateUsageInfo],
    import_graph: &ImportGraph,
    catalog: &ProjectCatalog,
) -> UnusedReport {
    // --- 1. Usage index: (declaring file, exported name) -> using files. ---
    let mut usages: HashMap<(PathBuf, String), SymbolUsages> = HashMap::new();
    let mut lazy_loaded_files: HashSet<PathBuf> = HashSet::new();

    let mut record_usage = |file: PathBuf, name: String, from: &Path| {
        let entry = usages.entry((file, name)).or_insert_with(|| SymbolUsages {
            production_files: Vec::new(),
            test_files: Vec::new(),
        });
        if is_test_file(from) {
            entry.test_files.push(from.to_path_buf());
        } else {
            entry.production_files.push(from.to_path_buf());
        }
    };

    let mut unused_imports: Vec<UnusedImport> = Vec::new();
    // (target file, file that namespace-imports it)
    let mut namespace_users: Vec<(PathBuf, PathBuf)> = Vec::new();

    for file in &results.source_files {
        for import in &file.imports {
            // `import './polyfills'` binds no name. It keeps the target's
            // top-level statements alive (that edge lives in the import
            // graph) but marks none of its exports as used.
            if import.imported_item.import_kind == ImportKind::SideEffect {
                continue;
            }

            let local = &import.imported_item.name;
            if !file.used_import_names.contains(local) {
                // A leftover import statement: the binding is never
                // referenced. Counting it as a usage is exactly how a dead
                // file keeps another dead file alive, so it does not count —
                // and it is a finding in its own right.
                //
                // Only reported for targets inside the workspace: a bare
                // `import React from 'react'` is required by the classic JSX
                // runtime without ever being referenced, and we cannot prove
                // anything about symbols we do not own.
                if catalog.project_of(&import.resolved_path).is_some() {
                    unused_imports.push(UnusedImport {
                        name: local.clone(),
                        specifier: import.source.clone(),
                        file: file.path.clone(),
                        project: catalog
                            .project_of(&file.path)
                            .map(|project| project.name.clone())
                            .unwrap_or_default(),
                    });
                }
                continue;
            }

            // `import * as ns` can reach every export through the namespace
            // object and we do not track which members are touched — so all
            // of them stay alive. Correctness over completeness (NFR-3).
            if import.imported_item.import_kind == ImportKind::Namespace {
                namespace_users.push((import.resolved_path.clone(), file.path.clone()));
                continue;
            }

            let exported = import
                .imported_item
                .alias
                .clone()
                .unwrap_or_else(|| local.clone());
            record_usage(import.resolved_path.clone(), exported, &file.path);
        }
        for import in &file.dynamic_imports {
            if !is_test_file(&file.path) {
                lazy_loaded_files.insert(import.resolved_path.clone());
            }
        }
    }

    // A namespace import is a usage of every export of the target file.
    let exports_by_file: HashMap<&Path, &[ExportInfo]> = results
        .source_files
        .iter()
        .map(|file| (file.path.as_path(), file.exports.as_slice()))
        .collect();
    for (target, from) in &namespace_users {
        let Some(exports) = exports_by_file.get(target.as_path()) else {
            continue;
        };
        for export in exports.iter() {
            record_usage(target.clone(), export.name.clone(), from);
        }
    }

    for usage in template_usages {
        // A recursive component rendering ITSELF is not a real usage —
        // without external consumers it is still dead.
        if usage.component_path == usage.target_path && usage.component == usage.target {
            continue;
        }
        record_usage(
            usage.target_path.clone(),
            usage.target.clone(),
            &usage.component_path,
        );
    }

    // Bootstrap components are always live.
    let bootstrap_names: HashSet<&str> = results
        .modules
        .iter()
        .flat_map(|module| module.bootstrap.iter().map(String::as_str))
        .collect();

    // --- 2. Metadata-only usage detection (declarations/imports arrays). ---
    // name -> set of files where the name appears in decorator metadata.
    let mut metadata_refs: HashMap<&str, HashSet<&Path>> = HashMap::new();
    for component in &results.components {
        for name in component
            .standalone_imports
            .iter()
            .chain(component.providers.iter())
        {
            metadata_refs
                .entry(name)
                .or_default()
                .insert(component.base.source_path.as_path());
        }
    }
    for module in &results.modules {
        for name in module
            .declarations
            .iter()
            .chain(module.imports_idents.iter())
            .chain(module.exports.iter())
            .chain(module.providers.iter())
        {
            metadata_refs
                .entry(name)
                .or_default()
                .insert(module.base.source_path.as_path());
        }
    }

    // Set of files that lazily reach a file through barrels: when a barrel is
    // lazy-loaded, everything it re-exports is reachable.
    let lazy_reachable: HashSet<PathBuf> = lazy_loaded_files
        .iter()
        .flat_map(|file| {
            let mut reachable = import_graph.get_all_dependencies(file);
            reachable.insert(file.clone());
            reachable
        })
        .collect();

    // Angular/React entity kinds beat the generic export kind ("Class") so
    // `unused --kind component` finds dead components.
    let mut entity_kinds: HashMap<(&Path, &str), &'static str> = HashMap::new();
    for component in &results.components {
        entity_kinds.insert(
            (component.base.source_path.as_path(), &component.base.name),
            "Component",
        );
    }
    for directive in &results.directives {
        entity_kinds.insert(
            (directive.base.source_path.as_path(), &directive.base.name),
            "Directive",
        );
    }
    for pipe in &results.pipes {
        entity_kinds.insert((pipe.base.source_path.as_path(), &pipe.base.name), "Pipe");
    }
    for service in &results.services {
        entity_kinds.insert(
            (service.base.source_path.as_path(), &service.base.name),
            "Service",
        );
    }
    for module in &results.modules {
        entity_kinds.insert(
            (module.base.source_path.as_path(), &module.base.name),
            "Module",
        );
    }
    for component in &results.react_components {
        entity_kinds.insert(
            (component.source_path.as_path(), &component.name),
            "ReactComponent",
        );
    }

    // --- 3. Unused exports. ---
    let mut unused_exports = Vec::new();
    let mut test_only_exports = Vec::new();
    let mut export_only = Vec::new();

    let is_ambient = |path: &Path| path.to_string_lossy().ends_with(".d.ts");

    for file in &results.source_files {
        if is_test_file(&file.path)
            || is_entry_file(&file.path)
            || catalog.is_framework_entry(&file.path)
            || is_ambient(&file.path)
        {
            continue;
        }
        let project = catalog
            .project_of(&file.path)
            .map(|p| p.name.clone())
            .unwrap_or_default();

        // Same-file liveness: a symbol referenced by a LIVE declaration in
        // its own file is alive too (union members, helper types, constants
        // used by exported functions). Roots: externally-used exports and
        // top-level statements (which run whenever anything is imported);
        // liveness then propagates through local references transitively.
        let mut alive_prod: HashSet<&str> = HashSet::new();
        let mut alive_test: HashSet<&str> = HashSet::new();
        for export in &file.exports {
            if let Some(symbol_usages) = usages.get(&(file.path.clone(), export.name.clone())) {
                if !symbol_usages.production_files.is_empty() {
                    alive_prod.insert(export.name.as_str());
                } else {
                    alive_test.insert(export.name.as_str());
                }
            }
            if bootstrap_names.contains(export.name.as_str()) {
                alive_prod.insert(export.name.as_str());
            }
        }
        let has_dependents = import_graph
            .get_dependents(&file.path)
            .is_some_and(|dependents| !dependents.is_empty());
        if has_dependents || lazy_reachable.contains(&file.path) {
            alive_prod.insert(""); // top-level statements execute on load
        }
        propagate_liveness(&mut alive_prod, &file.local_references);
        propagate_liveness(&mut alive_test, &file.local_references);

        for export in &file.exports {
            // Re-exports are just forwarding — the declaration is judged at
            // its own file.
            if matches!(export.kind, ExportKind::ReExport | ExportKind::ReExportAll) {
                continue;
            }
            if bootstrap_names.contains(export.name.as_str()) {
                continue;
            }
            if lazy_reachable.contains(&file.path) {
                continue;
            }

            let kind = entity_kinds
                .get(&(file.path.as_path(), export.name.as_str()))
                .map(|kind| kind.to_string())
                .unwrap_or_else(|| format!("{:?}", export.kind));
            let symbol = |confidence: Confidence| UnusedSymbol {
                name: export.name.clone(),
                kind: kind.clone(),
                file: file.path.clone(),
                project: project.clone(),
                confidence,
            };

            let key = (file.path.clone(), export.name.clone());
            let externally_used_in_prod = usages
                .get(&key)
                .is_some_and(|u| !u.production_files.is_empty());
            if externally_used_in_prod {
                continue;
            }

            if alive_prod.contains(export.name.as_str()) {
                // Alive through its own file — only the `export` is suspect.
                export_only.push(symbol(Confidence::Medium));
            } else if usages.contains_key(&key) || alive_test.contains(export.name.as_str()) {
                test_only_exports.push(symbol(Confidence::High));
            } else {
                unused_exports.push(symbol(Confidence::High));
            }
        }
    }

    // --- 4. Angular entities wired up but never rendered. ---
    let mut declared_not_rendered = Vec::new();

    let template_used: HashSet<(&Path, &str)> = template_usages
        .iter()
        .filter(|usage| {
            // Self-renders don't count (recursive components).
            !(usage.component_path == usage.target_path && usage.component == usage.target)
        })
        .map(|usage| (usage.target_path.as_path(), usage.target.as_str()))
        .collect();

    let mut check_entity = |name: &str, path: &Path, kind: &str| {
        if template_used.contains(&(path, name)) {
            return;
        }
        if bootstrap_names.contains(name) {
            return;
        }
        if lazy_reachable.contains(path) {
            return;
        }
        let key = (path.to_path_buf(), name.to_string());
        let Some(symbol_usages) = usages.get(&key) else {
            return; // Fully unused — already reported in unused_exports.
        };
        if symbol_usages.production_files.is_empty() {
            return; // Test-only — already reported.
        }

        // Every production usage comes from a file where the name appears in
        // decorator metadata (imports/declarations) — wired up, not rendered.
        let all_metadata = symbol_usages.production_files.iter().all(|from| {
            metadata_refs
                .get(name)
                .is_some_and(|files| files.contains(from.as_path()))
        });
        if all_metadata {
            declared_not_rendered.push(UnusedSymbol {
                name: name.to_string(),
                kind: kind.to_string(),
                file: path.to_path_buf(),
                project: catalog
                    .project_of(path)
                    .map(|p| p.name.clone())
                    .unwrap_or_default(),
                confidence: Confidence::Medium,
            });
        }
    };

    for component in &results.components {
        check_entity(
            &component.base.name,
            &component.base.source_path,
            "Component",
        );
    }
    for directive in &results.directives {
        check_entity(
            &directive.base.name,
            &directive.base.source_path,
            "Directive",
        );
    }
    for pipe in &results.pipes {
        check_entity(&pipe.base.name, &pipe.base.source_path, "Pipe");
    }

    // --- 5. Orphan files: no incoming edges at all.
    //
    // A barrel is a pass-through: imports THROUGH it resolve to the declaring
    // file, so it never receives an inbound edge itself and calling it an
    // orphan is a guaranteed false positive. What makes a file a barrel is
    // that everything it exports is a re-export — `index.ts` is merely the
    // most common spelling. Testing the name alone reports every `types.ts`,
    // `server.ts` and `public-api.ts` re-export hub as dead. ---
    let is_barrel = |file: &crate::analysis::models::file_facts::FileFactsInfo| {
        file.path.file_stem().is_some_and(|stem| stem == "index")
            || (!file.exports.is_empty()
                && file
                    .exports
                    .iter()
                    .all(|export| export.from_module.is_some()))
    };
    let mut orphan_files: Vec<PathBuf> = results
        .source_files
        .iter()
        .filter(|file| {
            !is_test_file(&file.path)
                && !is_entry_file(&file.path)
                && !catalog.is_framework_entry(&file.path)
                && !is_ambient(&file.path)
                && !is_barrel(file)
                && !lazy_reachable.contains(&file.path)
                && import_graph
                    .get_dependents(&file.path)
                    .is_none_or(|dependents| dependents.is_empty())
        })
        .map(|file| file.path.clone())
        .collect();
    orphan_files.sort();

    unused_exports.sort_by(|a, b| (&a.file, &a.name).cmp(&(&b.file, &b.name)));
    test_only_exports.sort_by(|a, b| (&a.file, &a.name).cmp(&(&b.file, &b.name)));
    export_only.sort_by(|a, b| (&a.file, &a.name).cmp(&(&b.file, &b.name)));
    declared_not_rendered.sort_by(|a, b| (&a.file, &a.name).cmp(&(&b.file, &b.name)));
    unused_imports
        .sort_by(|a, b| (&a.file, &a.name, &a.specifier).cmp(&(&b.file, &b.name, &b.specifier)));

    UnusedReport {
        unused_exports,
        test_only_exports,
        export_only,
        declared_not_rendered,
        unused_imports,
        orphan_files,
    }
}

/// Worklist propagation over same-file references: whatever a live
/// declaration mentions becomes live, transitively (union → member,
/// exported fn → helper type → nested type…).
fn propagate_liveness<'a>(alive: &mut HashSet<&'a str>, references: &'a [LocalReference]) {
    let mut changed = true;
    while changed {
        changed = false;
        for reference in references {
            if !alive.contains(reference.from.as_str()) {
                continue;
            }
            for target in &reference.to {
                if alive.insert(target.as_str()) {
                    changed = true;
                }
            }
        }
    }
}
