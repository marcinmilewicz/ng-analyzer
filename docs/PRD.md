# PRD — ngAnalyzer: kompleksowy analizator kodu Angular w workspace'ach NX

Wersja: 1.0 (2026-07-11) · Status: draft do akceptacji

## 1. Wizja

Jedno narzędzie CLI (Rust, szybkie, bez zależności od Node w runtime), które dla workspace'a NX
z projektami Angular odpowiada na pytania:

1. **Kto używa czego i ile razy** — statystyki powiązań między pakietami (lib → lib, app → lib),
   na poziomie pakietu, pliku i symbolu.
2. **Co jest martwe** — komponenty, dyrektywy, pipe'y, serwisy, funkcje, typy i całe pliki,
   których nikt nie importuje ani nie używa w szablonach/routingu/DI → kandydaci do usunięcia.
3. **Co jest źle położone** — symbol z pakietu A używany wyłącznie (lub prawie wyłącznie)
   przez pakiet B → kandydat do przeniesienia; pakiety o patologicznym couplingu.
4. **Zdrowie architektury** — cykle zależności (plikowe i pakietowe), naruszenia granic
   (tagi NX), metryki couplingu (Ca/Ce, niestabilność, odległość od "main sequence").

Wynik: raporty JSON (maszynowe), SARIF (CI/GitHub code scanning), HTML (interaktywny graf)
oraz czytelny output terminalowy.

## 2. Użytkownicy i przypadki użycia

| Persona | Przypadek użycia |
|---|---|
| Dev w monorepo NX | "Czy mogę usunąć ten komponent?" — `nx-analyzer unused` |
| Tech lead / architekt | "Które pakiety są zbyt powiązane? Co przenieść?" — `stats`, `move-candidates` |
| CI pipeline | Budżety: "fail, jeśli pojawi się nowy cykl albo nowy nieużywany eksport" — `--baseline` |
| Onboarding | "Pokaż mi mapę zależności" — `graph --format html/dot` |

## 3. Zakres funkcjonalny (wymagania)

### FR-1. Model workspace'u NX
- FR-1.1 Wykrywanie projektów: `project.json` **oraz** projekty inferowane z `package.json`
  (workspaces); `name` i `sourceRoot` opcjonalne (inferencja z katalogu) — dziś parser je wymaga.
- FR-1.2 Pełny łańcuch `tsconfig` (`extends` rekurencyjnie, także z `node_modules`),
  scalanie `paths`/`baseUrl` zgodnie z semantyką TS; wsparcie aliasów nie zaczynających się od `@`.
- FR-1.3 Odczyt `tags` z projektów (podstawa reguł granic architektury).
- FR-1.4 Mapowanie plik → projekt (dziś projekt zna pliki, ale analiza chodzi po dysku drugi raz).

### FR-2. Ekstrakcja symboli (parsowanie TS)
- FR-2.1 Wszystkie eksporty pliku (klasy, funkcje, consty, typy, enumy, re-eksporty,
  `export default`, `export * as ns`), nie tylko klasy dekorowane.
- FR-2.2 Byty Angulara z pełnymi metadanymi:
  - Component: selector, standalone (z uwzględnieniem **domyślnego standalone od Angular 19**),
    `imports` (kluczowe!), `template`/`templateUrl`, `styles`/`styleUrl(s)`, `providers`,
    `hostDirectives`, inputs/outputs (dekoratory **i** sygnałowe `input()`/`output()`/`model()`).
  - Directive: selector, standalone, inputs/outputs, hostDirectives.
  - Pipe: name, pure, standalone.
  - Injectable: providedIn, zależności konstruktora + `inject()`.
  - NgModule: declarations, **imports**, exports, providers, bootstrap.
- FR-2.3 Użycia symboli w kodzie: importy (wszystkie warianty), `inject(X)`, typy w konstruktorach,
  tokeny DI (`useClass`, `useExisting`, `InjectionToken`, `forwardRef`).
- FR-2.4 Dynamiczne importy: `loadChildren`/`loadComponent` w konfiguracji routingu
  (bez tego lazy-loadowane feature'y wyglądają na martwe — krytyczne dla poprawności "unused").

### FR-3. Analiza szablonów HTML
- FR-3.1 Parsowanie szablonów (zewnętrznych i inline) i dopasowanie: selektory komponentów
  i dyrektyw (w tym atrybutowe i strukturalne), pipe'y w wyrażeniach, `ng-template`/`ngComponentOutlet`.
- FR-3.2 Użycie w szablonie liczy się jako użycie symbolu (komponent użyty tylko w HTML nie jest martwy).
- FR-3.3 Nowa składnia sterująca (`@if/@for/@switch/@defer`) oraz klasyczne `*ngIf/*ngFor`.

### FR-4. Graf zależności i statystyki
- FR-4.1 Trzy poziomy grafu: symbol → symbol, plik → plik, projekt → projekt (agregacja).
- FR-4.2 Statystyki użyć: dla każdej pary (pakiet A, pakiet B): liczba importów, lista symboli,
  liczba użyć per symbol.
- FR-4.3 Cykle: plikowe i pakietowe (algorytm SCC — Tarjan, iteracyjnie, nie rekurencja DFS).
- FR-4.4 Metryki pakietu: Ca (afferent), Ce (efferent), I = Ce/(Ca+Ce), liczba eksportów,
  odsetek eksportów faktycznie używanych ("API waste").
- FR-4.5 Graf jest częścią wyniku (dziś jest budowany i wyrzucany).

### FR-5. Detekcja nieużytków
- FR-5.1 Nieużywane eksporty (symbol eksportowany, nigdzie nie importowany/nieużyty w szablonie/routingu/DI).
- FR-5.2 Nieużywane pliki (żaden symbol pliku nie jest osiągalny z entry pointów: `main.ts`,
  routing, `index.ts` publicznego API — analiza osiągalności, nie tylko "0 importów").
- FR-5.3 Nieużywane byty Angulara z semantyką: deklaracja w NgModule bez użycia w żadnym szablonie;
  standalone component nieimportowany nigdzie i nieużyty w routingu.
- FR-5.4 Eksporty "publicznego API" (`index.ts` pakietu), których nie używa żaden inny pakiet
  (nadmiarowe API — osobna kategoria, bo wewnątrz pakietu mogą być używane).
- FR-5.5 Mechanizm wykluczeń (adnotacje/konfiguracja, np. symbole używane przez Storybook/testy)
  + rozróżnienie użyć produkcyjnych od testowych (`.spec.ts` liczone osobno).

### FR-6. Kandydaci do refaktoryzacji
- FR-6.1 "Move candidate": symbol z pakietu A używany tylko przez pakiet B (próg konfigurowalny,
  np. ≥90% użyć w B) → sugestia przeniesienia z uzasadnieniem i listą użyć.
- FR-6.2 "Split candidate": pakiet, którego dwa rozłączne podzbiory eksportów są używane przez
  rozłączne zbiory konsumentów (niska kohezja).
- FR-6.3 Naruszenia granic: reguły na tagach NX (np. `type:feature` nie może importować `type:app`),
  kompatybilne koncepcyjnie z `@nx/enforce-module-boundaries`.

### FR-7. Raportowanie i CLI
- FR-7.1 Podkomendy: `analyze` (pełny JSON), `stats`, `unused`, `cycles`, `move-candidates`,
  `boundaries`, `graph --format json|dot|html`.
- FR-7.2 SARIF dla `unused`/`cycles`/`boundaries` (integracja z GitHub code scanning).
- FR-7.3 `--baseline plik.json` — raportuj tylko nowe problemy (adopcja w brownfield).
- FR-7.4 Kody wyjścia przyjazne CI (`--fail-on new-cycles,unused`).
- FR-7.5 Raport HTML: interaktywny graf pakietów (klik → pliki/symbole), tabele statystyk.

## 4. Wymagania niefunkcjonalne
- NFR-1 Wydajność: workspace ~2000 plików TS w < 5 s na laptopie (M-serii); parsowanie i analiza
  równoległe; każdy plik parsowany **dokładnie raz** (dziś barrels są re-parsowane wielokrotnie).
- NFR-2 Deterministyczny output (stabilne sortowanie) — warunek snapshot testów i baseline'ów.
- NFR-3 Poprawność > kompletność: kategoria "unused" nie może mieć false positives na ścieżkach
  objętych fixturami (lazy routes, szablony, DI); wątpliwe przypadki raportowane jako "low confidence".
- NFR-4 Testy: fixture'owe workspace'y NX + snapshot testy (`insta`), pokrycie każdej reguły analizy.
- NFR-5 Zero zależności od zainstalowanego Node/NX w runtime (czysta analiza statyczna).

## 5. Poza zakresem (v1)
- Analiza SCSS/CSS usage, i18n, monorepa inne niż NX (Turborepo itd.), automatyczne wykonywanie
  refaktoryzacji (tylko sugestie), analiza wersji zależności zewnętrznych z rejestru npm.

## 6. Miary sukcesu
- Na fixture'ach: 100% zgodności ze snapshotami (zero false positive/negative w zdefiniowanych ścieżkach).
- Na realnym workspace referencyjnym (ddd-hrm): ręczna weryfikacja próbki 20 zgłoszeń "unused" — ≥95% trafność.
- Czas analizy referencyjnego workspace'a < 2 s.

## 7. Kontekst konkurencyjny / state of the art

Czego używają najlepsze narzędzia w tej klasie i co z tego bierzemy:

| Narzędzie / technika | Co robi dobrze | Co bierzemy |
|---|---|---|
| **knip** | Nieużywane pliki/eksporty/zależności w JS/TS; entry-point reachability; obsługa monorepo | Model "osiągalności z entry pointów" zamiast naiwnego "0 importów"; system wykluczeń |
| **Nx project graph** + `enforce-module-boundaries` | Graf projektów, reguły na tagach | Reguły granic na tagach NX; format grafu zgodny mentalnie z `nx graph` |
| **dependency-cruiser** | Konfigurowalne reguły zależności, wiele formatów output (dot, html) | Silnik reguł + output DOT/HTML |
| **madge** | Cykle plikowe, wizualizacja | Cykle jako first-class feature (ale SCC zamiast DFS) |
| **ngtsc / Angular Language Service** | Semantyka szablonów: dopasowanie selektorów, pipe'ów, scope komponentu | Model "co jest widoczne w szablonie" (imports standalone / NgModule scope) |
| **SCIP/LSIF (Sourcegraph)** | Precyzyjny indeks symbol→użycia | Poziom symbolu (nie tylko pliku) w grafie; ewentualny eksport do SCIP w przyszłości |
| **CodeScene** | Hotspoty = churn × złożoność | (v2) Skrzyżowanie grafu z historią git: martwy kod, który dodatkowo nikt nie dotyka |
| **Metryki R. Martina** | Ca/Ce, Instability, Abstractness, Distance | Metryki pakietów w `stats` |
| **oxc_resolver (Rust crate)** | Pełna, przetestowana implementacja rezolucji Node + tsconfig paths/extends | **Zastępuje ręcznie pisany resolver** — eliminuje całą klasę bugów |
| **petgraph (Rust crate)** | Algorytmy grafowe (Tarjan SCC, toposort) | Rdzeń grafu zamiast własnych DFS-ów |
| **insta / criterion** | Snapshot testy / benchmarki w Rust | Infrastruktura testowa |

Kluczowa obserwacja: żadne z istniejących narzędzi nie łączy jednocześnie (a) świadomości NX
(tagi, projekty, publiczne API przez `index.ts`), (b) semantyki Angulara (szablony, DI, lazy routes,
standalone imports) i (c) szybkości natywnego binarki. To jest nisza tego projektu.
