use crate::report::FullReport;

/// Self-contained HTML report: project graph (SVG, circular layout, click to
/// filter) + filterable, sortable tables for stats, unused code, cycles,
/// move candidates and boundary violations. Filters (text / project / kind /
/// category) apply across every section, live in the URL hash and can be
/// driven from the graph. No external resources — one file, works offline.
pub fn to_html(report: &FullReport) -> Result<String, serde_json::Error> {
    let data = serde_json::to_string(&serde_json::json!({
        "stats": report.analysis.stats,
        "unused": report.analysis.unused,
        "moveCandidates": report.analysis.move_candidates,
        "boundaryViolations": report.analysis.boundary_violations,
        "fileCycles": report.import_graph.circular_dependencies,
    }))?;

    // The JSON is embedded inside <script>: a symbol name containing
    // "</script>" (or "<!--") in analyzed sources would otherwise break out
    // of the block (XSS). \uXXXX escapes are valid JSON, so the parsed
    // values are unchanged. U+2028/2029 are line separators that are legal
    // in JSON but not in JS string literals.
    let data = data
        .replace('&', "\\u0026")
        .replace('<', "\\u003c")
        .replace('>', "\\u003e")
        .replace('\u{2028}', "\\u2028")
        .replace('\u{2029}', "\\u2029");

    Ok(TEMPLATE.replace("__DATA__", &data))
}

const TEMPLATE: &str = r##"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>nx-analyzer report</title>
<style>
  :root { --fg:#1a1a2e; --muted:#6b7280; --line:#d1d5db; --accent:#4f46e5; --warn:#b45309; --bad:#b91c1c; --card:#ffffff; --bg:#f3f4f6; --chip:#eef2ff; }
  @media (prefers-color-scheme: dark) {
    :root { --fg:#e5e7eb; --muted:#9ca3af; --line:#374151; --accent:#818cf8; --warn:#f59e0b; --bad:#f87171; --card:#1f2937; --bg:#111827; --chip:#312e81; }
  }
  * { box-sizing: border-box; }
  body { font: 14px/1.5 -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif; color: var(--fg); background: var(--bg); margin: 0; padding: 24px; }
  h1 { font-size: 20px; margin: 0 0 12px; }
  h2 { font-size: 16px; margin: 0 0 8px; cursor: pointer; user-select: none; }
  h2 .count { color: var(--muted); font-weight: 400; font-size: 13px; }
  h2 .caret { display: inline-block; transition: transform .15s; color: var(--muted); font-size: 11px; }
  .card { background: var(--card); border: 1px solid var(--line); border-radius: 8px; padding: 16px; margin-bottom: 16px; overflow-x: auto; }
  .card.collapsed .body { display: none; }
  .card.collapsed .caret { transform: rotate(-90deg); }
  table { border-collapse: collapse; width: 100%; }
  th, td { text-align: left; padding: 6px 10px; border-bottom: 1px solid var(--line); font-size: 13px; }
  th { color: var(--muted); font-weight: 600; cursor: pointer; user-select: none; white-space: nowrap; }
  th:hover { color: var(--fg); }
  th .dir { font-size: 10px; }
  .num { text-align: right; font-variant-numeric: tabular-nums; }
  .muted { color: var(--muted); }
  .warn { color: var(--warn); }
  .bad { color: var(--bad); }
  .empty { color: var(--muted); font-style: italic; }
  svg text { font: 11px -apple-system, sans-serif; fill: var(--fg); }
  .edge { stroke: var(--line); stroke-width: 1.2; fill: none; marker-end: url(#arrow); }
  .edge.lazy { stroke-dasharray: 4 3; }
  .edge.hl { stroke: var(--accent); stroke-width: 2; }
  .node circle { fill: var(--card); stroke: var(--muted); stroke-width: 1.5; cursor: pointer; }
  .node.app circle { stroke: var(--accent); }
  .node.hl circle { stroke: var(--accent); stroke-width: 3; }
  .node.dim { opacity: 0.25; }
  .edge.dim { opacity: 0.15; }
  code { font-size: 12px; }
  code.copy { cursor: copy; }
  code.copy:hover { text-decoration: underline dotted; }
  mark { background: color-mix(in srgb, var(--accent) 28%, transparent); color: inherit; border-radius: 2px; padding: 0 1px; }

  /* Filter bar */
  #filters { position: sticky; top: 0; z-index: 10; background: var(--bg); padding: 8px 0 12px; display: flex; gap: 8px; flex-wrap: wrap; align-items: center; }
  #filters input[type="search"] { flex: 1 1 260px; min-width: 200px; }
  #filters input, #filters select, #filters button {
    font: inherit; font-size: 13px; color: var(--fg); background: var(--card);
    border: 1px solid var(--line); border-radius: 6px; padding: 6px 10px;
  }
  #filters input:focus, #filters select:focus { outline: 2px solid var(--accent); outline-offset: -1px; }
  #filters button { cursor: pointer; }
  #filters button:hover { border-color: var(--accent); }
  #filters .hint { color: var(--muted); font-size: 12px; }
  kbd { border: 1px solid var(--line); border-bottom-width: 2px; border-radius: 4px; padding: 0 5px; font-size: 11px; background: var(--card); }

  /* Summary chips */
  #summary { display: flex; gap: 8px; flex-wrap: wrap; margin-bottom: 16px; }
  #summary .chip { background: var(--chip); border: 1px solid var(--line); border-radius: 16px; padding: 3px 12px; font-size: 12px; cursor: pointer; }
  #summary .chip b { font-variant-numeric: tabular-nums; }
  #summary .chip:hover { border-color: var(--accent); }

  #toast { position: fixed; bottom: 20px; left: 50%; transform: translateX(-50%); background: var(--fg); color: var(--bg); padding: 6px 14px; border-radius: 6px; font-size: 12px; opacity: 0; transition: opacity .2s; pointer-events: none; }
  #toast.show { opacity: 0.92; }

  /* Per-section package-exclusion dropdown */
  .card-head { display: flex; justify-content: space-between; align-items: baseline; gap: 12px; flex-wrap: wrap; }
  .excl { position: relative; font-size: 12px; }
  .excl summary { cursor: pointer; color: var(--muted); border: 1px solid var(--line); border-radius: 6px; padding: 3px 10px; list-style: none; user-select: none; white-space: nowrap; }
  .excl summary::-webkit-details-marker { display: none; }
  .excl summary:hover { border-color: var(--accent); color: var(--fg); }
  .excl.active summary { border-color: var(--accent); border-width: 2px; color: var(--fg); }
  .excl-panel { position: absolute; right: 0; top: calc(100% + 4px); z-index: 20; background: var(--card); border: 1px solid var(--line); border-radius: 8px; padding: 8px; max-height: 280px; overflow-y: auto; min-width: 240px; box-shadow: 0 4px 16px rgba(0,0,0,.18); }
  .excl-panel label { display: block; padding: 3px 6px; border-radius: 4px; cursor: pointer; white-space: nowrap; }
  .excl-panel label:hover { background: var(--bg); }
  .excl-actions { display: flex; gap: 6px; margin-bottom: 6px; }
  .excl-actions button { font: inherit; font-size: 11px; color: var(--fg); background: var(--bg); border: 1px solid var(--line); border-radius: 4px; padding: 2px 8px; cursor: pointer; }
  .excl-actions button:hover { border-color: var(--accent); }
</style>
</head>
<body>
<h1>nx-analyzer — workspace report</h1>

<div id="summary"></div>

<div id="filters">
  <input id="f-q" type="search" placeholder="Search symbol, file, project…  ( / )">
  <select id="f-project"><option value="">all projects</option></select>
  <select id="f-tag"><option value="">all tags</option></select>
  <select id="f-kind"><option value="">all kinds</option></select>
  <select id="f-category"><option value="">all categories</option></select>
  <button id="f-clear" title="Clear all filters">✕ clear</button>
  <span class="hint"><kbd>/</kbd> search · <kbd>Esc</kbd> clear · click a graph node to filter by project · click a file path to copy</span>
</div>

<div class="card" id="card-graph">
  <h2 onclick="toggleCard(this)"><span class="caret">▼</span> Project graph <span class="muted">(click a node to filter; dashed = lazy)</span></h2>
  <div class="body"><svg id="graph" width="100%" height="480"></svg></div>
</div>

<div class="card" id="card-projects"><h2 onclick="toggleCard(this)"><span class="caret">▼</span> Projects <span class="count"></span></h2><div class="body"><div id="t-projects"></div></div></div>
<div class="card" id="card-deps"><h2 onclick="toggleCard(this)"><span class="caret">▼</span> Dependencies <span class="muted">(package → package)</span> <span class="count"></span></h2><div class="body"><div id="t-deps"></div></div></div>
<div class="card" id="card-unused"><h2 onclick="toggleCard(this)"><span class="caret">▼</span> Unused code <span class="count"></span></h2><div class="body"><div id="t-unused"></div></div></div>
<div class="card" id="card-cycles"><h2 onclick="toggleCard(this)"><span class="caret">▼</span> Cycles <span class="count"></span></h2><div class="body"><div id="t-cycles"></div></div></div>
<div class="card" id="card-moves">
  <div class="card-head">
    <h2 onclick="toggleCard(this)"><span class="caret">▼</span> Move candidates <span class="count"></span></h2>
    <details class="excl" id="moves-tags">
      <summary id="moves-tags-summary">tags ▾</summary>
      <div class="excl-panel" id="moves-tags-panel"></div>
    </details>
    <details class="excl" id="moves-excl">
      <summary id="moves-excl-summary">exclude packages ▾</summary>
      <div class="excl-panel" id="moves-excl-panel"></div>
    </details>
  </div>
  <div class="body"><div id="t-moves"></div></div>
</div>
<div class="card" id="card-boundaries"><h2 onclick="toggleCard(this)"><span class="caret">▼</span> Boundary violations <span class="count"></span></h2><div class="body"><div id="t-boundaries"></div></div></div>

<div id="toast"></div>

<script>
// A silently-dead script looks like "filters don't work" — surface any
// runtime error as a visible banner instead.
window.addEventListener('error', (e) => {
  let banner = document.getElementById('err-banner');
  if (!banner) {
    banner = document.createElement('div');
    banner.id = 'err-banner';
    banner.style.cssText = 'position:fixed;top:0;left:0;right:0;z-index:99;background:#b91c1c;color:#fff;padding:8px 16px;font:13px monospace;';
    document.body.prepend(banner);
  }
  banner.textContent = 'report script error: ' + e.message + ' @ line ' + e.lineno;
});

const DATA = __DATA__;

// ---------- state ----------
const state = {
  q: '', project: '', tag: '', kind: '', category: '',
  excludeMoves: new Set(),
  movesTags: new Set(),        // tags selected in the moves tag panel
  movesTagMode: 'exclude',     // 'exclude' hides matching, 'include' keeps only matching
};

// NX tag strategy: project name -> tags (from project.json).
const TAGS = {};
for (const p of DATA.stats.projects) TAGS[p.name] = p.tags || [];
const tagsOf = (...names) => names.flatMap(n => TAGS[n] || []);

function esc(s) {
  return String(s).replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
}

// Highlight the current search needle inside already-escaped text.
function hl(s) {
  const text = esc(s);
  if (!state.q) return text;
  const i = text.toLowerCase().indexOf(state.q.toLowerCase());
  if (i < 0) return text;
  return text.slice(0, i) + '<mark>' + text.slice(i, i + state.q.length) + '</mark>' + text.slice(i + state.q.length);
}

// ---------- row models ----------
const unusedRows = [
  ...DATA.unused.unused_exports.map(s => ({ category: 'unused', badge: '<span class="bad">unused</span>', ...s })),
  ...DATA.unused.declared_not_rendered.map(s => ({ category: 'not rendered', badge: '<span class="warn">not rendered</span>', ...s })),
  ...DATA.unused.test_only_exports.map(s => ({ category: 'test-only', badge: '<span class="muted">test-only</span>', ...s })),
  ...(DATA.unused.export_only || []).map(s => ({ category: 'export unnecessary', badge: '<span class="muted" title="used in its own file — only the export keyword is suspect">export?</span>', ...s })),
  ...DATA.unused.orphan_files.map(f => ({ category: 'orphan file', badge: '<span class="warn">orphan file</span>', name: '—', kind: 'File', project: '', file: f })),
];
const cycleRows = [
  ...DATA.stats.project_cycles.map(c => ({ level: 'projects', parts: c, badge: '<span class="bad">projects</span>' })),
  ...DATA.fileCycles.map(c => ({ level: 'files', parts: c, badge: 'files' })),
];

// ---------- section definitions ----------
// col: { label, text(row) -> plain value for sort/search, html(row), num? }
const file = f => `<code class="copy" title="click to copy">${hl(f)}</code>`;
const SECTIONS = {
  projects: {
    rows: DATA.stats.projects,
    match: (p) => matchProject([p.name]) && matchTag([p.name]) && matchQ([p.name, p.project_type, p.tags.join(' ')]),
    cols: [
      { label: 'project', text: p => p.name, html: p => `<strong>${hl(p.name)}</strong>` },
      { label: 'type', text: p => p.project_type, html: p => esc(p.project_type) },
      { label: 'tags', text: p => p.tags.join(', '), html: p => esc(p.tags.join(', ')) || '—' },
      { label: 'files', num: true, text: p => p.files },
      { label: 'exports', num: true, text: p => p.exports },
      { label: 'Ca', num: true, text: p => p.afferent },
      { label: 'Ce', num: true, text: p => p.efferent },
      { label: 'instability', num: true, text: p => p.instability, html: p => `<span class="num">${p.instability.toFixed(2)}</span>` },
    ],
  },
  deps: {
    rows: DATA.stats.dependencies,
    match: (d) => matchProject([d.from, d.to]) && matchTag([d.from, d.to]) &&
      matchQ([d.from, d.to, d.symbols.map(s => s.name).join(' ')]),
    cols: [
      { label: 'from', text: d => d.from, html: d => hl(d.from) },
      { label: 'to', text: d => d.to, html: d => hl(d.to) },
      { label: 'refs', num: true, text: d => d.count },
      { label: 'lazy', text: d => d.lazy ? 'yes' : '', html: d => d.lazy ? 'yes' : '' },
      { label: 'symbols', text: d => d.symbols.map(s => s.name).join(' '), html: d => `<code>${d.symbols.map(s => `${hl(s.name)}×${s.count}`).join(', ')}</code>` },
    ],
  },
  unused: {
    rows: unusedRows,
    match: (s) => matchProject([s.project]) && matchTag([s.project]) &&
      (!state.kind || s.kind === state.kind) &&
      (!state.category || s.category === state.category) &&
      matchQ([s.name, s.file, s.project]),
    cols: [
      { label: 'category', text: s => s.category, html: s => s.badge },
      { label: 'symbol', text: s => s.name, html: s => hl(s.name) },
      { label: 'kind', text: s => s.kind, html: s => esc(s.kind) },
      { label: 'project', text: s => s.project, html: s => hl(s.project) },
      { label: 'file', text: s => s.file, html: s => file(s.file) },
    ],
  },
  cycles: {
    rows: cycleRows,
    match: (c) => (!state.project || c.parts.some(p => p === state.project || p.includes(state.project))) &&
      // Tags apply to project-level cycles; file cycles have no tag identity.
      (c.level !== 'projects' || matchTag(c.parts)) &&
      matchQ([c.parts.join(' ')]),
    cols: [
      { label: 'level', text: c => c.level, html: c => c.badge },
      { label: 'cycle', text: c => c.parts.join(' '), html: c => `<code>${c.parts.map(hl).join(c.level === 'projects' ? ' ⇄ ' : ' → ')}</code>` },
    ],
  },
  moves: {
    rows: DATA.moveCandidates,
    match: (m) => {
      if (state.excludeMoves.has(m.from_project) || state.excludeMoves.has(m.to_project)) return false;
      if (state.movesTags.size) {
        const hit = tagsOf(m.from_project, m.to_project).some(t => state.movesTags.has(t));
        if (state.movesTagMode === 'exclude' ? hit : !hit) return false;
      }
      return matchProject([m.from_project, m.to_project]) &&
        matchTag([m.from_project, m.to_project]) &&
        matchQ([m.symbol, m.file]);
    },
    cols: [
      { label: 'symbol', text: m => m.symbol, html: m => `<strong>${hl(m.symbol)}</strong>` },
      { label: 'from', text: m => m.from_project, html: m => hl(m.from_project) },
      { label: 'suggested target', text: m => m.to_project, html: m => hl(m.to_project) },
      { label: 'external uses', num: true, text: m => m.external_usages },
      { label: 'file', text: m => m.file, html: m => file(m.file) },
    ],
  },
  boundaries: {
    rows: DATA.boundaryViolations,
    match: (v) => matchProject([v.from, v.to]) && matchTag([v.from, v.to]) &&
      matchQ([v.from, v.to, v.source_tag, v.to_tags.join(' ')]),
    cols: [
      { label: 'from', text: v => v.from, html: v => hl(v.from) },
      { label: 'to', text: v => v.to, html: v => hl(v.to) },
      { label: 'violated tag rule', text: v => v.source_tag, html: v => `<code>${esc(v.source_tag)} → [${esc(v.allowed_tags.join(', '))}]</code>` },
      { label: 'target tags', text: v => v.to_tags.join(', '), html: v => esc(v.to_tags.join(', ')) },
    ],
  },
};
const sort = {}; // section -> { col, dir }

function matchProject(fields) {
  return !state.project || fields.includes(state.project);
}
// Any of the involved projects carries the selected tag.
function matchTag(projectNames) {
  return !state.tag || tagsOf(...projectNames).includes(state.tag);
}
function matchQ(fields) {
  if (!state.q) return true;
  const q = state.q.toLowerCase();
  return fields.some(f => String(f).toLowerCase().includes(q));
}

// ---------- rendering ----------
function renderSection(id) {
  const s = SECTIONS[id];
  const shown = s.rows.filter(s.match);
  const ord = sort[id];
  if (ord) {
    const col = s.cols[ord.col];
    shown.sort((a, b) => {
      const x = col.text(a), y = col.text(b);
      const cmp = col.num ? (x - y) : String(x).localeCompare(String(y));
      return ord.dir === 'asc' ? cmp : -cmp;
    });
  }

  const card = document.getElementById('card-' + id);
  card.querySelector('.count').textContent =
    shown.length === s.rows.length ? `· ${s.rows.length}` : `· ${shown.length} / ${s.rows.length}`;

  const target = document.getElementById('t-' + id);
  if (!s.rows.length) { target.innerHTML = '<p class="empty">none</p>'; return; }
  if (!shown.length) {
    target.innerHTML = `<p class="empty">no matches (active: ${activeFilterSummary() || 'none'}) — <a href="javascript:void 0" onclick="clearFilters()">clear filters</a></p>`;
    return;
  }

  const head = s.cols.map((c, i) => {
    const dir = ord && ord.col === i ? `<span class="dir">${ord.dir === 'asc' ? '▲' : '▼'}</span>` : '';
    return `<th data-section="${id}" data-col="${i}">${c.label} ${dir}</th>`;
  }).join('');
  const body = shown.map(r => '<tr>' + s.cols.map(c => {
    const html = c.html ? c.html(r) : (c.num ? `<span class="num">${c.text(r)}</span>` : esc(c.text(r)));
    return `<td${c.num ? ' class="num"' : ''}>${html}</td>`;
  }).join('') + '</tr>').join('');
  target.innerHTML = `<table><thead><tr>${head}</tr></thead><tbody>${body}</tbody></table>`;
}

function renderAll() {
  syncControls();
  for (const id of Object.keys(SECTIONS)) renderSection(id);
  updateGraphDim();
  updateHash();
}

// Reflect state in the controls (graph clicks / chips also set filters) and
// mark active ones — a forgotten filter silently emptying tables is the #1
// "filtering is broken" trap.
function syncControls() {
  $q.value = state.q; $project.value = state.project; $tag.value = state.tag;
  $kind.value = state.kind; $category.value = state.category;
  for (const [el, val] of [[$q, state.q], [$project, state.project], [$tag, state.tag], [$kind, state.kind], [$category, state.category]]) {
    el.style.borderColor = val ? 'var(--accent)' : '';
    el.style.borderWidth = val ? '2px' : '';
  }
  const excluded = state.excludeMoves.size;
  document.getElementById('moves-excl-summary').textContent =
    excluded ? `exclude packages (${excluded}) ▾` : 'exclude packages ▾';
  document.getElementById('moves-excl').classList.toggle('active', excluded > 0);
  for (const box of document.querySelectorAll('#moves-excl-panel input[type="checkbox"]')) {
    box.checked = !state.excludeMoves.has(box.dataset.name);
  }
  const tagCount = state.movesTags.size;
  document.getElementById('moves-tags-summary').textContent =
    tagCount ? `tags: ${state.movesTagMode} (${tagCount}) ▾` : 'tags ▾';
  document.getElementById('moves-tags').classList.toggle('active', tagCount > 0);
  for (const box of document.querySelectorAll('#moves-tags-panel input[data-tag]')) {
    box.checked = state.movesTags.has(box.dataset.tag);
  }
  for (const radio of document.querySelectorAll('#moves-tags-panel input[name="mtagmode"]')) {
    radio.checked = radio.value === state.movesTagMode;
  }
}

function activeFilterSummary() {
  const parts = [];
  if (state.q) parts.push(`search "${esc(state.q)}"`);
  if (state.project) parts.push(`project ${esc(state.project)}`);
  if (state.tag) parts.push(`tag ${esc(state.tag)}`);
  if (state.kind) parts.push(`kind ${esc(state.kind)}`);
  if (state.category) parts.push(`category ${esc(state.category)}`);
  if (state.excludeMoves.size) parts.push(`${state.excludeMoves.size} package(s) excluded from moves`);
  if (state.movesTags.size) parts.push(`moves tags ${state.movesTagMode}: ${[...state.movesTags].map(esc).join(', ')}`);
  return parts.join(', ');
}

// ---------- move-candidates package exclusion ----------
function fillMovesExclude() {
  const names = [...new Set(DATA.moveCandidates.flatMap(m => [m.from_project, m.to_project]))]
    .filter(Boolean).sort();
  const panel = document.getElementById('moves-excl-panel');
  if (!names.length) {
    document.getElementById('moves-excl').style.display = 'none';
    return;
  }
  panel.innerHTML =
    '<div class="excl-actions"><button type="button" data-act="all">include all</button><button type="button" data-act="none">exclude all</button></div>' +
    names.map(n => `<label><input type="checkbox" checked data-name="${esc(n)}"> ${esc(n)}</label>`).join('');

  panel.addEventListener('change', (e) => {
    const box = e.target.closest('input[type="checkbox"]');
    if (!box) return;
    if (box.checked) state.excludeMoves.delete(box.dataset.name);
    else state.excludeMoves.add(box.dataset.name);
    renderAll();
  });
  panel.addEventListener('click', (e) => {
    const btn = e.target.closest('button[data-act]');
    if (!btn) return;
    state.excludeMoves = btn.dataset.act === 'none' ? new Set(names) : new Set();
    renderAll();
  });
}

// Tag-based include/exclude for move candidates. `exclude` hides candidates
// whose projects carry a selected tag; `include` keeps ONLY those.
function fillMovesTags() {
  const tags = Object.keys(allTags()).sort();
  const details = document.getElementById('moves-tags');
  if (!tags.length) {
    details.style.display = 'none';
    return;
  }
  const panel = document.getElementById('moves-tags-panel');
  panel.innerHTML =
    '<div class="excl-actions">' +
    '<label><input type="radio" name="mtagmode" value="exclude" checked> exclude selected</label> ' +
    '<label><input type="radio" name="mtagmode" value="include"> include only selected</label>' +
    '</div>' +
    '<div class="excl-actions"><button type="button" data-act="none">clear selection</button></div>' +
    tags.map(t => `<label><input type="checkbox" data-tag="${esc(t)}"> ${esc(t)}</label>`).join('');

  panel.addEventListener('change', (e) => {
    const radio = e.target.closest('input[name="mtagmode"]');
    if (radio) { state.movesTagMode = radio.value; renderAll(); return; }
    const box = e.target.closest('input[data-tag]');
    if (!box) return;
    if (box.checked) state.movesTags.add(box.dataset.tag);
    else state.movesTags.delete(box.dataset.tag);
    renderAll();
  });
  panel.addEventListener('click', (e) => {
    if (e.target.closest('button[data-act="none"]')) {
      state.movesTags = new Set();
      renderAll();
    }
  });
}

// Close any open exclusion dropdown when clicking outside it.
document.addEventListener('click', (e) => {
  for (const details of document.querySelectorAll('details.excl[open]')) {
    if (!details.contains(e.target)) details.open = false;
  }
});

// ---------- filter bar ----------
const $q = document.getElementById('f-q');
const $project = document.getElementById('f-project');
const $tag = document.getElementById('f-tag');
const $kind = document.getElementById('f-kind');
const $category = document.getElementById('f-category');

function allTags() {
  const counts = {};
  for (const p of DATA.stats.projects) for (const t of p.tags || []) counts[t] = (counts[t] || 0) + 1;
  return counts;
}

function fillOptions() {
  for (const p of [...DATA.stats.projects].sort((a, b) => a.name.localeCompare(b.name))) {
    $project.add(new Option(p.name, p.name));
  }
  const tags = allTags();
  for (const t of Object.keys(tags).sort()) $tag.add(new Option(`${t} (${tags[t]})`, t));
  if (!Object.keys(tags).length) $tag.style.display = 'none'; // untagged workspace
  const kinds = {};
  for (const r of unusedRows) kinds[r.kind] = (kinds[r.kind] || 0) + 1;
  for (const k of Object.keys(kinds).sort()) $kind.add(new Option(`${k} (${kinds[k]})`, k));
  const cats = {};
  for (const r of unusedRows) cats[r.category] = (cats[r.category] || 0) + 1;
  for (const c of Object.keys(cats)) $category.add(new Option(`${c} (${cats[c]})`, c));
}

let debounce;
$q.addEventListener('input', () => {
  clearTimeout(debounce);
  debounce = setTimeout(() => { state.q = $q.value.trim(); renderAll(); }, 150);
});
$project.addEventListener('change', () => { state.project = $project.value; renderAll(); });
$tag.addEventListener('change', () => { state.tag = $tag.value; renderAll(); });
$kind.addEventListener('change', () => { state.kind = $kind.value; renderAll(); });
$category.addEventListener('change', () => { state.category = $category.value; renderAll(); });
document.getElementById('f-clear').addEventListener('click', clearFilters);

function clearFilters() {
  state.q = state.project = state.tag = state.kind = state.category = '';
  state.excludeMoves = new Set();
  state.movesTags = new Set();
  state.movesTagMode = 'exclude';
  renderAll();
}

document.addEventListener('keydown', (e) => {
  if (e.key === '/' && document.activeElement !== $q) { e.preventDefault(); $q.focus(); $q.select(); }
  if (e.key === 'Escape') { clearFilters(); $q.blur(); }
});

// Sorting via header clicks (delegated — tables re-render).
document.addEventListener('click', (e) => {
  const th = e.target.closest('th[data-section]');
  if (th) {
    const id = th.dataset.section, col = +th.dataset.col;
    const cur = sort[id];
    sort[id] = { col, dir: cur && cur.col === col && cur.dir === 'asc' ? 'desc' : 'asc' };
    renderSection(id);
    return;
  }
  const code = e.target.closest('code.copy');
  if (code) {
    navigator.clipboard && navigator.clipboard.writeText(code.textContent).then(() => toast('path copied'));
  }
});

function toggleCard(h2) { h2.closest('.card').classList.toggle('collapsed'); }

let toastTimer;
function toast(msg) {
  const t = document.getElementById('toast');
  t.textContent = msg;
  t.classList.add('show');
  clearTimeout(toastTimer);
  toastTimer = setTimeout(() => t.classList.remove('show'), 1200);
}

// ---------- URL hash persistence ----------
function updateHash() {
  // Safari throws SecurityError for replaceState on file:// — persistence is
  // a nice-to-have, never let it break filtering.
  try {
    const params = new URLSearchParams();
    for (const key of ['q', 'project', 'tag', 'kind', 'category']) if (state[key]) params.set(key, state[key]);
    if (state.excludeMoves.size) params.set('xmoves', [...state.excludeMoves].join(','));
    if (state.movesTags.size) {
      params.set('mtags', [...state.movesTags].join(','));
      params.set('mtagmode', state.movesTagMode);
    }
    history.replaceState(null, '', params.toString() ? '#' + params.toString() : location.pathname);
  } catch (e) { /* filtering must keep working without persistence */ }
}
function restoreHash() {
  const params = new URLSearchParams(location.hash.slice(1));
  state.q = params.get('q') || '';
  state.project = params.get('project') || '';
  state.tag = params.get('tag') || '';
  state.kind = params.get('kind') || '';
  state.category = params.get('category') || '';
  state.excludeMoves = new Set((params.get('xmoves') || '').split(',').filter(Boolean));
  state.movesTags = new Set((params.get('mtags') || '').split(',').filter(Boolean));
  state.movesTagMode = params.get('mtagmode') === 'include' ? 'include' : 'exclude';
}

// ---------- summary chips ----------
// Chips for unused categories APPLY that category filter and scroll to the
// section; the rest just scroll.
(function () {
  const chips = [
    ['projects', DATA.stats.projects.length, 'card-projects', ''],
    ['dependencies', DATA.stats.dependencies.length, 'card-deps', ''],
    ['unused', DATA.unused.unused_exports.length, 'card-unused', 'unused'],
    ['test-only', DATA.unused.test_only_exports.length, 'card-unused', 'test-only'],
    ['export?', (DATA.unused.export_only || []).length, 'card-unused', 'export unnecessary'],
    ['orphan files', DATA.unused.orphan_files.length, 'card-unused', 'orphan file'],
    ['cycles', DATA.stats.project_cycles.length + DATA.fileCycles.length, 'card-cycles', ''],
    ['move candidates', DATA.moveCandidates.length, 'card-moves', ''],
    ['violations', DATA.boundaryViolations.length, 'card-boundaries', ''],
  ];
  document.getElementById('summary').innerHTML = chips
    .map(([label, n, target, category]) => `<span class="chip" data-target="${target}" data-category="${category}"><b>${n}</b> ${label}</span>`)
    .join('');
  document.getElementById('summary').addEventListener('click', (e) => {
    const chip = e.target.closest('.chip');
    if (!chip) return;
    if (chip.dataset.category) {
      state.category = state.category === chip.dataset.category ? '' : chip.dataset.category;
      renderAll();
    }
    document.getElementById(chip.dataset.target).scrollIntoView({ behavior: 'smooth' });
  });
})();

// ---------- graph (circular layout) ----------
const nodeEls = {};
const edgeEls = [];
(function () {
  function el(tag, attrs, children) {
    const node = document.createElementNS('http://www.w3.org/2000/svg', tag);
    for (const [k, v] of Object.entries(attrs || {})) node.setAttribute(k, v);
    for (const child of children || []) node.appendChild(child);
    return node;
  }

  const svg = document.getElementById('graph');
  const projects = DATA.stats.projects;
  const deps = DATA.stats.dependencies;
  const W = svg.clientWidth || 900, H = 480, cx = W / 2, cy = H / 2;
  const R = Math.min(W, H) / 2 - 70;
  svg.setAttribute('viewBox', `0 0 ${W} ${H}`);

  svg.appendChild(el('defs', {}, [
    el('marker', { id: 'arrow', viewBox: '0 0 10 10', refX: 22, refY: 5, markerWidth: 7, markerHeight: 7, orient: 'auto-start-reverse' }, [
      el('path', { d: 'M 0 0 L 10 5 L 0 10 z', fill: 'currentColor', opacity: 0.55 }),
    ]),
  ]));

  const pos = {};
  projects.forEach((p, i) => {
    const angle = (2 * Math.PI * i) / projects.length - Math.PI / 2;
    pos[p.name] = { x: cx + R * Math.cos(angle), y: cy + R * Math.sin(angle) };
  });

  for (const dep of deps) {
    const a = pos[dep.from], b = pos[dep.to];
    if (!a || !b) continue;
    const mx = (a.x + b.x) / 2 + (b.y - a.y) * 0.08;
    const my = (a.y + b.y) / 2 - (b.x - a.x) * 0.08;
    const path = el('path', { d: `M ${a.x} ${a.y} Q ${mx} ${my} ${b.x} ${b.y}`, class: 'edge' + (dep.lazy ? ' lazy' : '') });
    path.dataset.from = dep.from; path.dataset.to = dep.to;
    svg.appendChild(path);
    edgeEls.push(path);
  }

  for (const p of projects) {
    const { x, y } = pos[p.name];
    const r = 10 + Math.min(14, Math.sqrt(p.files) * 2.2);
    const group = el('g', { class: 'node' + (p.project_type === 'application' ? ' app' : ''), transform: `translate(${x},${y})` }, [
      el('circle', { r }),
      el('text', { y: r + 13, 'text-anchor': 'middle' }),
    ]);
    group.querySelector('text').textContent = p.name;
    // Click = toggle the project filter (drives every table).
    group.addEventListener('click', () => {
      state.project = state.project === p.name ? '' : p.name;
      $project.value = state.project;
      renderAll();
    });
    nodeEls[p.name] = group;
    svg.appendChild(group);
  }
})();

// Dim everything unrelated to the active project filter; highlight its edges.
function updateGraphDim() {
  const active = state.project;
  const connected = new Set();
  for (const e of edgeEls) {
    const isMine = active && (e.dataset.from === active || e.dataset.to === active);
    e.classList.toggle('hl', !!isMine);
    e.classList.toggle('dim', !!active && !isMine);
    if (isMine) { connected.add(e.dataset.from); connected.add(e.dataset.to); }
  }
  for (const [name, node] of Object.entries(nodeEls)) {
    node.classList.toggle('hl', name === active);
    node.classList.toggle('dim', !!active && name !== active && !connected.has(name));
  }
}

// ---------- boot ----------
fillOptions();
fillMovesExclude();
fillMovesTags();
restoreHash();
renderAll();
</script>
</body>
</html>
"##;
