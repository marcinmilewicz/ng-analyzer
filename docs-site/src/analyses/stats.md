# Package Statistics & Coupling

`nx-analyzer stats` aggregates the symbol graph to the project level.

## Project table

| Metric | Meaning |
|---|---|
| `files` | analyzed source files in the project |
| `exports` | exported symbols (its API surface, including internals re-exported by barrels) |
| `Ca` (afferent) | how many projects **depend on** this one |
| `Ce` (efferent) | how many projects this one **depends on** |
| `I` (instability) | `Ce / (Ca + Ce)` — Robert Martin's metric; 0 = maximally stable (everyone depends on it, it depends on nothing), 1 = maximally unstable |

Rules of thumb:

- High `Ca` + high `I` is a red flag: many packages depend on something that itself churns with its own dependencies.
- `Ca = 0` on a library means **nothing in the workspace uses it** — check `unused` and consider deleting or extracting it.
- Stable abstractions: your `util`/`model` packages should trend toward `I = 0`.

## Dependency matrix

Every cross-project edge with reference counts and the exact symbols:

```
🔗 Dependencies (package → package):
  feature-checkout → shared-utils (3 refs)
      formatDate ×1
      formatPrice ×2
  shop → feature-lazy (1 refs) [lazy]
      * ×1
```

Counts combine three mechanisms: static imports, Angular template usages and lazy `import()` (marked `[lazy]`, symbol `*`). This is the data behind [move candidates](./move-candidates.md) and [boundaries](./boundaries.md).

## Project cycles

Cycles at the package level are listed here too (and in [`cycles`](./cycles.md)):

```
🔄 Project cycles:
  feature-x ⇄ feature-y
```

## Filtering

```bash
nx-analyzer -d . stats --project shared-ui   # only rows involving shared-ui
```

## JSON shape

In the full report (`analyze`), this lives under `analysis.stats`:

```json
{
  "projects": [ { "name": "ui", "files": 3, "exports": 4, "afferent": 1, "efferent": 0, "instability": 0.0, "tags": ["type:ui"], "project_type": "library" } ],
  "dependencies": [ { "from": "feature-a", "to": "ui", "count": 2, "lazy": false, "symbols": [ { "name": "UiButtonComponent", "count": 1 } ] } ],
  "project_cycles": [ ["feature-x", "feature-y"] ]
}
```
