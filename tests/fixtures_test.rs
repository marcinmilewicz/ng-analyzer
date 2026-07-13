use serde_json::Value;
use std::path::Path;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

static RUN_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Runs the compiled binary against a fixture workspace and returns the
/// parsed JSON report. The working directory is the crate root, so all
/// paths in the report are stable, relative paths. The output file is
/// unique per invocation — tests run in parallel and must not share it.
fn run_fixture(name: &str) -> Value {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let out_dir = Path::new(manifest_dir).join("target").join("test-output");
    std::fs::create_dir_all(&out_dir).unwrap();
    let run_id = RUN_COUNTER.fetch_add(1, Ordering::Relaxed);
    let out_file = out_dir.join(format!("{name}-{}-{run_id}.json", std::process::id()));

    let output = Command::new(env!("CARGO_BIN_EXE_nx-analyzer"))
        .current_dir(manifest_dir)
        .args([
            "-d",
            &format!("tests/fixtures/{name}"),
            "analyze",
            "-o",
            out_file.to_str().unwrap(),
        ])
        .output()
        .expect("failed to run nx-analyzer");
    assert!(
        output.status.success(),
        "analyzer failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let content = std::fs::read_to_string(&out_file).unwrap();
    serde_json::from_str(&content).unwrap()
}

fn names(report: &Value, kind: &str) -> Vec<String> {
    report[kind]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item["name"].as_str().unwrap().to_string())
        .collect()
}

fn find<'a>(report: &'a Value, kind: &str, name: &str) -> &'a Value {
    report[kind]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["name"] == name)
        .unwrap_or_else(|| panic!("{kind} {name} not found"))
}

/// Resolved paths of all imports of the given entity.
fn resolved_imports(entity: &Value) -> Vec<String> {
    entity["imports"]
        .as_array()
        .unwrap()
        .iter()
        .map(|import| import["resolved_path"].as_str().unwrap().to_string())
        .collect()
}

#[test]
fn f01_detects_components_and_services() {
    let report = run_fixture("f01-basic-imports");

    assert_eq!(
        names(&report, "components"),
        vec!["FeatureAComponent", "UiButtonComponent"]
    );
    assert_eq!(
        names(&report, "services"),
        vec!["BrokenImportService", "CustomersService", "OrdersService"]
    );

    let button = find(&report, "components", "UiButtonComponent");
    assert_eq!(button["selector"], "fix-button");
    assert_eq!(button["standalone"], true);
    assert_eq!(button["package_name"], "ui");
}

#[test]
fn f01_resolves_barrel_imports_to_declaring_files() {
    let report = run_fixture("f01-basic-imports");

    let feature = find(&report, "components", "FeatureAComponent");
    let imports = resolved_imports(feature);

    // Both symbols come from the '@fix/ui' barrel but live in different files.
    assert!(
        imports
            .iter()
            .any(|path| path.ends_with("libs/ui/src/lib/button.component.ts")),
        "UiButtonComponent should resolve through the barrel, got: {imports:?}"
    );
    assert!(
        imports
            .iter()
            .any(|path| path.ends_with("libs/ui/src/lib/button.model.ts")),
        "ButtonConfig should resolve through the barrel, got: {imports:?}"
    );
}

#[test]
fn f01_relative_imports_are_cached_per_directory() {
    // B1 regression: './model' imported from two different directories must
    // resolve to two different files.
    let report = run_fixture("f01-basic-imports");

    let orders = resolved_imports(find(&report, "services", "OrdersService"));
    let customers = resolved_imports(find(&report, "services", "CustomersService"));

    assert!(
        orders.iter().any(|path| path.ends_with("orders/model.ts")),
        "OrdersService should import orders/model.ts, got: {orders:?}"
    );
    assert!(
        customers
            .iter()
            .any(|path| path.ends_with("customers/model.ts")),
        "CustomersService should import customers/model.ts, got: {customers:?}"
    );
}

#[test]
fn f01_nonexistent_import_creates_no_phantom_edge() {
    // B6 regression: './does-not-exist' must not appear anywhere.
    let report = run_fixture("f01-basic-imports");

    let broken = resolved_imports(find(&report, "services", "BrokenImportService"));
    assert!(
        broken.is_empty(),
        "BrokenImportService should have no resolved imports, got: {broken:?}"
    );

    let graph = serde_json::to_string(&report["import_graph"]).unwrap();
    assert!(!graph.contains("does-not-exist"));
}

#[test]
fn f01_unused_lib_has_no_incoming_edges() {
    let report = run_fixture("f01-basic-imports");

    for edge in report["import_graph"]["edges"].as_array().unwrap() {
        // Internal edges (barrel re-exports) are fine; nothing from OUTSIDE
        // the package may depend on it.
        if edge["from"].as_str().unwrap().contains("libs/util") {
            continue;
        }
        for target in edge["to"].as_array().unwrap() {
            assert!(
                !target.as_str().unwrap().contains("libs/util"),
                "nothing outside should import libs/util"
            );
        }
    }
    assert_eq!(
        report["import_graph"]["circular_dependencies"]
            .as_array()
            .unwrap()
            .len(),
        0
    );
}

#[test]
fn f01_snapshot() {
    let report = run_fixture("f01-basic-imports");
    insta::assert_json_snapshot!("f01-basic-imports", report);
}

#[test]
fn f03_resolves_paths_via_extends_chain_and_non_at_aliases() {
    let report = run_fixture("f03-tsconfig-paths");

    let consumer = find(&report, "services", "ConsumerService");
    let imports = resolved_imports(consumer);

    // 'shared/helper' — alias without '@', reachable only through the
    // two-level extends chain (B11, B12).
    assert!(
        imports
            .iter()
            .any(|path| path.ends_with("libs/shared/src/lib/helper.ts")),
        "non-@ alias should resolve, got: {imports:?}"
    );
    // '@fix/multi' — first paths variant doesn't exist, second must win.
    assert!(
        imports
            .iter()
            .any(|path| path.ends_with("libs/multi/src/lib/multi.model.ts")),
        "multi-variant alias should resolve through the barrel, got: {imports:?}"
    );
}

#[test]
fn f03_base_url_is_anchored_to_declaring_tsconfig() {
    // A project tsconfig overriding baseUrl with "../.." (Next.js apps do
    // this) anchors it at ITS OWN directory — the workspace root — not two
    // levels above the workspace. Anchoring it wrong silently breaks every
    // alias import in the project, and its imports vanish from usage joins.
    let report = run_fixture("f03-tsconfig-paths");

    let deep = find(&report, "services", "DeepBaseService");
    let imports = resolved_imports(deep);
    assert!(
        imports
            .iter()
            .any(|path| path.ends_with("libs/shared/src/lib/helper.ts")),
        "alias should resolve under a re-anchored baseUrl, got: {imports:?}"
    );
}

#[test]
fn f03_project_without_tsconfig_is_still_analyzed() {
    // B9 regression: only tsconfig.lib.json next to project.json.
    let report = run_fixture("f03-tsconfig-paths");
    let service = find(&report, "services", "OrphanConfigService");
    assert_eq!(service["package_name"], "no-tsconfig");
}

#[test]
fn f03_project_without_name_gets_inferred_name() {
    // B10 regression: project.json with neither name nor sourceRoot.
    let report = run_fixture("f03-tsconfig-paths");
    let service = find(&report, "services", "AnonymousService");
    assert_eq!(service["package_name"], "anonymous");
}

#[test]
fn f03_snapshot() {
    let report = run_fixture("f03-tsconfig-paths");
    insta::assert_json_snapshot!("f03-tsconfig-paths", report);
}

#[test]
fn f12_finds_classes_in_every_export_position() {
    // B5 regression.
    let report = run_fixture("f12-edge-cases");

    assert_eq!(names(&report, "components"), vec!["DefaultExportComponent"]);
    assert_eq!(names(&report, "directives"), vec!["LaterExportedDirective"]);
    assert_eq!(
        names(&report, "services"),
        vec!["ImportVariantsService", "InternalService"]
    );
}

#[test]
fn f12_handles_import_variants_and_survives_broken_files() {
    let report = run_fixture("f12-edge-cases");

    let variants = find(&report, "services", "ImportVariantsService");
    let imports = resolved_imports(variants);

    // Aliased import (`helperOne as one`) and namespace import both resolve
    // to helpers.ts; broken.ts and `export =` must not crash the run
    // (proven by the report existing at all).
    assert!(
        imports
            .iter()
            .filter(|path| path.ends_with("libs/edge/src/lib/helpers.ts"))
            .count()
            >= 1,
        "helpers.ts imports should resolve, got: {imports:?}"
    );
}

#[test]
fn f12_snapshot() {
    let report = run_fixture("f12-edge-cases");
    insta::assert_json_snapshot!("f12-edge-cases", report);
}

#[test]
fn f02_deep_barrel_chain_resolves_to_declaring_file() {
    let report = run_fixture("f02-barrel-exports");

    let consumer = find(&report, "services", "BarrelConsumerService");
    let imports = resolved_imports(consumer);

    // DeepButton is declared 3 re-export levels below the barrel.
    assert!(
        imports
            .iter()
            .any(|path| path.ends_with("libs/ui-kit/src/lib/deep/level3.ts")),
        "DeepButton should resolve through 3 barrel levels, got: {imports:?}"
    );
    // UiCard is an aliased re-export (`export {{ Card as UiCard }}`).
    assert!(
        imports
            .iter()
            .any(|path| path.ends_with("libs/ui-kit/src/lib/card.ts")),
        "UiCard should resolve through the aliased re-export, got: {imports:?}"
    );
}

#[test]
fn f02_barrel_records_reexport_kinds() {
    let report = run_fixture("f02-barrel-exports");

    let index = report["source_files"]
        .as_array()
        .unwrap()
        .iter()
        .find(|file| {
            file["path"]
                .as_str()
                .unwrap()
                .ends_with("ui-kit/src/index.ts")
        })
        .expect("barrel index.ts should be in source_files");

    let kinds: Vec<(&str, &str)> = index["exports"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| (e["name"].as_str().unwrap(), e["kind"].as_str().unwrap()))
        .collect();

    assert!(kinds.contains(&("* from ./lib/level1", "ReExportAll")));
    assert!(kinds.contains(&("UiCard", "ReExport")));
    assert!(kinds.contains(&("tokens", "ReExport")));
}

#[test]
fn f04_angular19_defaults_to_standalone() {
    let report = run_fixture("f04-standalone-components");

    // Neither component declares `standalone:` — Angular 19 default applies.
    let badge = find(&report, "components", "BadgeComponent");
    assert_eq!(badge["standalone"], true);

    let panel = find(&report, "components", "PanelComponent");
    assert_eq!(panel["standalone"], true);
}

#[test]
fn f04_collects_signals_inline_template_and_imports() {
    let report = run_fixture("f04-standalone-components");

    let badge = find(&report, "components", "BadgeComponent");
    assert!(badge["template_inline"].as_str().unwrap().contains("badge"));
    assert_eq!(badge["style_paths"][0], "./badge.component.css");

    let inputs: Vec<&str> = badge["inputs"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect();
    let outputs: Vec<&str> = badge["outputs"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect();
    // AST order: signal inputs (input.required, input, model), then @Input.
    assert_eq!(inputs, vec!["label", "variant", "counter", "legacyTitle"]);
    // AST order: signal output, then @Output.
    assert_eq!(outputs, vec!["dismissed", "legacyClosed"]);

    let panel = find(&report, "components", "PanelComponent");
    assert_eq!(panel["standalone_imports"][0], "BadgeComponent");
    assert_eq!(panel["providers"][0], "PanelStateService");
}

#[test]
fn f05_collects_full_ngmodule_metadata() {
    let report = run_fixture("f05-ngmodule-classic");

    let module = find(&report, "modules", "LegacyModule");
    assert_eq!(module["declarations"][0], "ListComponent");
    assert_eq!(module["declarations"][1], "DetailComponent");
    assert_eq!(module["imports_idents"][0], "CommonModule");
    assert_eq!(module["exports"][0], "ListComponent");
    assert_eq!(module["providers"][0], "LegacyStateService");
    assert_eq!(module["bootstrap"][0], "ListComponent");

    // @Injectable() without arguments must still be detected.
    let service = find(&report, "services", "LegacyStateService");
    assert_eq!(service["provided_in"], serde_json::Value::Null);
}

#[test]
fn f14_plain_ts_exports_and_dynamic_imports() {
    let report = run_fixture("f14-plain-ts");

    let math = report["source_files"]
        .as_array()
        .unwrap()
        .iter()
        .find(|file| file["path"].as_str().unwrap().ends_with("lib/math.ts"))
        .expect("math.ts should be in source_files");

    let exports: Vec<(&str, &str)> = math["exports"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| (e["name"].as_str().unwrap(), e["kind"].as_str().unwrap()))
        .collect();
    assert_eq!(
        exports,
        vec![
            ("add", "Function"),
            ("unusedMultiply", "Function"),
            ("PI_ISH", "Variable"),
            ("RoundingMode", "Enum"),
        ]
    );

    let calculator = report["source_files"]
        .as_array()
        .unwrap()
        .iter()
        .find(|file| {
            file["path"]
                .as_str()
                .unwrap()
                .ends_with("lib/calculator.ts")
        })
        .expect("calculator.ts should be in source_files");

    // Static imports resolved through the barrel + used names recorded.
    let used: Vec<&str> = calculator["used_import_names"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect();
    assert_eq!(used, vec!["Calculation", "add"]);

    // Dynamic import('@fix/toolbox') captured as a lazy edge.
    let dynamic = calculator["dynamic_imports"].as_array().unwrap();
    assert_eq!(dynamic.len(), 1);
    assert!(dynamic[0]["resolved_path"]
        .as_str()
        .unwrap()
        .ends_with("libs/toolbox/src/index.ts"));
}

fn template_usages(report: &Value) -> Vec<(String, String, String)> {
    report["template_usages"]
        .as_array()
        .unwrap()
        .iter()
        .map(|usage| {
            (
                usage["component"].as_str().unwrap().to_string(),
                usage["target"].as_str().unwrap().to_string(),
                usage["via"].as_str().unwrap().to_string(),
            )
        })
        .collect()
}

#[test]
fn f06_template_usages_cover_selectors_and_pipes() {
    let report = run_fixture("f06-templates");
    let usages = template_usages(&report);

    let page = |target: &str, via: &str| {
        (
            "PageComponent".to_string(),
            target.to_string(),
            via.to_string(),
        )
    };

    assert!(usages.contains(&page("UiButtonComponent", "Selector")));
    assert!(usages.contains(&page("UiTooltipDirective", "Selector")));
    // Structural directive sugar *uiIf matches the [uiIf] selector.
    assert!(usages.contains(&page("UiIfDirective", "Selector")));
    assert!(usages.contains(&page("UiCurrencyPipe", "Pipe")));
    // Imported but never used in the template — must NOT be a usage.
    assert!(!usages
        .iter()
        .any(|(_, target, _)| target == "UnusedInTemplateComponent"));
}

#[test]
fn f06_template_usage_creates_graph_edges() {
    let report = run_fixture("f06-templates");

    let edges = report["import_graph"]["edges"].as_array().unwrap();
    let page_edges = edges
        .iter()
        .find(|edge| {
            edge["from"]
                .as_str()
                .unwrap()
                .ends_with("page.component.ts")
        })
        .expect("PageComponent should have outgoing edges");

    let targets: Vec<&str> = page_edges["to"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t.as_str().unwrap())
        .collect();
    assert!(targets.iter().any(|t| t.ends_with("button.component.ts")));
    assert!(targets.iter().any(|t| t.ends_with("currency.pipe.ts")));
}

#[test]
fn f10_lazy_routes_create_dynamic_import_edges() {
    let report = run_fixture("f10-lazy-routes");

    let routes_file = report["source_files"]
        .as_array()
        .unwrap()
        .iter()
        .find(|file| file["path"].as_str().unwrap().ends_with("app.routes.ts"))
        .expect("app.routes.ts should be analyzed");

    let dynamic: Vec<&str> = routes_file["dynamic_imports"]
        .as_array()
        .unwrap()
        .iter()
        .map(|import| import["resolved_path"].as_str().unwrap())
        .collect();

    assert!(
        dynamic
            .iter()
            .any(|path| path.ends_with("libs/feature-lazy/src/index.ts")),
        "loadChildren should create a lazy edge, got: {dynamic:?}"
    );
    assert!(
        dynamic
            .iter()
            .any(|path| path.ends_with("libs/feature-page/src/index.ts")),
        "loadComponent should create a lazy edge, got: {dynamic:?}"
    );
}

#[test]
fn f11_di_usages_are_recorded() {
    let report = run_fixture("f11-di-providers");

    let consumer = report["source_files"]
        .as_array()
        .unwrap()
        .iter()
        .find(|file| {
            file["path"]
                .as_str()
                .unwrap()
                .ends_with("consumer.service.ts")
        })
        .expect("consumer.service.ts should be analyzed");

    let used: Vec<&str> = consumer["used_import_names"]
        .as_array()
        .unwrap()
        .iter()
        .map(|name| name.as_str().unwrap())
        .collect();

    // inject(ApiService) — call argument.
    assert!(used.contains(&"ApiService"));
    // constructor(private logger: Logger) — type usage.
    assert!(used.contains(&"Logger"));
    // useClass: FileLogger — provider object.
    assert!(used.contains(&"FileLogger"));
    // InjectionToken<AppConfig> — generic type argument.
    assert!(used.contains(&"AppConfig"));
}

fn unused_names(report: &Value, category: &str) -> Vec<String> {
    report["analysis"]["unused"][category]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item["name"].as_str().unwrap().to_string())
        .collect()
}

#[test]
fn f07_finds_exactly_the_dead_code() {
    let report = run_fixture("f07-unused-code");

    let unused = unused_names(&report, "unused_exports");
    assert_eq!(
        unused,
        vec![
            "DeadComponent",
            "DeadService",
            "DeadModel",
            "orphanThing",
            "DeadResult",
            "DeadSuccessResult",
            "deadUtil"
        ],
        "exactly the dead symbols, no false positives"
    );

    for name in [
        "TemplateOnlyComponent",
        "InjectOnlyService",
        "mainOnlyUtil",
        "AppComponent",
        "WiredNotRenderedComponent",
    ] {
        assert!(
            !unused.contains(&name.to_string()),
            "{name} must not be reported unused"
        );
    }
}

#[test]
fn f07_test_only_and_not_rendered_are_separate_categories() {
    let report = run_fixture("f07-unused-code");

    assert_eq!(
        unused_names(&report, "test_only_exports"),
        vec!["specOnlyHelper"]
    );
    assert_eq!(
        unused_names(&report, "declared_not_rendered"),
        vec!["WiredNotRenderedComponent"]
    );

    let orphans: Vec<&str> = report["analysis"]["unused"]["orphan_files"]
        .as_array()
        .unwrap()
        .iter()
        .map(|f| f.as_str().unwrap())
        .collect();
    assert_eq!(orphans.len(), 1, "got: {orphans:?}");
    assert!(orphans[0].ends_with("lib/orphan.ts"));
}

#[test]
fn f07_union_members_are_export_only_not_unused() {
    // A symbol nobody imports but referenced by a LIVE declaration in its own
    // file (union member, helper type) is alive — only its `export` keyword
    // is suspect. Liveness propagates transitively through local references.
    let report = run_fixture("f07-unused-code");
    let unused = unused_names(&report, "unused_exports");
    let export_only = unused_names(&report, "export_only");

    // FetchResult is imported by main.ts; its members live through it.
    assert!(
        !unused.contains(&"FetchResult".to_string()),
        "got: {unused:?}"
    );
    assert!(
        !unused.contains(&"FetchSuccessResult".to_string()),
        "union member of a used union must not be unused, got: {unused:?}"
    );
    assert!(
        export_only.contains(&"FetchSuccessResult".to_string()),
        "got: {export_only:?}"
    );
    assert!(export_only.contains(&"FetchErrorResult".to_string()));

    // References from DEAD exports must not revive anything.
    assert!(unused.contains(&"DeadResult".to_string()));
    assert!(unused.contains(&"DeadSuccessResult".to_string()));
    assert!(!export_only.contains(&"DeadSuccessResult".to_string()));
}

#[test]
fn f08_move_candidate_detected_with_thresholds() {
    let report = run_fixture("f08-move-candidate");

    let candidates = report["analysis"]["move_candidates"].as_array().unwrap();
    assert_eq!(
        candidates.len(),
        1,
        "only formatPrice qualifies, got: {candidates:?}"
    );

    let candidate = &candidates[0];
    assert_eq!(candidate["symbol"], "formatPrice");
    assert_eq!(candidate["from_project"], "shared-utils");
    assert_eq!(candidate["to_project"], "feature-checkout");
    assert_eq!(candidate["external_usages"], 2);
    assert_eq!(candidate["internal_usages"], 0);
}

#[test]
fn f09_file_and_project_cycles_detected() {
    let report = run_fixture("f09-circular-deps");

    let file_cycles = report["import_graph"]["circular_dependencies"]
        .as_array()
        .unwrap();
    assert!(
        file_cycles.iter().any(|cycle| {
            let files: Vec<&str> = cycle
                .as_array()
                .unwrap()
                .iter()
                .map(|f| f.as_str().unwrap())
                .collect();
            files.iter().any(|f| f.ends_with("tangle/src/lib/a.ts"))
                && files.iter().any(|f| f.ends_with("tangle/src/lib/b.ts"))
                && files.iter().any(|f| f.ends_with("tangle/src/lib/c.ts"))
        }),
        "a→b→c→a file cycle should be detected, got: {file_cycles:?}"
    );

    let project_cycles = report["analysis"]["stats"]["project_cycles"]
        .as_array()
        .unwrap();
    assert!(
        project_cycles.iter().any(|cycle| {
            let names: Vec<&str> = cycle
                .as_array()
                .unwrap()
                .iter()
                .map(|n| n.as_str().unwrap())
                .collect();
            names == vec!["feature-x", "feature-y"]
        }),
        "feature-x ⇄ feature-y project cycle should be detected, got: {project_cycles:?}"
    );
}

#[test]
fn f13_boundary_violations_detected() {
    let report = run_fixture("f13-boundaries");

    let violations = report["analysis"]["boundary_violations"]
        .as_array()
        .unwrap();
    assert_eq!(violations.len(), 2, "got: {violations:?}");

    assert_eq!(violations[0]["from"], "feature-shop");
    assert_eq!(violations[0]["to"], "feature-admin");
    assert_eq!(violations[0]["source_tag"], "scope:shop");

    assert_eq!(violations[1]["from"], "ui-kit");
    assert_eq!(violations[1]["to"], "feature-shop");
    assert_eq!(violations[1]["source_tag"], "type:ui");
}

#[test]
fn f01_stats_matrix_counts_symbols() {
    let report = run_fixture("f01-basic-imports");

    let deps = report["analysis"]["stats"]["dependencies"]
        .as_array()
        .unwrap();
    let feature_to_ui = deps
        .iter()
        .find(|dep| dep["from"] == "feature-a" && dep["to"] == "ui")
        .expect("feature-a -> ui dependency should exist");

    assert_eq!(feature_to_ui["count"], 2);
    let symbols: Vec<&str> = feature_to_ui["symbols"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| s["name"].as_str().unwrap())
        .collect();
    assert_eq!(symbols, vec!["ButtonConfig", "UiButtonComponent"]);

    let projects = report["analysis"]["stats"]["projects"].as_array().unwrap();
    let util = projects
        .iter()
        .find(|p| p["name"] == "util")
        .expect("util project in stats");
    assert_eq!(util["afferent"], 0);
    assert_eq!(util["efferent"], 0);
}

fn run_cli(fixture: &str, extra_args: &[&str]) -> (i32, String, String) {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let output = Command::new(env!("CARGO_BIN_EXE_nx-analyzer"))
        .current_dir(manifest_dir)
        .arg("-d")
        .arg(format!("tests/fixtures/{fixture}"))
        .args(extra_args)
        .output()
        .expect("failed to run nx-analyzer");
    (
        output.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&output.stdout).to_string(),
        String::from_utf8_lossy(&output.stderr).to_string(),
    )
}

#[test]
fn cli_fail_on_unused_exits_2() {
    let (code, _, stderr) = run_cli("f07-unused-code", &["unused", "--fail-on", "unused"]);
    assert_eq!(code, 2, "stderr: {stderr}");
    assert!(stderr.contains("unused:"));
}

#[test]
fn cli_fail_on_respects_baseline() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let out_dir = Path::new(manifest_dir).join("target").join("test-output");
    std::fs::create_dir_all(&out_dir).unwrap();
    let baseline = out_dir.join(format!("baseline-{}.json", std::process::id()));

    let (code, _, _) = run_cli(
        "f07-unused-code",
        &["baseline", "-o", baseline.to_str().unwrap()],
    );
    assert_eq!(code, 0);

    // Same findings + baseline → nothing new → exit 0.
    let (code, _, stderr) = run_cli(
        "f07-unused-code",
        &[
            "unused",
            "--baseline",
            baseline.to_str().unwrap(),
            "--fail-on",
            "all",
        ],
    );
    assert_eq!(code, 0, "stderr: {stderr}");
}

#[test]
fn cli_fail_on_cycles_only_ignores_unused() {
    // f07 has unused findings but no cycles — failing on cycles passes.
    let (code, _, _) = run_cli("f07-unused-code", &["unused", "--fail-on", "cycles"]);
    assert_eq!(code, 0);

    let (code, _, _) = run_cli("f09-circular-deps", &["cycles", "--fail-on", "cycles"]);
    assert_eq!(code, 2);
}

#[test]
fn cli_graph_mermaid_renders_project_edges() {
    let (code, stdout, _) = run_cli("f09-circular-deps", &["graph", "--format", "mermaid"]);
    assert_eq!(code, 0);
    assert!(stdout.contains("graph LR"));
    assert!(stdout.contains("feature_x -->|1| feature_y"));
    assert!(stdout.contains("feature_y -->|1| feature_x"));
}

#[test]
fn cli_lazy_edges_are_dashed_in_mermaid() {
    let (code, stdout, _) = run_cli("f10-lazy-routes", &["graph", "--format", "mermaid"]);
    assert_eq!(code, 0, "got: {stdout}");
    assert!(stdout.contains("-. lazy .->"), "got: {stdout}");
}

#[test]
fn cli_sarif_is_valid_and_complete() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let out_dir = Path::new(manifest_dir).join("target").join("test-output");
    std::fs::create_dir_all(&out_dir).unwrap();
    let sarif_file = out_dir.join(format!("test-{}.sarif", std::process::id()));

    let (code, _, _) = run_cli(
        "f07-unused-code",
        &["sarif", "-o", sarif_file.to_str().unwrap()],
    );
    assert_eq!(code, 0);

    let sarif: Value =
        serde_json::from_str(&std::fs::read_to_string(&sarif_file).unwrap()).unwrap();
    assert_eq!(sarif["version"], "2.1.0");
    let results = sarif["runs"][0]["results"].as_array().unwrap();
    // 7 unused + 2 export-only + 1 not-rendered + 1 orphan.
    assert_eq!(results.len(), 11);
    assert!(results.iter().any(|r| r["ruleId"] == "unused-export"));
    assert!(results.iter().any(|r| r["ruleId"] == "export-only"));
    assert!(results
        .iter()
        .any(|r| r["ruleId"] == "declared-not-rendered"));
}

#[test]
fn cli_html_report_is_self_contained() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let out_dir = Path::new(manifest_dir).join("target").join("test-output");
    std::fs::create_dir_all(&out_dir).unwrap();
    let html_file = out_dir.join(format!("report-{}.html", std::process::id()));

    let (code, _, _) = run_cli(
        "f07-unused-code",
        &["html", "-o", html_file.to_str().unwrap()],
    );
    assert_eq!(code, 0);

    let html = std::fs::read_to_string(&html_file).unwrap();
    assert!(html.contains("DeadComponent"));
    assert!(html.contains("WiredNotRenderedComponent"));
    // Self-contained: no external resources.
    assert!(!html.contains("https://cdn"));
    assert!(!html.contains("<script src"));
    assert!(!html.contains("<link "));
}

#[test]
fn f15_react_components_detected() {
    let report = run_fixture("f15-react");

    let components: Vec<(&str, bool)> = report["react_components"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| (c["name"].as_str().unwrap(), c["wrapped"].as_bool().unwrap()))
        .collect();

    // Function components, memo-wrapped const, default export, plain apps.
    assert!(components.contains(&("Button", false)));
    assert!(components.contains(&("Card", true)), "memo() wrapper");
    assert!(components.contains(&("App", false)));
    assert!(components.contains(&("Settings", false)), "export default");
    assert!(components.contains(&("UnusedWidget", false)));
}

#[test]
fn f15_react_usage_and_prop_stats() {
    let report = run_fixture("f15-react");

    let usage = report["analysis"]["react_usage"].as_array().unwrap();
    let find_usage = |name: &str| {
        usage
            .iter()
            .find(|u| u["component"] == name)
            .unwrap_or_else(|| panic!("{name} in react_usage"))
    };

    // Button rendered twice in App with different props.
    let button = find_usage("Button");
    assert_eq!(button["usage_count"], 2);
    let props: Vec<(&str, i64)> = button["props"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| (p["name"].as_str().unwrap(), p["count"].as_i64().unwrap()))
        .collect();
    assert!(props.contains(&("variant", 2)));
    assert!(props.contains(&("size", 1)));
    assert!(props.contains(&("onClick", 1)));

    let card = find_usage("Card");
    assert_eq!(card["usage_count"], 1);

    let unused_widget = find_usage("UnusedWidget");
    assert_eq!(unused_widget["usage_count"], 0);
}

#[test]
fn f15_react_lazy_creates_dynamic_edge_and_unused_detects_widget() {
    let report = run_fixture("f15-react");

    // React.lazy(() => import('./settings')) — lazy edge.
    let app_facts = report["source_files"]
        .as_array()
        .unwrap()
        .iter()
        .find(|f| f["path"].as_str().unwrap().ends_with("app/app.tsx"))
        .expect("app.tsx analyzed");
    let dynamic = app_facts["dynamic_imports"].as_array().unwrap();
    assert_eq!(dynamic.len(), 1);
    assert!(dynamic[0]["resolved_path"]
        .as_str()
        .unwrap()
        .ends_with("app/settings.tsx"));

    // Settings must NOT be unused (lazy-loaded); UnusedWidget must be.
    let unused = unused_names(&report, "unused_exports");
    assert!(
        unused.contains(&"UnusedWidget".to_string()),
        "got: {unused:?}"
    );
    assert!(!unused.contains(&"Settings".to_string()), "got: {unused:?}");

    // JSX tag usage counts as symbol usage: Button/Card are not unused.
    assert!(!unused.contains(&"Button".to_string()));
    assert!(!unused.contains(&"Card".to_string()));
}

#[test]
fn cli_usages_shows_full_symbol_picture() {
    let (code, stdout, _) = run_cli("f06-templates", &["usages", "UiButtonComponent", "--json"]);
    assert_eq!(code, 0);

    let usage: Value = serde_json::from_str(&stdout).unwrap();
    let declaration = &usage["declarations"][0];
    assert_eq!(declaration["kind"], "Component");
    assert_eq!(declaration["project"], "ui");
    assert_eq!(declaration["total_usages"], 2);
    assert_eq!(declaration["by_project"]["page"], 2);

    let vias: Vec<&str> = declaration["usages"]
        .as_array()
        .unwrap()
        .iter()
        .map(|u| u["via"].as_str().unwrap())
        .collect();
    assert!(vias.contains(&"Import"));
    assert!(vias.contains(&"Template"));
}

#[test]
fn cli_usages_react_counts_jsx_renders() {
    let (code, stdout, _) = run_cli("f15-react", &["usages", "Button", "--json"]);
    assert_eq!(code, 0);

    let usage: Value = serde_json::from_str(&stdout).unwrap();
    let declaration = &usage["declarations"][0];
    assert_eq!(declaration["kind"], "ReactComponent");
    // 1 import + 2 JSX renders.
    assert_eq!(declaration["total_usages"], 3);
}

#[test]
fn cli_usages_from_filter_and_unknown_symbol() {
    let (code, stdout, _) = run_cli(
        "f08-move-candidate",
        &["usages", "formatDate", "--from", "feature-cart", "--json"],
    );
    assert_eq!(code, 0);
    let usage: Value = serde_json::from_str(&stdout).unwrap();
    // formatDate is used by checkout AND cart; --from narrows to cart only.
    assert_eq!(usage["declarations"][0]["total_usages"], 1);
    assert_eq!(usage["declarations"][0]["by_project"]["feature-cart"], 1);

    let (code, stdout, _) = run_cli("f08-move-candidate", &["usages", "NoSuchSymbol"]);
    assert_eq!(code, 0);
    assert!(stdout.contains("not found"));
}

#[test]
fn cli_unused_kind_filter_finds_dead_component() {
    let (code, stdout, _) = run_cli("f07-unused-code", &["unused", "--kind", "component"]);
    assert_eq!(code, 0);
    assert!(stdout.contains("DeadComponent"), "got: {stdout}");
    assert!(!stdout.contains("DeadService"), "got: {stdout}");
    assert!(!stdout.contains("deadUtil"), "got: {stdout}");

    let (code, stdout, _) = run_cli("f07-unused-code", &["unused", "--kind", "service,function"]);
    assert_eq!(code, 0);
    assert!(stdout.contains("DeadService"));
    assert!(stdout.contains("deadUtil"));
    assert!(!stdout.contains("DeadComponent"));
}

#[test]
fn cli_stats_project_filter() {
    let (code, stdout, _) = run_cli("f01-basic-imports", &["stats", "--project", "ui"]);
    assert_eq!(code, 0);
    assert!(stdout.contains("feature-a → ui"));
    assert!(
        !stdout.contains("\nutil "),
        "util row should be filtered out"
    );
}

#[test]
fn f16_nested_projects_are_attributed_correctly() {
    let report = run_fixture("f16-nested-projects");

    // Longest-root matching: files under libs/parent/nested belong to
    // `nested`, NOT `parent` — and must not be processed twice.
    let nested = find(&report, "services", "NestedService");
    assert_eq!(nested["package_name"], "nested");

    let parent = find(&report, "services", "ParentService");
    assert_eq!(parent["package_name"], "parent");

    let nested_count = report["services"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|s| s["name"] == "NestedService")
        .count();
    assert_eq!(nested_count, 1, "nested files must not be processed twice");

    // Stats: consumer depends on BOTH parent and nested separately.
    let deps = report["analysis"]["stats"]["dependencies"]
        .as_array()
        .unwrap();
    assert!(deps
        .iter()
        .any(|d| d["from"] == "consumer" && d["to"] == "nested"));
    assert!(deps
        .iter()
        .any(|d| d["from"] == "consumer" && d["to"] == "parent"));
}

#[test]
fn f17_circular_barrels_terminate_and_resolve() {
    let report = run_fixture("f17-barrel-cycles");

    // fromB reachable through barrel -> a -> b despite the a<->b cycle.
    let user = find(&report, "services", "BarrelUserService");
    let imports = resolved_imports(user);
    assert!(
        imports
            .iter()
            .any(|path| path.ends_with("libs/alpha/src/lib/b.ts")),
        "fromB should resolve through circular re-exports, got: {imports:?}"
    );
}

#[test]
fn f17_name_collisions_are_kept_apart() {
    let report = run_fixture("f17-barrel-cycles");

    // Two `Config` interfaces: alpha's is used, gamma's is dead.
    let unused = report["analysis"]["unused"]["unused_exports"]
        .as_array()
        .unwrap();
    let dead_configs: Vec<&str> = unused
        .iter()
        .filter(|s| s["name"] == "Config")
        .map(|s| s["file"].as_str().unwrap())
        .collect();
    assert_eq!(dead_configs.len(), 1, "got: {dead_configs:?}");
    assert!(dead_configs[0].contains("libs/gamma/"));
}

#[test]
fn f17_usages_reports_both_declarations() {
    let (code, stdout, _) = run_cli("f17-barrel-cycles", &["usages", "Config", "--json"]);
    assert_eq!(code, 0);
    let usage: Value = serde_json::from_str(&stdout).unwrap();
    let declarations = usage["declarations"].as_array().unwrap();
    assert_eq!(declarations.len(), 2, "both Config declarations listed");

    let (alpha, gamma) = if declarations[0]["project"] == "alpha" {
        (&declarations[0], &declarations[1])
    } else {
        (&declarations[1], &declarations[0])
    };
    assert_eq!(alpha["total_usages"], 1);
    assert_eq!(gamma["total_usages"], 0);
}

#[test]
fn f18_jsonc_tsconfig_and_js_extension_imports() {
    let report = run_fixture("f18-modern-syntax");

    let service = find(&report, "services", "EsmStyleService");
    let imports = resolved_imports(service);

    // './helper.js' (NodeNext style) must resolve to helper.ts —
    // and the paths alias comes from a tsconfig full of comments.
    assert!(
        imports
            .iter()
            .any(|path| path.ends_with("libs/modern/src/lib/helper.ts")),
        ".js specifier should resolve to .ts file, got: {imports:?}"
    );
    // `import type` counts as usage.
    assert!(
        imports
            .iter()
            .any(|path| path.ends_with("libs/modern/src/lib/model.ts")),
        "type-only import should resolve, got: {imports:?}"
    );

    // ModernModel used (type import) — not unused; UnusedTypeOnlyModel dead.
    let unused = unused_names(&report, "unused_exports");
    assert!(
        !unused.contains(&"ModernModel".to_string()),
        "got: {unused:?}"
    );
    assert!(
        unused.contains(&"UnusedTypeOnlyModel".to_string()),
        "got: {unused:?}"
    );
}

#[test]
fn f19_pipes_inside_control_flow_blocks_count_as_usage() {
    let report = run_fixture("f19-template-advanced");
    let usages = template_usages(&report);

    // @if (items | uiHas) and @for (... of items | uiSort) — pipes in block
    // conditions, not interpolations.
    assert!(
        usages.contains(&(
            "HostComponent".to_string(),
            "UiHasPipe".to_string(),
            "Pipe".to_string()
        )),
        "pipe in @if condition should count, got: {usages:?}"
    );
    assert!(
        usages.contains(&(
            "HostComponent".to_string(),
            "UiSortPipe".to_string(),
            "Pipe".to_string()
        )),
        "pipe in @for expression should count, got: {usages:?}"
    );
}

#[test]
fn f19_compound_selector_matches_only_button() {
    let report = run_fixture("f19-template-advanced");

    // `button[fixBtn]`: <button fixBtn> matches, <a fixBtn> must not —
    // exactly ONE selector usage of the directive.
    let count = report["template_usages"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|u| u["target"] == "FixBtnDirective")
        .count();
    assert_eq!(count, 1);
}

#[test]
fn f19_recursive_only_component_is_dead() {
    let report = run_fixture("f19-template-advanced");

    // OrphanTreeComponent renders itself in its own template and nothing
    // else references it — self-usage must NOT keep it alive.
    let unused = unused_names(&report, "unused_exports");
    assert!(
        unused.contains(&"OrphanTreeComponent".to_string()),
        "self-referencing orphan should be dead, got: {unused:?}"
    );
    // HostComponent is imported by main.ts — alive.
    assert!(!unused.contains(&"HostComponent".to_string()));
}

#[test]
fn f20_vercel_config_and_dist_artifacts_are_ignored() {
    let report = run_fixture("f20-next-conventions");

    // .vercel/project.json is Vercel config, not an NX project.
    let projects: Vec<&str> = report["analysis"]["stats"]["projects"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p["name"].as_str().unwrap())
        .collect();
    assert_eq!(
        projects,
        vec!["mobile", "web", "web-ui"],
        "got: {projects:?}"
    );

    // dist/ artifacts are never analyzed.
    let has_dist = report["source_files"]
        .as_array()
        .unwrap()
        .iter()
        .any(|f| f["path"].as_str().unwrap().contains("/dist/"));
    assert!(!has_dist, "dist artifacts must not be analyzed");
}

#[test]
fn f20_nextjs_conventions_are_entry_points() {
    let report = run_fixture("f20-next-conventions");
    let unused = unused_names(&report, "unused_exports");

    // page.tsx / layout.tsx / proxy.ts exports are consumed by the framework.
    for name in [
        "DashboardPage",
        "RootLayout",
        "PageProps",
        "LayoutProps",
        "metadata",
        "proxy",
        "config",
    ] {
        assert!(
            !unused.contains(&name.to_string()),
            "{name} is a Next.js convention export, got: {unused:?}"
        );
    }
    // Regular dead code in a lib is still found.
    assert!(
        unused.contains(&"TrulyDeadWidget".to_string()),
        "got: {unused:?}"
    );
    // WebPanel imported by a page — alive.
    assert!(!unused.contains(&"WebPanel".to_string()));
}

#[test]
fn f20_expo_router_and_ambient_files_are_not_dead() {
    let report = run_fixture("f20-next-conventions");
    let unused = unused_names(&report, "unused_exports");

    // expo-router: every file under app/ is a route.
    assert!(
        !unused.contains(&"HomeScreen".to_string()),
        "got: {unused:?}"
    );
    assert!(
        !unused.contains(&"MobileRootLayout".to_string()),
        "got: {unused:?}"
    );

    // Route files and .d.ts must not be orphans either.
    let orphans: Vec<&str> = report["analysis"]["unused"]["orphan_files"]
        .as_array()
        .unwrap()
        .iter()
        .map(|f| f.as_str().unwrap())
        .collect();
    assert!(
        !orphans
            .iter()
            .any(|f| f.contains("/app/") || f.ends_with(".d.ts")),
        "got: {orphans:?}"
    );
}

// ---------------------------------------------------------------------------
// f21: npm-workspace packages and re-exports through aliases — patterns from
// a real workspace where every one of these produced FALSE "unused" findings
// (verified by grepping the real project for the reported symbols).
// ---------------------------------------------------------------------------

#[test]
fn f21_workspace_package_resolves_through_symlink_and_manifest() {
    // '@fix/wpkg' is NOT in tsconfig paths. It must resolve through the
    // node_modules symlink (→ libs/wpkg), the package.json entry
    // (→ src/index.ts) and the barrel to the declaring files — spelled
    // workspace-relative, never through node_modules.
    let report = run_fixture("f21-workspace-packages");

    let consumer = find(&report, "react_components", "Consumer");
    let consumer_file = consumer["source_path"].as_str().unwrap();
    let imports: Vec<String> = report["source_files"]
        .as_array()
        .unwrap()
        .iter()
        .find(|f| f["path"] == consumer_file)
        .map(|f| {
            f["imports"]
                .as_array()
                .unwrap()
                .iter()
                .map(|i| i["resolved_path"].as_str().unwrap().to_string())
                .collect()
        })
        .unwrap();

    assert!(
        imports
            .iter()
            .any(|p| p.ends_with("libs/wpkg/src/lib/widget.ts")),
        "workspace-package import should reach the declaring file, got: {imports:?}"
    );
    assert!(
        !imports.iter().any(|p| p.contains("node_modules")),
        "workspace paths must not be spelled through node_modules: {imports:?}"
    );
}

#[test]
fn f21_workspace_package_symbols_are_not_unused() {
    let report = run_fixture("f21-workspace-packages");
    let unused = unused_names(&report, "unused_exports");

    // Imported from '@fix/wpkg' (workspace package, barrel re-exports).
    assert!(
        !unused.contains(&"widgetThing".to_string()),
        "got: {unused:?}"
    );
    // Barrel `export { default as DefaultWidget }` → default export chain.
    assert!(
        !unused.contains(&"DefaultWidget".to_string()),
        "got: {unused:?}"
    );
    // Reached via `export type { Rect } from '@fix/models'` in another lib.
    assert!(!unused.contains(&"Rect".to_string()), "got: {unused:?}");
}

#[test]
fn f21_snapshot() {
    let report = run_fixture("f21-workspace-packages");
    insta::assert_json_snapshot!("f21-workspace-packages", report);
}

/// A leftover import statement must not count as a usage of what it names.
/// Before this, `ghost` looked alive because a dead file imported it without
/// ever referencing it — the mechanism by which dead code props up dead code.
#[test]
fn f22_dead_import_does_not_keep_its_target_alive() {
    let report = run_fixture("f22-dead-imports");
    let unused = unused_names(&report, "unused_exports");

    assert!(
        unused.contains(&"ghost".to_string()),
        "ghost is only ever named by an unreferenced import in zombie.ts, got: {unused:?}"
    );
    assert!(
        unused.contains(&"zombie".to_string()),
        "zombie itself has no consumers, got: {unused:?}"
    );

    let dead_imports = &report["analysis"]["unused"]["unused_imports"];
    let dead: Vec<(String, String)> = dead_imports
        .as_array()
        .unwrap()
        .iter()
        .map(|item| {
            (
                item["name"].as_str().unwrap().to_string(),
                item["specifier"].as_str().unwrap().to_string(),
            )
        })
        .collect();
    assert_eq!(
        dead,
        vec![("ghost".to_string(), "./ghost".to_string())],
        "the leftover statement is the only dead import in the fixture"
    );
}

/// The other side of the filter: an import whose binding IS referenced still
/// counts. `Shape` is referenced only in a type position — if type identifiers
/// were missed, every type-only import in a real workspace would be reported
/// dead and the analysis would be worthless.
#[test]
fn f22_used_imports_still_count_including_type_positions() {
    let report = run_fixture("f22-dead-imports");
    let unused = unused_names(&report, "unused_exports");

    assert!(
        !unused.contains(&"helper".to_string()),
        "helper() is called in alive.ts, got: {unused:?}"
    );
    assert!(
        !unused.contains(&"Shape".to_string()),
        "Shape is used as a parameter type in typed.ts, got: {unused:?}"
    );
}

#[test]
fn f22_snapshot() {
    let report = run_fixture("f22-dead-imports");
    insta::assert_json_snapshot!("f22-dead-imports", report);
}

/// A side-effect import (`import '@fix/effects'`) binds no name. It is a real
/// edge — the package dependency matrix must show it — but it names no symbol,
/// so no analysis may emit an empty symbol name because of it.
#[test]
fn f22_cross_project_side_effect_import_is_an_edge_without_a_symbol_name() {
    let report = run_fixture("f22-dead-imports");

    let dependencies = report["analysis"]["stats"]["dependencies"]
        .as_array()
        .unwrap();

    let edge = dependencies
        .iter()
        .find(|dependency| dependency["from"] == "edge" && dependency["to"] == "effects")
        .unwrap_or_else(|| panic!("edge → effects dependency missing: {dependencies:?}"));

    for symbol in edge["symbols"].as_array().unwrap() {
        let name = symbol["name"].as_str().unwrap();
        assert!(
            !name.is_empty(),
            "side-effect import produced an empty symbol name"
        );
        assert_eq!(
            name, "*",
            "a module imported for effects only is a whole-module dependency"
        );
    }

    for candidate in report["analysis"]["move_candidates"].as_array().unwrap() {
        let name = candidate["symbol"].as_str().unwrap_or_default();
        assert!(!name.is_empty(), "empty symbol name in move candidates");
    }
}

/// The trust metric: a specifier pointing inside the workspace that fails to
/// resolve is a missing edge, and a missing edge is how a live symbol lands on
/// the dead list. It must be reported — and it must NOT be confused with an
/// uninstalled npm package, which is harmless.
#[test]
fn f01_unresolved_internal_imports_are_reported_apart_from_external_ones() {
    let report = run_fixture("f01-basic-imports");
    let resolution = &report["analysis"]["resolution"];

    let internal: Vec<String> = resolution["unresolved_internal"]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item["specifier"].as_str().unwrap().to_string())
        .collect();
    assert_eq!(
        internal,
        vec!["./does-not-exist".to_string()],
        "the deliberately broken relative import must surface as internal"
    );

    let external: Vec<String> = resolution["unresolved_external"]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item["specifier"].as_str().unwrap().to_string())
        .collect();
    assert!(
        external.contains(&"@angular/core".to_string()),
        "a bare specifier with no tsconfig alias and no node_modules is external, got: {external:?}"
    );
}
