# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2026-07-19

First public release.

### Added

- **NX workspace discovery**: `project.json` with inferred `name`/`sourceRoot` and tags,
  recursive tsconfig `extends` resolution, fallback to `tsconfig.lib.json` / `tsconfig.app.json`.
- **Framework-agnostic symbol graph**: exports/imports per file (aliases, namespaces, defaults,
  re-exports, `export * as`), barrel following, dynamic `import()` as lazy edges, identifier and
  type references. Supports `.ts`, `.tsx`, and `.js/.jsx/.mjs/.cjs`.
- **Angular semantics**: components, directives, pipes, services and NgModules; template analysis
  (element/attribute/structural selector matching, pipes, `@if`/`@for` and `*ngIf`); lazy routes
  via `loadChildren`/`loadComponent`.
- **React support** (basic): function components in `.tsx` including `memo`/`forwardRef`, JSX usage
  edges, `React.lazy()`, per-component prop usage statistics.
- **Analyses**: `unused` (reachability-aware dead code, unused imports, orphan files),
  `resolution` (import resolution trust metric, `--strict` exits 3), `stats` (coupling matrix with
  Ca/Ce and instability), `cycles` (Tarjan SCC at file and project level), `move-candidates`,
  and `boundaries` (NX tag rules from `nx-analyzer.json`).
- **Reporting**: deterministic JSON, Mermaid/DOT graph export, self-contained interactive HTML
  report, and SARIF 2.1.0 for GitHub code scanning.
- **CI integration**: `--baseline` to report only new findings and `--fail-on unused,cycles,boundaries`
  (exit 2).

[Unreleased]: https://github.com/marcinmilewicz/nx-analyzer/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/marcinmilewicz/nx-analyzer/releases/tag/v0.1.0
