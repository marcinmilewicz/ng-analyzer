#!/usr/bin/env python3
"""Cross-check ng-analyzer findings against grep — the accuracy harness.

The analyzer only parses what its file filter admits (.ts/.tsx by default) and
only follows edges its resolver can resolve. Grep has neither limit, so it is
the independent witness: anything the analyzer calls dead but grep finds
referenced is a SUSPECT false positive.

Three checks, in decreasing order of how much a false positive costs trust:

  unused_exports  a symbol nobody imports, renders, lazy-loads or injects
  orphan_files    a file with no inbound edge at all — grep here also sweeps
                  .js/.mjs/.cjs, which the analyzer skips under its default
                  --typescript-only, so it catches files kept alive by an
                  entry point the analyzer never opened (RN `index.js`,
                  webpack configs, scripts)
  unused_imports  an import statement whose binding is never referenced; the
                  binding must appear on exactly one line, the import itself

Usage:
    scripts/verify-unused-with-grep.py <analysis.json> [workspace]

Exit code is always 0 — this reports, it does not gate. The counts block at
the end is stable output, meant to be committed as a regression reference so
the next change to the liveness rules can be diffed against it.
"""
import json
import re
import subprocess
import sys
from collections import defaultdict
from pathlib import Path

ANALYSIS = sys.argv[1] if len(sys.argv) > 1 else "analysis2.json"
WORKSPACE = Path(sys.argv[2] if len(sys.argv) > 2 else "/Users/marcinek/dev/easy-trackly")

data = json.load(open(ANALYSIS))
unused = data["analysis"]["unused"]
resolution = data["analysis"]["resolution"]

# Grep matches names, not symbols, and a name says almost nothing on its own:
# `Divider` is declared in a lib AND in the RN app, `colors` is a local const
# in a dozen files, `CardContent` is a prop somewhere. Grading every hit as a
# suspect drowns the harness — and a harness that cries wolf cannot measure
# anything.
#
# The precise question is narrower: is the symbol imported FROM THE FILE THAT
# DECLARES IT? That, and only that, is proof the analyzer is wrong. Everything
# else is a same-named something-else.
#
# A file declares a name when it exports it without forwarding (`from_module`
# is absent) — which covers `class X {}` and the `class X {}; export { X }`
# spelling alike.
declaring_files = defaultdict(set)
for source_file in data.get("source_files", []):
    for export in source_file.get("exports", []):
        if export.get("from_module") is None:
            declaring_files[export["name"]].add(source_file["path"])

SOURCE_GLOBS = ["*.ts", "*.tsx", "*.js", "*.jsx", "*.mts", "*.cts", "*.mjs", "*.cjs"]
GREP_ARGS = [
    "grep", "-Frnw",
    *[f"--include={glob}" for glob in SOURCE_GLOBS],
    "--exclude-dir=node_modules", "--exclude-dir=dist", "--exclude-dir=.next",
    "--exclude-dir=coverage", "--exclude=*.d.ts",
]


def rg(needle, fixed=True):
    """All word-boundary hits of `needle` in workspace sources."""
    args = GREP_ARGS if fixed else [a for a in GREP_ARGS if a != "-Frnw"] + ["-rn"]
    proc = subprocess.run(
        [*args, "-e", needle, str(WORKSPACE)], capture_output=True, text=True
    )
    hits = []
    for line in proc.stdout.splitlines():
        try:
            path, lineno, text = line.split(":", 2)
        except ValueError:
            continue
        hits.append((Path(path), int(lineno), text.strip()))
    return hits


def is_comment(text):
    stripped = text.lstrip()
    return stripped.startswith("//") or stripped.startswith("*") or stripped.startswith("/*")


def abs_path(raw):
    return Path(raw).resolve() if raw.startswith("/") else (WORKSPACE / raw).resolve()


SPECIFIER = re.compile(r"""(?:\bfrom\s*|\bimport\s*\(?\s*|\brequire\(\s*)['"]([^'"]+)['"]""")
STATEMENT_START = re.compile(r"^\s*(import|export)\b")

_file_lines = {}


def statement_kind(path, lineno):
    """'import', 'export' or None for the statement containing `lineno`.

    A multi-line block ends on `} from './x'`, which is identical whether the
    statement opened with `import {` or `export {` — and the two mean opposite
    things here (one consumes the symbol, the other only forwards it). So walk
    back to the line that opened the statement instead of guessing.
    """
    if path not in _file_lines:
        try:
            _file_lines[path] = path.read_text().splitlines()
        except OSError:
            _file_lines[path] = []
    lines = _file_lines[path]

    for n in range(min(lineno, len(lines)) - 1, -1, -1):
        match = STATEMENT_START.match(lines[n])
        if match:
            return match.group(1)
    return None


def specifier_targets(spec, importing_file, target):
    """Does `spec`, written in `importing_file`, name the file `target`?

    Relative specifiers are resolved for real — matching on the file stem
    alone makes every `from '../types'` in the repo look like a hit on every
    `types.ts`, which buries the signal. Bare specifiers are matched on their
    subpath tail, enough to catch deep imports into a package. Barrel imports
    (`@org/lib`) cannot be attributed without the tsconfig and are left out:
    this check is for proving the analyzer WRONG, so it must never guess.
    """
    if spec.startswith("."):
        base = (importing_file.parent / spec).resolve()
        candidates = [base]
        candidates += [base.with_suffix(s) for s in (".ts", ".tsx", ".js", ".jsx", ".mts", ".cts")]
        candidates += [base / f"index{s}" for s in (".ts", ".tsx", ".js")]
        return any(candidate == target for candidate in candidates)

    tail = "/".join(spec.split("/")[2:]) if spec.startswith("@") else None
    if not tail:
        return False
    target_str = str(target)
    return any(target_str.endswith(f"/{tail}{s}") for s in (".ts", ".tsx"))


# ---------------------------------------------------------------- resolution
# Findings are only as complete as the graph. Print this first: a non-zero
# internal count means the lists below have unknown false positives and no
# amount of grepping will tell you which.
print("== resolution health ==")
print(f"  resolved imports    : {resolution['resolved_imports']}")
print(f"  unresolved INTERNAL : {len(resolution['unresolved_internal'])}"
      f"{'  ⚠️  findings below are UNTRUSTWORTHY' if resolution['unresolved_internal'] else '  ✅'}")
for item in resolution["unresolved_internal"][:10]:
    print(f"      {item['file']} → {item['specifier']}")
external = sum(item["files"] for item in resolution["unresolved_external"])
print(f"  unresolved external : {external} import(s) in "
      f"{len(resolution['unresolved_external'])} package(s) — harmless")
print()


# ------------------------------------------------------------ unused_exports
# false_positive : the symbol is imported FROM ITS OWN DECLARING FILE — proof
#                  the analyzer is wrong. This is the number that matters and
#                  it must be zero.
# same_name_only : the name occurs elsewhere, but never as an import of THIS
#                  file — a same-named local, prop or symbol in another lib.
#                  Weak evidence; listed so a human can spot-check.
false_positive, same_name_only, confirmed, skipped = [], [], [], []

for item in unused["unused_exports"]:
    name = item["name"]
    declaring = abs_path(item["file"])

    # "default" is not a searchable identifier; 1-2 char names drown in noise.
    if name == "default" or len(name) < 3:
        skipped.append(item)
        continue

    hits = [
        (path, lineno, text)
        for path, lineno, text in rg(name)
        if path.resolve() != declaring and not is_comment(text)
    ]

    # A re-export (`export { X } from './X'` in a barrel) forwards the symbol,
    # it does not consume it — so it is NOT evidence of usage. A symbol a
    # barrel re-exports that nobody then imports is exactly what "unused
    # export" means, and treating the barrel line as a usage would make the
    # analysis unable to report anything reachable from a public API at all.
    # Only a real `import` proves the analyzer wrong.
    importing_this_file = [
        (path, lineno, text)
        for path, lineno, text in hits
        if statement_kind(path, lineno) == "import"
        and any(
            specifier_targets(spec, path.resolve(), declaring)
            for spec in SPECIFIER.findall(text)
        )
    ]

    if importing_this_file:
        false_positive.append((item, importing_this_file))
    elif hits:
        same_name_only.append((item, hits))
    else:
        confirmed.append((item, hits))

print(f"== unused_exports ({len(unused['unused_exports'])}) ==")
print(f"  confirmed dead : {len(confirmed)}  (name occurs nowhere else)")
print(f"  FALSE POSITIVE : {len(false_positive)}  ← imported from its own declaring file")
print(f"  same-name-only : {len(same_name_only)}  (name seen elsewhere, never imported from here)")
print(f"  skipped        : {len(skipped)}  (default / name shorter than 3 chars)")
print()

for item, hits in false_positive:
    print(f"  ❌ {item['name']} [{item['kind']}] — {item['file']} ({item['project']})")
    for path, lineno, text in hits[:4]:
        rel = str(path).replace(str(WORKSPACE) + "/", "")
        print(f"       {rel}:{lineno}: {text[:100]}")
if false_positive:
    print()


# ------------------------------------------------------------- orphan_files
# The high-value check: grep sweeps .js/.mjs/.cjs too. A file imported only
# from an entry the analyzer never parsed (React Native `index.js`, config
# scripts) looks orphaned to the analyzer and alive to grep.
orphan_confirmed, orphan_suspects = [], []

for raw in unused["orphan_files"]:
    path = abs_path(raw)

    hits = []
    for hit_path, lineno, text in rg(path.stem):
        if hit_path.resolve() == path or is_comment(text):
            continue
        for spec in SPECIFIER.findall(text):
            if specifier_targets(spec, hit_path.resolve(), path):
                hits.append((hit_path, lineno, text))
                break

    (orphan_suspects if hits else orphan_confirmed).append((raw, hits))

print(f"== orphan_files ({len(unused['orphan_files'])}) ==")
print(f"  confirmed orphan : {len(orphan_confirmed)}")
print(f"  SUSPECT          : {len(orphan_suspects)}  ← imported from a file the analyzer never parsed")
for raw, hits in orphan_suspects:
    print(f"     {raw}")
    for hit_path, lineno, text in hits[:3]:
        rel = str(hit_path).replace(str(WORKSPACE) + "/", "")
        print(f"         {rel}:{lineno}: {text[:100]}")
print()


# ----------------------------------------------------------- unused_imports
# The binding must occur exactly once in its file: on the import line itself.
import_confirmed, import_suspects = [], []

for item in unused.get("unused_imports", []):
    path = abs_path(item["file"])
    try:
        lines = path.read_text().splitlines()
    except OSError:
        continue
    pattern = re.compile(rf"\b{re.escape(item['name'])}\b")
    hits = [
        (n, line.strip())
        for n, line in enumerate(lines, 1)
        if pattern.search(line) and not is_comment(line)
    ]
    non_import = [(n, line) for n, line in hits if not re.match(r"\s*import\b", line)]
    (import_suspects if non_import else import_confirmed).append((item, non_import))

print(f"== unused_imports ({len(unused.get('unused_imports', []))}) ==")
print(f"  confirmed dead : {len(import_confirmed)}")
print(f"  SUSPECT        : {len(import_suspects)}  ← binding referenced outside its import line")
for item, hits in import_suspects:
    print(f"     {item['name']} from '{item['specifier']}' — {item['file']}")
    for lineno, line in hits[:3]:
        print(f"         :{lineno}: {line[:100]}")
print()


# ------------------------------------------------------------------ summary
# Stable, diffable block. Commit it; compare after every liveness change.
print("== counts (regression reference) ==")
for key in ("unused_exports", "test_only_exports", "export_only",
            "declared_not_rendered", "unused_imports", "orphan_files"):
    print(f"  {key:24s} {len(unused.get(key, []))}")
print(f"  {'resolved_imports':24s} {resolution['resolved_imports']}")
print(f"  {'unresolved_internal':24s} {len(resolution['unresolved_internal'])}")
print()
print("  -- accuracy (all three must be 0) --")
print(f"  {'unused_exports.FP':24s} {len(false_positive)}")
print(f"  {'orphan_files.FP':24s} {len(orphan_suspects)}")
print(f"  {'unused_imports.FP':24s} {len(import_suspects)}")
