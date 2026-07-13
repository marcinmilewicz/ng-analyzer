# Architecture

nx-analyzer is a pipeline of pure, independently testable phases:

```
discover → parse → extract → resolve → connect → analyze → report
```

## Phases

**Discover** (`src/nx/`) — walk the workspace for `project.json` files; parse project config (name/sourceRoot optional, inferred from the directory), read tags, resolve the tsconfig `extends` chain recursively (relative paths and node_modules specifiers) merging `paths`/`baseUrl` with child-wins semantics. Fallback order for a project's tsconfig: `tsconfig.json` → `tsconfig.lib.json` → `tsconfig.app.json` → workspace `tsconfig.base.json`/`tsconfig.json`.

**Parse & extract** (`src/ng/visitors/`) — each file parsed once with SWC (TSX syntax by extension, decorators on). A single AST pass collects: imports (all specifier kinds), every export, dynamic `import()` calls, identifier/type references, Angular decorated classes with full metadata (in any export position), React function components and JSX usages.

**Resolve** (`src/analysis/resolvers/`) — import specifiers to files: relative paths, tsconfig `paths` aliases (exact and wildcard, `@`-prefixed or not, resolved against workspace root + `baseUrl`), node_modules walking upward. Barrel files are then followed (`find_export_declaration`) to the file that actually declares the symbol — with a shared parsed-module cache, so barrels are parsed once, not once per lookup. The import cache is keyed per importing directory for relative sources (two `./model` imports in different directories are distinct).

**Connect** (`src/ng/templates/`) — component templates (external + inline) scanned with a lightweight Angular-aware HTML tokenizer; selectors parsed and matched with CSS semantics; pipes extracted from interpolations and binding expressions. Matches become graph edges.

**Analyze** (`src/analyses/`) — pure functions over the collected facts:

- `stats` — project aggregation, Ca/Ce/instability, dependency matrix, project cycles (petgraph Tarjan SCC),
- `unused` — usage index (imports + templates + JSX + lazy + bootstrap), metadata-only detection, orphan files,
- `move_candidates`, `boundaries`, `react_usage`.

**Report** (`src/report/`) — terminal printers, Mermaid/DOT exporters, self-contained HTML, SARIF 2.1.0, baseline computation, per-symbol usage reports.

## Key design decisions

- **Determinism everywhere**: every collection in the output is explicitly sorted. Same input ⇒ byte-identical JSON. This enables snapshot testing and baseline diffs.
- **Conservative matching**: where semantics are ambiguous (`:not()` selectors, scope visibility), the analyzer over-matches rather than under-matches — dead-code reports must not have false positives.
- **Errors are loud but not fatal**: a file that fails to parse prints a warning and is skipped; the analysis continues.
- **Framework semantics as a layer**: the core graph (exports/imports/usages) is framework-agnostic; Angular and React are extractors on top. A future plugin contract will formalize this (see [Roadmap](./roadmap.md)).

## Concurrency

Files are processed in parallel chunks (rayon). Shared state is limited to concurrent caches (dashmap): resolved-import cache, parsed-module cache, file-content cache, and the import graph itself.
