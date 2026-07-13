# Plan działania i modyfikacji kodu — ngAnalyzer

Wersja: 1.1 (2026-07-12). Towarzyszy `PRD.md` (wymagania) i `FIXTURES.md` (spec fixture'ów).

## Status realizacji

| Milestone | Status | Uwagi |
|---|---|---|
| M0 — fundament, bugi B1–B14, fixtures F01/F03/F12, CI | ✅ dostarczone | 14 bugów naprawionych z testami regresyjnymi |
| M1 — graf symboli, eksporty, tsx/js, cache AST, metadane Ng | ✅ dostarczone | `oxc_resolver` odłożony: własny resolver przechodzi wszystkie fixtures (decyzja: nie wymieniać działającego komponentu bez potrzeby); F02/F04/F05/F14 |
| M2 — szablony, lazy routes, DI | ✅ dostarczone | własny skaner HTML + matcher selektorów (zero nowych zależności); F06/F10/F11 |
| M3 — stats/unused/cykle/move/boundaries | ✅ dostarczone | F07/F08/F09/F13 |
| M4 — CLI subkomendy, Mermaid/DOT, HTML, SARIF, baseline | ✅ dostarczone | testy CLI z kodami wyjścia |
| M4.5 — wiarygodność `unused` (krok 0 + 1) | ✅ dostarczone | metryka rezolucji + `--strict`, konsumpcja `used_import_names`, kategoria `unused_imports`, importy side-effect/namespace, detekcja barreli po strukturze; F22 |
| M5 — incremental cache, watch, hotspoty git | ⬜ zaplanowane | opcjonalne |
| M6 — architektura pluginowa + React | 🟡 M6-lite dostarczone | komponenty funkcyjne, JSX usage, React.lazy, statystyki propsów (F15); formalny kontrakt pluginu — później |
| M7 — MCP server, analityka inputów Ng | ⬜ zaplanowane | opcjonalne |

Stan testów: 16 jednostkowych + 39 integracyjnych na 13 fixture'ach; `clippy -D warnings` i `fmt --check` czyste.

## 0. Diagnoza obecnego kodu

Architektura (parser SWC → visitor → analyzery dekoratorów → resolver importów → graf) jest
sensownym szkieletem i nadaje się do rozbudowy. Poniżej wszystkie znalezione problemy,
posortowane wg wagi. "P" = priorytet.

### 0.1 Bugi poprawności (P0 — naprawić przed rozbudową)

| # | Miejsce | Problem |
|---|---|---|
| B1 | `src/analysis/resolvers/cache.rs` + `import_resolver.rs:40` | **Klucz cache to (źródło importu, nazwa) globalnie.** Import względny `./model` z dwóch różnych katalogów trafia w ten sam wpis — pierwszy zapis wygrywa, kolejne pliki dostają błędnie zresolvowaną ścieżkę. Klucz musi zawierać katalog pliku importującego (lub cache'ować dopiero po normalizacji do ścieżki absolutnej). |
| B2 | `src/analysis/processor/file_processor.rs:55-64` | `filter()` **ignoruje predykat** (buduje `Self` bez użycia `predicate`), więc `filter_node_modules()`/`filter_ts_files()` to no-opy. Flagi CLI `-n`/`-t` nie robią nic; filtry są zahardkodowane w `collect_paths()`. |
| B3 | `src/main.rs:46-51` | `#[arg(default_value = "true")]` na `bool` — flag nie da się wyłączyć z CLI (clap traktuje je jako przełączniki). Potrzebne `ArgAction::Set` + wartość, albo flagi odwrotne (`--include-node-modules`). |
| B4 | `src/ng/visitors/visitor.rs:49` | `DecoratorAnalysisCache::new()` tworzony **w każdym wywołaniu** `process_decorator` — cache jest martwy. Dodatkowo klucz `format!("{:p}", decorator)` (adres pamięci) jest niestabilny i może kolidować po zwolnieniu pamięci. Cache do usunięcia albo przeniesienia na poziom pliku z sensownym kluczem. |
| B5 | `src/ng/visitors/visitor.rs:165-182` | Analizowane są tylko klasy w postaci `export class X` na top-level. Pomijane: klasy niewyeksportowane, `export default class`, `class X {} export { X }`. |
| B6 | `src/analysis/resolvers/resolver.rs:42-60` | `resolve_relative_import` zwraca `Some(resolved)` nawet gdy ścieżka **nie istnieje** (fallthrough po pętli rozszerzeń). Błędne ścieżki propagują się do grafu. |
| B7 | `src/analysis/resolvers/import_graph.rs:62-103` | Detekcja cykli: pojedynczy zbiór `visited` sprawia, że cykle przechodzące przez raz odwiedzony węzeł nie są wykrywane; rekurencyjny DFS grozi stack overflow na dużych grafach; `path.contains` jest O(n). Zastąpić Tarjan SCC z `petgraph`. |
| B8 | `src/main.rs` | `ImportGraph` jest budowany i **nigdy nie trafia do wyniku** ani raportu. |
| B9 | `src/nx/nx_workspace.rs:59-84` | Projekt bez `tsconfig.json` obok `project.json` jest **pomijany w całości** (bez logu w trybie domyślnym). NX często trzyma tsconfig tylko wariantowe (`tsconfig.lib.json`, `tsconfig.app.json`). |
| B10 | `src/nx/config/project.rs` | `name` i `sourceRoot` wymagane przy deserializacji — w NX oba są opcjonalne (inferowane). Realne workspace'y będą sypać błędami parsowania i projekty znikną z analizy. |
| B11 | `src/nx/nx_workspace.rs:109-148` | `extends` w tsconfig rozwiązywany tylko o **jeden poziom**, tylko ścieżką względną (brak `extends` po nazwie pakietu z node_modules), scalanie tylko `paths`+`baseUrl`. NX ma łańcuch `tsconfig.json → tsconfig.base.json`. |
| B12 | `src/analysis/resolvers/resolver.rs:99-109` | Aliasy tsconfig nie zaczynające się od `@` (np. `libs/*`, `shared-ui`) klasyfikowane jako `NodeModule` i nigdy nie próbowane przez `ts_paths`. |
| B13 | `src/ng/models/ng_results.rs:39` | `context.project_name.clone().parse()?` — parsowanie `String`→`String`; zbędne i potencjalnie mylące. |
| B14 | `src/analysis/timing.rs` użycie w `main.rs` | `total_analysis_time` nigdy nie ustawiany („Total analysis time: 0ns" widoczne nawet w README). |

### 0.2 Braki funkcjonalne względem celu (P1)

| # | Obszar | Brak |
|---|---|---|
| F1 | Komponent | Nie zbieramy: inline `template`, `styles`, **`imports` (standalone!)**, `providers`, `hostDirectives`, inputs/outputs (dekoratorowe i sygnałowe), `standalone` domyślnie `true` od Angular 19 (dziś default `false`). |
| F2 | NgModule | Nie zbieramy tablicy `imports`; `declarations/exports/providers` tylko jako gołe identyfikatory (bez rezolucji do symboli). |
| F3 | Eksporty | Zbieramy tylko klasy dekorowane; do analizy nieużytków potrzebne **wszystkie eksporty** każdego pliku. |
| F4 | Użycia | Zbieramy tylko `import` deklaracje. Brak: `inject()`, typy konstruktorów, tokeny DI, `loadChildren`/`loadComponent` (lazy routes!), użycia w szablonach HTML. |
| F5 | Szablony | Zero analizy HTML — bez tego nie da się stwierdzić, że komponent/pipe/dyrektywa jest nieużywana. |
| F6 | Raport | Brak jakichkolwiek analiz pochodnych: statystyk pakiet→pakiet, unused, cykli w output, move-candidates. |
| F7 | Testy | Jedyne testy to `path_utils`; zero testów parserów, resolvera, wizytora. |

### 0.3 Dług techniczny / higiena (P2)

- `parsers.rs` parsuje pliki barrel od zera przy każdym zapytaniu (własny `SourceMap` per call) — potrzebny cache sparsowanych modułów; obecnie potencjalnie O(plików × eksportów).
- Zależność `swc = "8"` (pełny kompilator) jest nieużywana — wystarczą `swc_ecma_parser/ast/visit`; wywalić, dramatycznie skróci build.
- `Box<dyn Error>` wszędzie — przejść na `thiserror`/`anyhow`; błędy parsowania są dziś połykane (`Err(_) => Ok(default)` w `visitors/mod.rs:58`).
- `collect_project_files` w `nx_workspace.rs` zbiera listę plików projektu, po czym `ProjectProcessor` i tak robi drugi `WalkDir`.
- `CachedFileReader` z TTL 300 s nie ma sensu dla jednorazowego procesu batch — uprościć do zwykłego cache bez TTL (albo usunąć, gdy pliki czytane raz).
- Nazwa binarki w README (`angular-analysis`) ≠ nazwa pakietu (`nx-analyzer`).
- `walkdir` bez respektowania `.gitignore` — użyć crate'a `ignore` (szybszy, gitignore-aware, wbudowana równoległość).
- Brak CI, brak `cargo fmt`/`clippy` w pipeline. Uwaga: na tej maszynie nie ma toolchaina Rust (`cargo` nie znaleziony) — krok 0: `rustup`.

## 1. Docelowa architektura

Pipeline (każda faza czysta, testowalna osobno):

```
discover   → WorkspaceModel  (projekty NX, tsconfig-chain, tagi, mapowanie plik→projekt)
parse      → ParsedFile      (AST per plik, parsowany DOKŁADNIE RAZ, cache w DashMap)
extract    → FileFacts       (eksporty, byty Ng z metadanymi, importy, użycia: inject/DI/lazy-routes,
                              referencje szablonów po parsowaniu HTML)
resolve    → SymbolGraph     (oxc_resolver: import → plik → symbol; krawędzie z typem użycia)
aggregate  → ProjectGraph    (agregacja do poziomu pakietów; petgraph)
analyze    → Findings        (unused / cycles / move-candidates / boundaries / stats — każda analiza
                              to osobny moduł na wspólnych grafach)
report     → JSON | SARIF | DOT | HTML | terminal
```

Kluczowe decyzje:
- **`oxc_resolver`** zamiast własnego `ImportPathResolver`/`ImportParser` — dostajemy za darmo:
  tsconfig `paths`+`extends` (rekurencyjnie), `exports` z package.json, symlinki, index-resolution.
  Własny kod zostaje tylko do "który plik faktycznie deklaruje symbol X" (przejście po barrelach)
  — z cache'em AST.
- **`petgraph`** dla grafów + Tarjan SCC (cykle), toposort, reachability (unused).
- Krawędź grafu symboli niesie **typ użycia**: `import`, `template-selector`, `template-pipe`,
  `di-token`, `lazy-route`, `test-only` — to pozwala odróżnić "martwy" od "używany tylko w testach".
- Szablony: parser HTML (`tree-sitter-html` lub `html5ever`) + własne dopasowanie selektorów
  (CSS-selector matching na elementach/atrybutach) i ekstrakcja pipe'ów z wyrażeń (regex/mini-parser
  wystarczy na v1; pełny parser wyrażeń Angulara w v2).
- Entry pointy do reachability: `main.ts` aplikacji, pliki routingu, `index.ts` (public API libki),
  konfigurowalne dodatkowe (np. `*.stories.ts`).

## 2. Roadmapa

### M0 — Fundament i naprawa poprawności (1–2 tyg.)
1. Instalacja toolchaina (rustup), CI (GitHub Actions: fmt, clippy, test, build).
2. Naprawa bugów B1–B14 (każdy z testem regresyjnym tam, gdzie się da).
3. Usunięcie zależności `swc`, wprowadzenie `thiserror`+`anyhow`, `ignore` zamiast `walkdir`.
4. Zbudowanie infrastruktury fixture'ów (patrz `FIXTURES.md`) + snapshot testy `insta`
   uruchamiające pełną binarkę na fixture i porównujące JSON.
   **Kryterium wyjścia:** zielone CI, fixtury F01–F03 przechodzą, `-p/-n/-t` działają.

### M1 — Poprawny graf symboli (2–3 tyg.)
1. Wymiana resolvera na `oxc_resolver` (feature-flag na czas migracji, porównanie wyników na fixturach).
2. Ekstrakcja **wszystkich** eksportów i importów (w tym `export default`, `export * as ns`).
3. Cache sparsowanych AST; każdy plik parsowany raz; przejście barreli na cache'u.
4. Graf symboli i plików na `petgraph`; serializacja grafu do JSON (`graph --format json|dot`).
5. Metadane Ng rozszerzone (F1, F2): standalone imports, inline template, providers, sygnały;
   flaga wersji Angulara (default standalone).
6. Pełne pokrycie plików workspace'u: `tsx: true` dla `.tsx` (dziś pliki React są niewidoczne),
   rozszerzenia `.js/.mjs/.cjs/.mts/.cts`; fixture F14 (czysta biblioteka TS bez frameworka) —
   core (unused/cykle/statystyki) musi działać dla bezframeworkowego kodu.
   **Kryterium:** fixtury F02, F04, F05, F12, F14 zielone; benchmark criterion na fixture dużym.

### M2 — Semantyka Angulara (2–3 tyg.)
1. Parser szablonów + dopasowanie selektorów komponentów/dyrektyw i pipe'ów (F5).
2. Scope widoczności: standalone `imports` / NgModule declarations+imports+exports —
   użycie selektora liczy się tylko, gdy symbol jest w scope szablonu.
3. Lazy routes: `loadChildren`/`loadComponent` (dynamiczne importy) jako krawędzie grafu.
4. DI: `inject(X)`, typy konstruktorów, `useClass/useExisting/InjectionToken`.
   **Kryterium:** fixtury F06, F10, F11 zielone.

### M3 — Analizy (2 tyg.)
1. `stats` — macierz pakiet→pakiet (liczby importów/symboli/użyć), metryki Ca/Ce/I, API waste.
2. `unused` — reachability z entry pointów; poziomy pewności; wykluczenia; test-only osobno.
3. `cycles` — SCC plikowe i pakietowe, z minimalnym cyklem przykładowym per SCC.
4. `move-candidates` — próg koncentracji użyć; `boundaries` — reguły na tagach NX.
   **Kryterium:** fixtury F07, F08, F09 zielone; ręczna walidacja na realnym workspace.

### M4 — Raportowanie i CI-adoption (1–2 tyg.)
1. Eksport DOT + **Mermaid** (raport renderowalny w PR na GitHub/GitLab — wzorzec skott).
2. Raport HTML: jeden samowystarczalny plik, graf pakietów w trybie composite z drill-down
   do plików/symboli (wzorzec Nx graph), panele unused/cykle/statystyki (podniesiony priorytet
   — patrz `COMPETITIVE_ANALYSIS.md` §2.3).
3. SARIF output; `--baseline`; `--fail-on`; stabilne sortowanie całego outputu.
4. README, przykłady, wersjonowanie 0.x → publikacja binarek (GH Releases).

### M5 — Wydajność i DX (opcjonalnie)
1. Incremental cache na dysku (hash pliku → FileFacts), tryb `--watch`; `--affected --base=<ref>`.
2. Eksport SCIP; integracja churn z git (hotspoty — nakładka na graf HTML).

### M6 — Architektura pluginowa + React (po M3)
1. Formalny podział: core (framework-agnostic) + plugin `angular`; kontrakt pluginu
   (ekstraktor semantyki: byty, użycia, lazy edges).
2. Plugin `react`: komponenty (funkcje/`memo`/`forwardRef`), użycia przez elementy JSX,
   `React.lazy()` jako lazy edge, statystyki użyć propsów (parytet react-scanner).
3. Fixture F15: aplikacja React + biblioteka komponentów w NX.

### M7 — Integracje (opcjonalnie)
1. Tryb MCP server (`nx-analyzer serve --mcp`): zapytania o graf/użycia/unused dla agentów AI
   (wzorzec knip @knip/mcp).
2. Analityka inputów/outputów Angulara per komponent (adopcja design systemu) — unikat rynkowy.

## 3. Kolejność prac w istniejących plikach (mapa modyfikacji)

| Plik | Akcja |
|---|---|
| `analysis/resolvers/cache.rs` | Klucz cache: ścieżka absolutna po rezolucji (naprawa B1); docelowo cache wewnątrz warstwy oxc_resolver |
| `analysis/resolvers/resolver.rs` | Do usunięcia po M1 (zastępuje `oxc_resolver`); wcześniej łatka B6, B12 |
| `analysis/resolvers/parsers.rs` | Zostaje jako "symbol locator" po barrelach, ale na współdzielonym cache AST |
| `analysis/resolvers/import_graph.rs` | Migracja na `petgraph`; usunięcie własnego `find_cycles` (B7); serializacja (B8) |
| `analysis/processor/file_processor.rs` | Naprawa `filter` (B2); walk przez crate `ignore`; jeden walk na workspace zamiast per projekt |
| `ng/visitors/visitor.rs` | Pełny `Visit` (klasy w dowolnej pozycji — B5); usunięcie `DecoratorAnalysisCache` (B4); zbieranie wszystkich eksportów i użyć (F3, F4) |
| `ng/analyzers/*` | Rozszerzenie metadanych (F1, F2); wspólny helper na property-extraction z ObjectLit |
| `nx/nx_workspace.rs` | Fallback tsconfig (B9), rekurencyjny `extends` (B11); projekty z `package.json` (FR-1.1) |
| `nx/config/project.rs` | Pola opcjonalne + inferencja (B10); doczytanie `tags` do modelu analizy |
| `main.rs` | Podkomendy clap (FR-7.1); naprawa flag (B3); wpięcie grafu do outputu (B8); timing (B14) |
| `file_cache_reader.rs` | Uproszczenie: cache bez TTL albo usunięcie |
| nowe: `analysis/graph/` | SymbolGraph/ProjectGraph na petgraph |
| nowe: `ng/templates/` | Parser HTML + selector matching |
| nowe: `analyses/` | unused.rs, cycles.rs, stats.rs, move_candidates.rs, boundaries.rs |
| nowe: `report/` | json.rs, sarif.rs, dot.rs, html.rs, terminal.rs |
| nowe: `tests/` | testy integracyjne na fixturach + snapshoty insta |

## 4. Ryzyka

| Ryzyko | Mitygacja |
|---|---|
| Semantyka scope szablonów (NgModule vs standalone) jest złożona | Fixtury najpierw, implementacja pod nie; przypadki niepewne raportować "low confidence" zamiast zgadywać |
| False positives w `unused` niszczą zaufanie | Reachability + typy użyć + baseline; publikować dopiero po walidacji na realnym repo |
| oxc_resolver może różnić się zachowaniem od obecnego kodu | Okres podwójnego działania za feature-flagą + porównanie na fixturach |
| Parsowanie wyrażeń szablonowych (pipe'y w skomplikowanych wyrażeniach) | v1: heurystyka `| pipeName`; pełny parser w v2 |
