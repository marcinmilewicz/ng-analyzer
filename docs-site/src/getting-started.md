# Getting Started

## Installation

### Prebuilt binaries

Every [release](https://github.com/marcinmilewicz/nx-analyzer/releases) ships archives for Linux
(x86_64, statically linked against musl), macOS (Apple Silicon and Intel) and Windows. Extract the
archive for your platform and put `nx-analyzer` on your `PATH`. Each archive is accompanied by a
`.sha256` checksum file.

### From crates.io

Requires a Rust toolchain (`rustup`):

```bash
cargo install nx-analyzer
```

### From source

```bash
git clone https://github.com/marcinmilewicz/nx-analyzer
cd nx-analyzer
cargo build --release
# binary at ./target/release/nx-analyzer
```

Optionally put it on your PATH:

```bash
cargo install --path .
```

## First run

Point the analyzer at your NX workspace root (the directory containing `nx.json` / `tsconfig.base.json`):

```bash
nx-analyzer -d /path/to/workspace analyze -o report.json
```

You will see each project being processed (with `-v`), and a `report.json` with the complete analysis. For a human-friendly view, generate the HTML report instead:

```bash
nx-analyzer -d /path/to/workspace html -o report.html
open report.html
```

The HTML report is a single self-contained file (no CDN, works offline): an interactive project graph plus tables for statistics, unused code, cycles, move candidates and boundary violations.

## Typical first questions

```bash
# What can I delete?
nx-analyzer -d . unused

# Only dead components in one library
nx-analyzer -d . unused --kind component --project shared-ui

# How coupled are my packages?
nx-analyzer -d . stats

# Who actually uses this component?
nx-analyzer -d . usages UiButtonComponent

# Do I have dependency cycles?
nx-analyzer -d . cycles
```

## What gets analyzed

- Every NX project found via `project.json` (name and `sourceRoot` are optional — inferred from the directory when missing).
- `.ts` and `.tsx` files by default; add `.js/.jsx/.mjs/.cjs` with `--typescript-only false`.
- `node_modules` is excluded by default (`--exclude-node-modules false` to include — rarely useful).
- tsconfig `paths` aliases are resolved through the full `extends` chain, including configs referenced from `node_modules`. Projects without a sibling `tsconfig.json` fall back to `tsconfig.lib.json`, `tsconfig.app.json`, then the workspace config.

## Angular version detection

Angular 19 changed the default of `standalone:` to `true`. nx-analyzer reads `@angular/core` from the workspace `package.json` and applies the correct default automatically — components without an explicit `standalone:` flag are treated as standalone on Angular ≥ 19.
