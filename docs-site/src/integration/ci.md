# CI Setup & Baseline Workflow

## The baseline mechanism

Brownfield workspaces have existing findings you can't fix today. The baseline freezes them so CI fails **only on new problems**:

```bash
# 1. Accept the current state (run once, commit the file)
nx-analyzer -d . baseline -o .nx-analyzer-baseline.json

# 2. In CI: fail only on findings not in the baseline
nx-analyzer -d . analyze --baseline .nx-analyzer-baseline.json --fail-on all
```

The baseline is a sorted set of finding keys (`unused:<file>:<symbol>`, `cycle:<files>`, `boundary:<from>-><to>:<tag>`, …) — diff-friendly in code review. When you fix old findings, regenerate it to ratchet the standard down.

`--fail-on` picks categories independently: `unused`, `cycles`, `boundaries`, or `all`. Exit code **2** signals findings (vs 1 for hard errors), so pipelines can distinguish policy failures from tool failures.

## GitHub Actions example

```yaml
name: architecture
on: [pull_request]

jobs:
  nx-analyzer:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Build nx-analyzer
        run: cargo install --git https://github.com/marcinek/nx-analyzer   # or download a release binary

      - name: Check for new dead code, cycles and boundary violations
        run: nx-analyzer -d . analyze
              --baseline .nx-analyzer-baseline.json
              --fail-on all

      - name: Upload SARIF to code scanning
        if: always()
        run: nx-analyzer -d . sarif -o results.sarif
      - uses: github/codeql-action/upload-sarif@v3
        if: always()
        with:
          sarif_file: results.sarif

      - name: HTML report as artifact
        if: always()
        run: nx-analyzer -d . html -o nx-analyzer-report.html
      - uses: actions/upload-artifact@v4
        if: always()
        with:
          name: nx-analyzer-report
          path: nx-analyzer-report.html
```

## Graph in the PR description

```yaml
      - name: Project graph comment
        run: |
          {
            echo '```mermaid'
            nx-analyzer -d . graph --format mermaid
            echo '```'
          } >> "$GITHUB_STEP_SUMMARY"
```

GitHub renders the Mermaid block directly in the job summary.

## Performance notes

Analysis is parallel (rayon) and each file is parsed exactly once. Small-to-mid workspaces complete in tens of milliseconds; the analyzer adds no meaningful time to CI. There is no daemon/watch mode yet (planned — see [Roadmap](../reference/roadmap.md)).
