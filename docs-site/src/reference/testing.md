# Testing & Fixtures

nx-analyzer is tested against **17 miniature NX workspaces** (`tests/fixtures/`), each targeting one analysis path, plus unit tests for the algorithmic cores. Every fixture run goes through the real binary end-to-end and asserts on the JSON report; full-report [insta](https://insta.rs) snapshots guard against regressions.

## Fixture map

| Fixture | Exercises |
|---|---|
| `f01-basic-imports` | basic resolution, per-directory import cache, phantom-edge prevention, stats matrix |
| `f02-barrel-exports` | 3-level `export *` chains, aliased re-exports (`export { Card as UiCard }`), `export * as ns` |
| `f03-tsconfig-paths` | 2-level `extends` chains, non-`@` aliases, multi-variant `paths`, projects without `tsconfig.json`, `project.json` without `name` |
| `f04-standalone-components` | Angular 19 standalone default, inline templates, `styleUrl`, signal `input()/output()/model()`, `imports:`/`providers:` arrays |
| `f05-ngmodule-classic` | full NgModule metadata, `@Injectable()` without arguments |
| `f06-templates` | selector matching (element/attribute/structural), pipes, `@if/@for` + `*ngIf`, imported-but-not-rendered detection |
| `f07-unused-code` | all four unused categories with counter-examples (template-only, inject-only, main-only, spec-only usage must NOT be flagged) |
| `f08-move-candidate` | move thresholds: single external consumer vs shared vs internally-used |
| `f09-circular-deps` | file SCC (a→b→c→a), project cycle (x⇄y) |
| `f10-lazy-routes` | `loadChildren`/`loadComponent` lazy edges keep features alive |
| `f11-di-providers` | `inject()`, constructor types, `useClass`, `InjectionToken` |
| `f12-edge-cases` | `export default class`, non-exported decorated class, `export { X }` after declaration, import aliases/namespaces/side-effects, syntax-error resilience, `export =` |
| `f13-boundaries` | tag rules, exactly-two-violations assertion |
| `f14-plain-ts` | framework-free library: export kinds, `used_import_names`, dynamic import |
| `f15-react` | component detection (fn/memo/default), JSX usage & prop counts, `React.lazy`, dead React component |
| `f16-nested-projects` | a project rooted inside another project: correct file attribution, no double-processing, separate stats edges |
| `f17-barrel-cycles` | circular `export *` chains (termination + resolution through the cycle), same-named symbols in different projects kept apart |
| `f18-modern-syntax` | JSONC tsconfig (comments, trailing commas), NodeNext-style `./x.js` specifiers resolving to `.ts`, `import type` as usage |
| `f19-template-advanced` | pipes inside `@if`/`@for` conditions, compound selectors (`button[fixBtn]`), recursive self-only component correctly reported dead |

## Running

```bash
cargo test                       # everything
cargo test --test fixtures_test  # integration only
INSTA_UPDATE=always cargo test   # refresh snapshots after an intentional output change
```

CI additionally enforces `cargo fmt --check` and `cargo clippy --all-targets -- -D warnings`.

## Writing a new fixture

1. Create `tests/fixtures/fNN-name/` with `nx.json`, `package.json`, `tsconfig.base.json` and `libs/*/project.json` + sources — the smallest workspace that exhibits the behavior.
2. Add targeted assertions in `tests/fixtures_test.rs` (the `run_fixture` helper runs the binary and parses the report).
3. Include **counter-examples**: things that must *not* be flagged are as important as things that must.
4. Keep expectations deterministic — output collections are sorted, so exact `assert_eq!` on vectors is safe.
