# React

NX workspaces are often mixed — an Angular app next to React tools, or a React app next to shared TS libraries. nx-analyzer parses `.tsx`/`.jsx` with JSX enabled and applies the same graph semantics.

## Component detection

Recognized as React components (in `.tsx`/`.jsx` files):

```tsx
export function Button(props: ButtonProps) { … }          // capitalized function
export const Card = memo((props) => { … });               // memo/forwardRef wrapper
const Inner = () => <div />;                              // capitalized const arrow
export default function Settings() { … }                  // default export
```

Wrapped components (`memo`, `forwardRef`, including `React.memo`) are marked `"wrapped": true` in the report.

## JSX usage edges

Every capitalized JSX element is a usage:

```tsx
<Card elevated>
  <Button variant="primary" size="lg" onClick={buy}>Buy</Button>
  <Button variant="ghost">Cancel</Button>
</Card>
```

- the tag resolves through the file's imports to the declaring file (same-file components resolve locally),
- each render counts separately (`Button` above: 2 usages),
- lowercase tags (`<div>`, `<button>`) are DOM elements and are ignored.

JSX usage keeps components alive in the [unused analysis](../analyses/unused.md) and is listed by [`usages`](../analyses/usages.md) with the `jsx` mechanism.

## Prop usage statistics

The report aggregates which props are actually passed, per component — the data design-system teams use to measure adoption (à la react-scanner / Omlet):

```json
{
  "component": "Button",
  "package_name": "react-ui",
  "usage_count": 2,
  "props": [
    { "name": "variant", "count": 2 },
    { "name": "size", "count": 1 },
    { "name": "onClick", "count": 1 }
  ]
}
```

Available under `analysis.react_usage` in the full report. A component with `usage_count: 0` is dead — it will also show up in `unused_exports` with kind `ReactComponent`.

## React.lazy

```tsx
const LazySettings = lazy(() => import('./settings'));
```

The dynamic import creates a lazy edge — lazily-loaded pages are reachable, not dead.

## Current limitations

- Component detection is heuristic (capitalized function/const in a JSX file) — it does not verify the return type. In practice this matches how React code is written.
- Class components are not detected (rare in modern codebases); they still participate in the generic export/import analysis.
- `<Foo.Bar>` member-expression tags are not yet resolved.
