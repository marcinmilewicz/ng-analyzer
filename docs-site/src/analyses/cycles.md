# Dependency Cycles

Cycles are computed as **strongly connected components** (Tarjan's algorithm, iterative — safe on graphs of any size) at two levels:

## File-level cycles

Every group of files that can reach each other through imports:

```
🔁 File cycles (2):
  libs/tangle/src/lib/a.ts → libs/tangle/src/lib/b.ts → libs/tangle/src/lib/c.ts
```

Note the SCC semantics: two overlapping loops through a shared file are reported as **one** component, not two separate cycles — the whole group must be untangled together.

The file graph includes template-usage edges, so a cycle like *component A renders B, B imports A* is detected even though no TypeScript import goes one way.

## Project-level cycles

The same computation on the aggregated project graph:

```
🔄 Project cycles (1):
  feature-x ⇄ feature-y
```

Project cycles are almost always architectural bugs in an NX workspace — NX itself refuses to build them in many configurations.

## CI

```bash
nx-analyzer -d . cycles --fail-on cycles
```

With a baseline, only **new** cycles fail the build:

```bash
nx-analyzer -d . cycles --baseline .nx-analyzer-baseline.json --fail-on cycles
```

## Self-loops

A file importing itself (or a project depending on itself through re-export tricks) is reported as a single-element cycle.
