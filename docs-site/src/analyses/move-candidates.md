# Move Candidates

Finds symbols that live in the wrong package: everything that uses them sits in exactly **one other project**, and the home project itself never touches them (barrel re-exports don't count as home usage).

```
📦 Move candidates (1):
  formatPrice : shared-utils → feature-checkout (2 uses, 0 internal) — libs/shared-utils/src/lib/format.ts
```

Moving `formatPrice` into `feature-checkout` removes one reason for the `feature-checkout → shared-utils` edge — and if it was the only reason, the whole package dependency disappears.

## Detection rules

A symbol qualifies when **all** of these hold:

1. it has at least one production usage (test-file usages are ignored entirely),
2. zero usages inside its home project,
3. all external usages come from exactly **one** project.

Symbols used by two or more external projects are *shared* — they live in the right place. Symbols with internal usages would need a re-import after moving, so they are skipped (v1 keeps the suggestion list high-precision).

## Filtering

```bash
nx-analyzer -d . move-candidates --project shared-utils   # from OR into shared-utils
```

## Interpreting results

- A long list of candidates from one `shared-*` package usually means the package became a dumping ground — consider splitting it along consumer lines.
- Before moving, check [`usages <symbol>`](./usages.md) for the exact call sites, and remember that moving a symbol changes its import path for the target project (the barrel of the new home).
