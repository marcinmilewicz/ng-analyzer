// Imported for its side effects only — no bindings. Crosses a project
// boundary, so it reaches the package dependency matrix, where an import that
// names no symbol must not surface as an empty symbol name.
(globalThis as { fixReady?: boolean }).fixReady = true;

export {};
