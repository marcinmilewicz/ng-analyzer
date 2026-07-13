# nx-analyzer

**nx-analyzer** is a fast, native static analyzer for [NX](https://nx.dev) workspaces. It parses your TypeScript sources (Angular, React, or plain TS/JS), builds a workspace-wide symbol graph, and answers the questions monorepo teams keep asking:

- **What is dead?** Exported symbols, components, services and whole files that nothing imports, renders, injects or lazy-loads.
- **Who uses what, and how much?** Package→package dependency matrix with per-symbol counts, coupling metrics, per-component usage breakdowns.
- **What is misplaced?** Symbols used exclusively by one other package — candidates for moving.
- **Is the architecture healthy?** Dependency cycles (file- and project-level) and NX tag boundary violations.

## Why another tool?

Existing tools each cover a slice:

| Tool | Covers | Missing |
|---|---|---|
| [knip](https://knip.dev) | unused code in JS/TS | no framework semantics — a component "used" only in an Angular template or route looks dead |
| [ngx-unused](https://github.com/wgrabowski/ngx-unused) | unused Angular classes | one analysis, slow (tsc-based), no graph/stats |
| Angular compiler (NG8113) | unused standalone imports | single-component scope only |
| [dependency-cruiser](https://github.com/sverweij/dependency-cruiser), [madge](https://github.com/pahen/madge) | module graphs, cycles | file level only, no symbols, no NX awareness |
| `nx graph` | project graph | project level only — can't tell you which *export* is dead |

nx-analyzer combines **NX awareness** (projects, tags, barrels as public API), **framework semantics** (Angular templates, DI, lazy routes; React JSX) and **symbol-level precision**, in a single Rust binary that analyzes a workspace in milliseconds.

## At a glance

```console
$ nx-analyzer -d ./my-workspace unused
🪦 Unused exports (5):
  DeadComponent [Component] — libs/stuff/src/lib/dead.component.ts (stuff)
  ...
🧟 Declared but never rendered (1):
  WiredNotRenderedComponent [Component] — libs/stuff/src/lib/wired.component.ts

$ nx-analyzer -d ./my-workspace usages UiButtonComponent
🔎 UiButtonComponent [Component] — declared in libs/ui/src/lib/button.component.ts (ui)
   Total usages: 12
   By project:
     feature-checkout ×7
     feature-cart ×5
```

## How it works

1. **Discover** — find NX projects (`project.json`), resolve tsconfig `extends` chains, read tags.
2. **Parse** — every `.ts`/`.tsx` file once, with [SWC](https://swc.rs); extract exports, imports, dynamic imports, identifier references, Angular decorators, React components.
3. **Resolve** — imports through tsconfig `paths` aliases and barrel files down to the declaring file.
4. **Connect** — scan Angular templates (selector matching, pipes) and JSX; add usage edges.
5. **Analyze** — reachability-aware unused detection, coupling stats, Tarjan SCC cycles, move candidates, tag boundaries.
6. **Report** — JSON, terminal, Mermaid/DOT, self-contained HTML, SARIF; with baseline support for CI.
