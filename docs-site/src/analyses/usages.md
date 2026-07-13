# Symbol Usages

`usages <SYMBOL>` answers "who uses this, from where, and how?" for a single symbol — the drill-down companion to the aggregate [statistics](./stats.md).

```console
$ nx-analyzer -d . usages UiButtonComponent
🔎 UiButtonComponent [Component] — declared in libs/ui/src/lib/button.component.ts (ui)
   Total usages: 12
   By project:
     feature-cart ×5
     feature-checkout ×7
   Usages:
     [import]   libs/feature-cart/src/lib/cart.component.ts
     [template] libs/feature-cart/src/lib/cart.component.ts
     [import]   libs/feature-checkout/src/lib/summary.component.ts
     ...
```

## Usage mechanisms

| Tag | Meaning |
|---|---|
| `import` | static import resolved (through any barrel chain) to the declaring file |
| `template` | Angular template renders it — selector match or pipe usage |
| `jsx` | React JSX render (each `<Button />` occurrence counts) |
| `lazy` | dynamic `import()` of the declaring file (routes, `React.lazy`) |
| `[test]` suffix | the usage comes from a test file |

## Options

```bash
nx-analyzer -d . usages formatPrice --from feature-checkout   # one consumer project only
nx-analyzer -d . usages Button --json                         # machine-readable
```

The JSON shape:

```json
{
  "symbol": "Button",
  "declarations": [
    {
      "file": "libs/react-ui/src/lib/button.tsx",
      "project": "react-ui",
      "kind": "ReactComponent",
      "total_usages": 3,
      "by_project": { "webapp": 3 },
      "usages": [
        { "file": "apps/webapp/src/app/app.tsx", "project": "webapp", "via": "Import", "test": false },
        { "file": "apps/webapp/src/app/app.tsx", "project": "webapp", "via": "Jsx", "test": false }
      ]
    }
  ]
}
```

Multiple declarations with the same name (e.g. two `Button`s in different packages) are reported as separate entries, each with its own usage list.

## Related: React prop statistics

For React components the full report additionally carries per-prop usage counts (design-system adoption data) under `analysis.react_usage`:

```json
{
  "component": "Button",
  "usage_count": 2,
  "props": [
    { "name": "variant", "count": 2 },
    { "name": "size", "count": 1 },
    { "name": "onClick", "count": 1 }
  ]
}
```
