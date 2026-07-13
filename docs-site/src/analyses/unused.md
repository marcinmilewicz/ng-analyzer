# Unused Code

The headline analysis. It is **reachability-aware**: a symbol counts as used not only when imported, but also when it is rendered in an Angular template, matched by a selector, injected via DI, referenced as a type, lazy-loaded through a route, listed in a `bootstrap` array, or rendered in JSX. This is what separates nx-analyzer from generic dead-code tools — an Angular feature loaded only via `loadChildren` is *not* dead.

## Categories

### `unused_exports` — confidence: High

Exported symbols with **zero** references anywhere:

- no static import resolves to them (through any barrel chain),
- no Angular template uses their selector or pipe name,
- no JSX renders them,
- their file is not dynamically imported (directly or transitively through a lazy-loaded barrel),
- they are not in any NgModule `bootstrap` array.

The `kind` is framework-aware: a dead component reports as `Component`, not a generic `Class` — so `unused --kind component` does what you expect.

### `test_only_exports` — confidence: High

Symbols whose **only** consumers are test files (`*.spec.*`, `*.test.*`, `*_test.ts`). Not dead — but production code doesn't need them. Often extract-worthy into test utilities.

### `declared_not_rendered` — confidence: Medium

Angular entities that are **wired up but never rendered**: every production usage is a decorator-metadata reference (`imports: [...]` of a standalone component, or NgModule `declarations`/`imports`/`exports`/`providers`) and the selector/pipe never appears in any template. The workspace-wide generalization of Angular's own [NG8113](https://angular.dev/extended-diagnostics/NG8113) diagnostic.

Medium confidence because dynamic component creation (`ViewContainerRef.createComponent`, `ngComponentOutlet` with a variable) is invisible to static analysis. Review before deleting.

### `orphan_files`

Files with **no incoming edges at all**. Exclusions:

- entry points (`main.ts`, `main.tsx`, `polyfills.ts`),
- test files,
- barrels (`index.*`) — imports *through* a barrel resolve to declaring files, so barrels legitimately have no incoming edges,
- anything reachable from a dynamic import.

## What keeps a symbol alive

| Mechanism | Example |
|---|---|
| Static import | `import { X } from '@scope/lib'` (any alias/barrel chain) |
| Template selector | `<ui-button>`, `[uiTooltip]`, `*uiIf` |
| Template pipe | `{{ x \| uiCurrency }}` |
| DI / type reference | `inject(ApiService)`, `constructor(x: ApiService)`, `useClass: FileLogger`, `InjectionToken<Config>` |
| Lazy route | `loadChildren: () => import('@scope/feature')` |
| Bootstrap | `bootstrap: [AppComponent]` |
| JSX render | `<Button variant="primary" />` |
| `React.lazy` | `lazy(() => import('./settings'))` |

## Known limitations

- **Dynamic component creation** by variable reference is not tracked (flagged only at Medium confidence via `declared_not_rendered`).
- Selector matching is **workspace-global**, not scope-aware (NgModule/standalone visibility is not enforced). This is deliberately conservative: it can only cause *false negatives* (something kept alive), never a false "unused".
- String-based references (e.g. selectors built at runtime) are invisible, as in every static analyzer.

## CI usage

```bash
nx-analyzer -d . baseline -o .nx-analyzer-baseline.json    # accept current state
nx-analyzer -d . unused --baseline .nx-analyzer-baseline.json --fail-on unused
```

Exit code 2 means new dead code appeared since the baseline.
