# nx-analyzer - TypeScript NX Workspace Analysis Tool

[![CI](https://github.com/marcinmilewicz/nx-analyzer/actions/workflows/ci.yml/badge.svg)](https://github.com/marcinmilewicz/nx-analyzer/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/nx-analyzer.svg)](https://crates.io/crates/nx-analyzer)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

A Rust-based static analyzer for NX workspaces — Angular, React and Next.js: framework-aware semantics (Angular templates, DI and lazy routes; JSX renders; Next.js file conventions), a framework-agnostic symbol graph, dead-code detection, package statistics and architecture checks — as one fast native binary.

## Installation

Prebuilt binaries for Linux (x86_64, static/musl), macOS (Apple Silicon and Intel) and Windows are
attached to every [release](https://github.com/marcinmilewicz/nx-analyzer/releases). Download the
archive for your platform, extract it, and put `nx-analyzer` on your `PATH`.

With a Rust toolchain:

```bash
cargo install nx-analyzer
```

From source:

```bash
git clone https://github.com/marcinmilewicz/nx-analyzer
cd nx-analyzer
cargo install --path .
```

## Features

- **NX Workspace Support**
  - `project.json` with optional `name`/`sourceRoot` (inferred), tags
  - tsconfig `extends` chains resolved recursively (including node_modules specifiers)
  - fallback to `tsconfig.lib.json` / `tsconfig.app.json` / workspace config
- **Symbol graph** (framework-agnostic)
  - all exports/imports per file (aliases, namespaces, defaults, re-exports, `export * as`)
  - barrels (`index.ts`) followed to the declaring file
  - dynamic `import()` as lazy edges; identifier/type references (covers DI)
  - `.ts`, `.tsx` (JSX), `.js/.jsx/.mjs/.cjs` (with `--typescript-only false`)
- **Angular semantics**
  - components (selector, standalone incl. the Angular 19 default, `imports`, providers,
    inline templates, `styleUrl(s)`, signal `input()`/`output()`/`model()` + decorators),
    directives, pipes, services (`@Injectable()` without args too), NgModules (full metadata)
  - **template analysis**: element/attribute/structural selector matching, pipes,
    `@if/@for` and `*ngIf` alike — usage in HTML counts as a dependency edge
  - lazy routes (`loadChildren`/`loadComponent`) keep lazy features alive
- **React (basic)**
  - function components in `.tsx` (incl. `memo`/`forwardRef`), JSX usage edges,
    `React.lazy()`, **prop usage statistics** per component
- **Analyses**
  - `unused`: unused exports, test-only exports, Angular entities wired-up-but-never-rendered,
    **unused import statements**, orphan files — reachability-aware (templates, DI, lazy routes,
    bootstrap). An import whose binding is never referenced does not keep its target alive;
    `import * as ns` conservatively keeps every export of the target alive; `import './x'`
    counts as an edge
  - `resolution`: how many import specifiers failed to resolve. Every unresolved *internal*
    one is an edge missing from the graph, and a missing edge is how a live symbol lands on
    the dead list — so this is the trust metric for `unused`. Gate CI on `--strict` (exit 3)
    before gating on `--fail-on unused`. Unresolved *external* specifiers (an npm package
    that is not installed) are counted separately and are harmless
  - `stats`: package→package matrix with symbol counts, Ca/Ce coupling, instability
  - `cycles`: file-level and project-level (Tarjan SCC)
  - `move-candidates`: symbols used exclusively by one other project
  - `boundaries`: NX tag rules from `nx-analyzer.json`
- **Reporting**
  - JSON (deterministic), **Mermaid**/DOT graph export, self-contained **HTML report**
    (interactive project graph + tables), **SARIF 2.1.0** (GitHub code scanning)
  - `--baseline` (report only new findings) and `--fail-on unused,cycles,boundaries` (exit 2)

## CLI

```bash
nx-analyzer [OPTIONS] [COMMAND]

COMMANDS:
    analyze            Full analysis, JSON report (default) [-o file.json]
    stats              Package statistics and dependency matrix [--project X]
    unused             Dead code report [--project X] [--kind component,service,...]
    usages <SYMBOL>    Where and how a symbol is used: imports, templates, JSX,
                       lazy loads, counts per project [--from X] [--json]
    cycles             File and project dependency cycles
    move-candidates    Symbols worth moving to their only consumer [--project X]
    boundaries         NX tag boundary violations
    graph              Export graph: --format mermaid|dot|json --level project|file
    html               Self-contained HTML report [-o report.html]
    sarif              SARIF output [-o results.sarif]
    baseline           Write current findings as a baseline [-o baseline.json]

OPTIONS (global):
    -d, --project-path <PATH>            Workspace directory [default: .]
    -v, --verbose                        Verbose output
    -p, --projects <PROJECTS>            Filter projects (comma-separated)
    -n, --exclude-node-modules <BOOL>    [default: true]
    -t, --typescript-only <BOOL>         .ts/.tsx only; false adds .js/.jsx/.mjs/.cjs [default: true]
        --baseline <FILE>                Report/fail only on findings not in the baseline
        --fail-on <CATEGORIES>           unused, cycles, boundaries, all → exit code 2
        --strict                         Exit 3 if any import inside the workspace fails to
                                         resolve — the graph is then incomplete and the
                                         dead-code findings cannot be trusted
```

### Examples

```bash
# Full report
nx-analyzer -d /path/to/workspace analyze -o report.json

# Dead code, failing CI only on NEW findings.
# --strict first: an incomplete graph invalidates every finding below it.
nx-analyzer -d . baseline -o .nx-analyzer-baseline.json     # once, commit the file
nx-analyzer -d . --strict unused --baseline .nx-analyzer-baseline.json --fail-on unused

# Who uses this component, from where and how?
nx-analyzer -d . usages UiButtonComponent
nx-analyzer -d . usages formatPrice --from feature-checkout --json

# Dead components only, in one project
nx-analyzer -d . unused --kind component --project shared-ui

# Project graph for a PR description (renders on GitHub)
nx-analyzer -d . graph --format mermaid

# Interactive report
nx-analyzer -d . html -o report.html
```

### Boundary rules (`nx-analyzer.json` at the workspace root)

```json
{
  "boundaries": [
    { "sourceTag": "type:ui", "allowedTags": ["type:ui", "type:util"] },
    { "sourceTag": "scope:shop", "allowedTags": ["scope:shop", "scope:shared"] }
  ]
}
```

## Project Structure

```
src/
├── analysis/           # Core: processors, resolvers, import graph, file facts
├── analyses/           # Derived analyses: stats, unused, cycles, moves, boundaries, react
├── ng/                 # Angular: visitors, decorator analyzers, template scanner
├── nx/                 # NX workspace handling (project.json, tsconfig chains)
└── report/             # Outputs: terminal, mermaid/dot, HTML, SARIF, baseline
tests/
├── fixtures/           # 13 miniature NX workspaces (F01–F15)
└── fixtures_test.rs    # 39 integration tests + insta snapshots
docs/
├── PRD.md                    # Product requirements
├── IMPLEMENTATION_PLAN.md    # Roadmap M0–M7 with delivery status
├── FIXTURES.md               # Fixture specification
└── COMPETITIVE_ANALYSIS.md   # Market survey and direction rationale
```

## Documentation

Full documentation lives in [`docs-site/`](docs-site/) ([mdBook](https://rust-lang.github.io/mdBook/)) and is deployed to GitHub Pages by the `Docs` workflow on every push to `main` (enable Pages → "GitHub Actions" in the repo settings).

```bash
brew install mdbook            # or: cargo install mdbook
mdbook serve docs-site --open  # local preview with live reload
```

Contents: getting started, full CLI reference, semantics of every analysis (unused categories and their guarantees, coupling metrics, boundary rules), Angular/React/plain-TS coverage, CI recipes with baseline workflow, architecture and fixture guide.

## Development

```bash
cargo test          # unit + integration tests (fixtures under tests/fixtures)
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```

Snapshot tests use [insta](https://insta.rs). After an intentional output change, refresh with `INSTA_UPDATE=always cargo test` and review the diff.

## Roadmap

Delivered: M0 (correctness + tests), M1 (symbol graph), M2 (Angular semantics), M3 (analyses), M4 (reporting), M6-lite (React basics). Planned next (see `docs/IMPLEMENTATION_PLAN.md`): M5 incremental cache/watch/git hotspots, M7 MCP server + Angular input analytics.

## Contributing

Project is in active development; issues and PRs welcome once the repository is public.

## License

MIT
