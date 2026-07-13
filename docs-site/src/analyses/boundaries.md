# Boundary Rules

Tag-based architecture rules, conceptually compatible with [`@nx/enforce-module-boundaries`](https://nx.dev/docs/features/enforce-module-boundaries) — but enforced by a native binary with no ESLint run, and aware of template-usage edges.

## Configuration

Create `nx-analyzer.json` at the workspace root:

```json
{
  "boundaries": [
    { "sourceTag": "type:ui",    "allowedTags": ["type:ui", "type:util"] },
    { "sourceTag": "type:util",  "allowedTags": ["type:util"] },
    { "sourceTag": "scope:shop", "allowedTags": ["scope:shop", "scope:shared"] },
    { "sourceTag": "scope:admin","allowedTags": ["scope:admin", "scope:shared"] }
  ]
}
```

Tags come from each project's `project.json`:

```json
{ "name": "ui-kit", "tags": ["type:ui", "scope:shared"] }
```

## Semantics

For every cross-project dependency `A → B`:

- every rule whose `sourceTag` is among A's tags is checked **independently**;
- the rule passes when B has at least one tag from `allowedTags` (or `allowedTags` contains `"*"`);
- projects with no matching rule are unrestricted.

The typical NX two-dimension setup (a `type:` rule + a `scope:` rule per project) works exactly as in `enforce-module-boundaries`.

## Output

```
🚧 Boundary violations (2):
  feature-shop → feature-admin — tag `scope:shop` allows only [scope:shop, scope:shared], target has [type:feature, scope:admin]
  ui-kit → feature-shop — tag `type:ui` allows only [type:ui, type:util], target has [type:feature, scope:shop]
```

## CI

```bash
nx-analyzer -d . boundaries --fail-on boundaries
```

Violations are also included in [SARIF output](../integration/reports.md#sarif) (`boundary-violation` rule) and respect the [baseline](../integration/ci.md) mechanism, so legacy violations can be frozen while new ones fail the build.
