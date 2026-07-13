# CLI Reference

```
nx-analyzer [GLOBAL OPTIONS] [COMMAND]
```

## Global options

Available on every command:

| Option | Default | Description |
|---|---|---|
| `-d, --project-path <PATH>` | `.` | Workspace root directory |
| `-v, --verbose` | off | Progress and timing output |
| `-p, --projects <NAMES>` | all | Analyze only these projects (comma-separated). Note: this restricts the *analysis input*, which changes results — cross-project usages from excluded projects are not seen. |
| `-n, --exclude-node-modules <BOOL>` | `true` | Prune `node_modules` from the walk |
| `-t, --typescript-only <BOOL>` | `true` | `.ts`/`.tsx` only; `false` adds `.js/.jsx/.mjs/.cjs` |
| `--baseline <FILE>` | — | Report/fail only on findings **not** present in the baseline |
| `--fail-on <CATEGORIES>` | — | Exit with code 2 when new findings exist: `unused`, `cycles`, `boundaries`, `all` (comma-separated) |

## Exit codes

| Code | Meaning |
|---|---|
| 0 | success, no policy failures |
| 1 | error (bad arguments, unreadable workspace) |
| 2 | `--fail-on` matched at least one (new) finding |

---

## `analyze` (default)

Full analysis; writes the complete JSON report.

```bash
nx-analyzer -d . analyze -o report.json
```

The report contains: `components`, `directives`, `pipes`, `services`, `modules`, `react_components`, `source_files` (per-file facts), `template_usages`, `import_graph` (edges + file cycles) and `analysis` (stats, unused, move candidates, boundary violations, react usage).

## `stats`

Package statistics and the dependency matrix.

```bash
nx-analyzer -d . stats
nx-analyzer -d . stats --project shared-ui     # rows involving one project
```

Columns: files, exports, **Ca** (afferent coupling — how many projects depend on it), **Ce** (efferent — how many it depends on), **I** (instability = Ce/(Ca+Ce)).

## `unused`

Dead-code report in four categories (see [Unused Code](./analyses/unused.md)).

```bash
nx-analyzer -d . unused
nx-analyzer -d . unused --project shared-ui
nx-analyzer -d . unused --kind component,pipe
nx-analyzer -d . unused --baseline .baseline.json --fail-on unused
```

`--kind` accepts (case-insensitive): `component`, `directive`, `pipe`, `service`, `module`, `reactcomponent`, `class`, `function`, `variable`, `interface`, `typealias`, `enum`, `default`.

## `usages <SYMBOL>`

Everything about one symbol: where it is declared and every place it is referenced, classified by mechanism.

```bash
nx-analyzer -d . usages UiButtonComponent
nx-analyzer -d . usages formatPrice --from feature-checkout
nx-analyzer -d . usages Button --json
```

Usage mechanisms: `import` (static import), `template` (Angular HTML selector/pipe match), `jsx` (React render), `lazy` (dynamic `import()` of the declaring file). Test-file usages are marked `[test]`. `--from <project>` narrows to usages originating in one project; `--json` prints the machine-readable structure.

## `cycles`

File-level and project-level dependency cycles (strongly connected components).

```bash
nx-analyzer -d . cycles --fail-on cycles
```

## `move-candidates`

Symbols whose only consumers live in a single other project (and that are not used at home).

```bash
nx-analyzer -d . move-candidates
nx-analyzer -d . move-candidates --project shared-utils
```

## `boundaries`

NX tag rule violations. Rules live in `nx-analyzer.json` at the workspace root — see [Boundary Rules](./analyses/boundaries.md).

```bash
nx-analyzer -d . boundaries --fail-on boundaries
```

## `graph`

Dependency graph export.

```bash
nx-analyzer -d . graph --format mermaid                  # project level, renders on GitHub
nx-analyzer -d . graph --format dot | dot -Tsvg > g.svg  # Graphviz
nx-analyzer -d . graph --format dot --level file         # file-level graph
nx-analyzer -d . graph --format json --level file        # raw edges + cycles
```

Lazy (dynamic-import) edges are dashed in Mermaid/DOT.

## `html`

Self-contained interactive HTML report — a single file with the project graph (click a node to highlight its edges), statistics, unused code, cycles, move candidates and violations.

```bash
nx-analyzer -d . html -o report.html
```

## `sarif`

SARIF 2.1.0 output for GitHub code scanning and compatible tools. Rules: `unused-export`, `declared-not-rendered`, `orphan-file`, `circular-dependency`, `boundary-violation`.

```bash
nx-analyzer -d . sarif -o results.sarif
```

## `baseline`

Snapshot current findings so CI fails only on **new** problems (brownfield adoption).

```bash
nx-analyzer -d . baseline -o .nx-analyzer-baseline.json   # commit this file
nx-analyzer -d . unused --baseline .nx-analyzer-baseline.json --fail-on all
```
