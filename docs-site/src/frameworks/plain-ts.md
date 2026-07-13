# Plain TypeScript / JavaScript

The core of nx-analyzer is framework-agnostic. Utility libraries, model packages, node tooling — anything TypeScript in the workspace gets the full treatment without any framework semantics needed.

## What works out of the box

- **All export forms**: classes, functions, variables, interfaces, type aliases, enums, `export default`, re-exports (`export { X } from`), wildcard re-exports (`export * from`), namespace re-exports (`export * as ns from`).
- **All import forms**: named (with aliases — `import { A as B }` correctly tracks the original exported name), default, namespace, side-effect imports.
- **Dynamic `import()`** anywhere in the code — lazy edges.
- **Reference tracking**: identifier *and type* references to imported symbols (a type used only in a signature still counts as used).
- **Barrel resolution**: imports through `index.ts` chains resolve to the file that actually declares the symbol, however deep the re-export chain.

Every analysis applies: [unused exports](../analyses/unused.md), [statistics](../analyses/stats.md), [cycles](../analyses/cycles.md), [move candidates](../analyses/move-candidates.md), [boundaries](../analyses/boundaries.md), [usages](../analyses/usages.md).

## File extensions

By default only `.ts`/`.tsx` are analyzed. JavaScript sources join with one flag:

```bash
nx-analyzer -d . --typescript-only false unused
```

This adds `.js`, `.jsx`, `.mjs`, `.cjs`, `.mts`, `.cts`. (TS syntax is a superset — plain JS parses fine; `export =` assignments are ignored gracefully.)

## Entry points and test files

- `main.ts` / `main.tsx` / `polyfills.ts` are reachability roots — their exports are never reported unused.
- `*.spec.*`, `*.test.*` and `*_test.ts` are test files: their own exports are ignored, and symbols used *only* by them land in the separate `test_only_exports` category instead of `unused_exports`.

## Example

A pure-TS library in an NX workspace:

```ts
// libs/toolbox/src/lib/math.ts
export function add(a: number, b: number): number { … }        // used → fine
export function unusedMultiply(a: number, b: number) { … }     // ← unused_exports
export const PI_ISH = 3.14;                                    // ← unused_exports
export enum RoundingMode { Up, Down }                          // used via dynamic import → fine
```

```console
$ nx-analyzer -d . unused --project toolbox
🪦 Unused exports (2):
  PI_ISH [Variable] — libs/toolbox/src/lib/math.ts (toolbox)
  unusedMultiply [Function] — libs/toolbox/src/lib/math.ts (toolbox)
```
