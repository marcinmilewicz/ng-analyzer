# Roadmap & FAQ

## Delivered

- **M0** — correctness fixes (14 bugs), fixture/test infrastructure, CI
- **M1** — full symbol graph: all exports/imports, TSX/JS support, AST caching, extended Angular metadata
- **M2** — Angular semantics: template analysis, lazy routes, DI
- **M3** — analyses: unused, stats/coupling, cycles, move candidates, boundaries
- **M4** — reporting: CLI subcommands, Mermaid/DOT, HTML, SARIF, baseline/`--fail-on`, `usages`, filters
- **M6-lite** — React: component detection, JSX usage edges, `React.lazy`, prop statistics

## Planned

- **M5** — incremental on-disk cache (file hash → facts), `--watch` mode, `--affected --base=<ref>`, git-churn hotspot overlay for the HTML graph
- **M6 full** — formal plugin contract separating the framework-agnostic core from Angular/React extractors
- **M7** — MCP server mode (`nx-analyzer serve --mcp`) so AI agents can query the live graph ("who uses X?", "what can I delete?"); Angular input/output usage analytics (design-system adoption for Angular — no tool on the market does this today)

## FAQ

**Why not just use knip?**
knip is excellent for generic JS/TS, but it has no framework semantics: a Angular component used only via its selector in a template, or a feature loaded only through `loadChildren`, looks dead to it. nx-analyzer's usage graph includes templates, DI, routes and JSX.

**Why Rust?**
Whole-workspace parsing and graph analysis in milliseconds, one static binary in CI, no Node toolchain requirement for the analyzer itself.

**Can it delete the dead code for me?**
Not yet — findings are reports. Auto-fix (`--fix`) is deliberately deferred until the detection has earned trust on real workspaces.

**Does `-p/--projects` filter the output?**
No — it restricts the *analysis input* (which projects are parsed). For output filtering use per-command flags: `unused --project/--kind`, `stats --project`, `move-candidates --project`, `usages --from`.

**How do I exclude a symbol from unused detection?**
Add it to the baseline (`nx-analyzer baseline`). A dedicated ignore-annotation mechanism is under consideration.

**Is the JSON output stable across runs?**
Yes — every collection is deterministically sorted; identical input produces byte-identical output. Safe for diffing, snapshots and caching.
