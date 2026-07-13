# Analiza konkurencji i kierunki rozwoju — ngAnalyzer

Wersja: 1.0 (2026-07-11). Uzupełnia `PRD.md` (sekcja 7) o świeży przegląd rynku
i ocenę trzech kierunków: wsparcie React, wsparcie czystego TS/JS, wizualizacja.

## 1. Mapa konkurencji (stan: lipiec 2026)

| Narzędzie | Kategoria | Co robi dobrze | Czego nie robi (nasza szansa) |
|---|---|---|---|
| **knip** | martwy kod JS/TS | 150+ pluginów (Nx, Next, Storybook, Jest…), reachability z entry pointów, auto-fix (usuwa martwe eksporty), monorepo first-class, **MCP server + LSP dla agentów AI** | Zero semantyki frameworków UI: nie wie, że komponent Angulara "użyty w szablonie" albo React "użyty w JSX" to użycie; brak statystyk powiązań i metryk couplingu |
| **ngx-unused** | martwy kod Angular | Rozumie użycia w szablonach; działa na NX | Wolny (ts-morph/tsc), tylko jedna analiza (unused), brak grafu, statystyk, cykli, granic |
| **Angular compiler (NG8113)** | diagnostyka | "Unused standalone imports" wbudowane w ngtsc | Tylko scope jednego komponentu; nic między pakietami |
| **dependency-cruiser** | graf + reguły | Silnik reguł architektury (severity, orphans, reachability), wiele formatów (dot/svg/html), dojrzały CI | Poziom plików/modułów, nie symboli; brak semantyki frameworków; JS-based (wolny na dużych repo) |
| **madge / skott** | graf + cykle | Skott: interaktywny raport HTML, eksport mermaid/svg/json, szybszy od madge | Tylko import graph; brak monorepo-awareness, brak unused na poziomie symboli |
| **Nx graph** | graf projektów | Nowe UI (composite mode, tracing ścieżek, panel szczegółów), zna targets/tags, `affected` | Poziom projektów, nie plików/symboli; nie odpowie "który eksport jest martwy" ani "co przenieść" |
| **react-scanner / Omlet** | analityka komponentów React | Zliczanie użyć komponentów i **propsów** (adopcja design systemu, dashboard, mapowanie na CODEOWNERS) | Tylko React; brak grafu zależności, unused, monorepo-semantyki |
| **CodeScene / CodeCharta** | wizualizacja + metryki | Hotspoty (churn × złożoność), zdrowie kodu, mapy interaktywne | Ciężkie/komercyjne; nie znają semantyki Angular/NX; nie wskażą konkretnego martwego symbolu |
| **CodeSee** | mapy kodu | (przejęte przez GitKraken, produkt zamknięty 2024) — popyt na alternatywy istnieje | — |

Wnioski przekrojowe:
1. **Nikt nie łączy** semantyki frameworka UI + świadomości NX (tagi, publiczne API) +
   poziomu symboli + szybkości natywnej binarki. To pozostaje naszą niszą (potwierdzenie PRD §7).
2. **Wizualizacja jest standardem rynkowym** — skott, dependency-cruiser, Nx graph mają ją w
   podstawie; sam JSON to za mało, żeby narzędzie było używane przez ludzi (nie tylko CI).
3. **Analityka propsów/inputów** (react-scanner, Omlet) to wzorzec, którego **nikt nie przeniósł
   na Angular** (użycia `input()`/`output()` per komponent) — tani, unikatowy diff.
4. **Integracja z agentami AI** (knip: MCP server) to nowy kanał dystrybucji narzędzi
   deweloperskich — graf zależności jako kontekst dla agenta refaktorującego.

## 2. Ocena proponowanych kierunków

### 2.1 React w NX workspace — TAK, przez architekturę pluginową (wysoka wartość, umiarkowany koszt)

Fundament (discover → parse → resolve → graf → unused/stats/cycles) jest **niezależny od
frameworka** — Angular to tylko jeden "ekstraktor semantyki". Właściwy ruch to nie "dopisać
Reacta", tylko rozdzielić rdzeń od pluginów frameworkowych (wzorzec knip):

- **core**: pliki, importy/eksporty, tsconfig, graf, cykle, unused-exports, statystyki pakietów
  — działa dla KAŻDEGO kodu TS/JS w NX;
- **plugin angular**: dekoratory, szablony, DI, lazy routes (M2 bez zmian);
- **plugin react** (nowy): komponenty (funkcje zwracające JSX + `forwardRef`/`memo`), użycia
  przez elementy JSX, hooki, `React.lazy()` jako lazy edge, **statystyki propsów**
  (parytet z react-scanner, ale szybszy i z grafem w tle).

Warunek wstępny (tani, do zrobienia od razu): dziś parser ma zahardkodowane `tsx: false`
i filtr tylko `.ts` — **pliki `.tsx` w ogóle nie są analizowane**, więc nawet importy
z projektów React w workspace są niewidoczne. Włączenie `tsx: true` dla `.tsx` + rozszerzenie
filtra to godzina pracy i naprawia analizę mieszanych workspace'ów.

### 2.2 Bezframeworkowy JS/TS w NX — TAK, i to prawie za darmo (niski koszt)

Biblioteki utili/modeli w NX to zwykły TS — core po M1 (ekstrakcja wszystkich eksportów,
graf symboli) obsłuży je w całości: unused exports, cykle, statystyki, move-candidates.
Do zrobienia: rozszerzenia `.js/.mjs/.cjs/.mts/.cts` w resolverze i filtrach oraz fixture F14
(czysta biblioteka TS bez Angulara). Czyli: nie "nowy feature", tylko domknięcie M1.

### 2.3 Wizualizacja wyników — TAK, podnieść priorytet (wysoka wartość)

Rynek pokazuje trzy poziomy; proponuję wszystkie, w kolejności kosztu:
1. **Eksport DOT + Mermaid** (M4, ~dzień) — mermaid renderuje się w GitHub/GitLab MD,
   czyli raport w PR bez żadnej infrastruktury (wzorzec skott).
2. **Samowystarczalny raport HTML** (M4, podniesiony priorytet): jeden plik, dane wbudowane,
   graf pakietów w trybie composite (wzorzec Nx graph — domyślnie zwinięte pakiety, drill-down
   do plików/symboli), panele: unused, cykle, macierz pakiet→pakiet, metryki Ca/Ce/I.
   Rendering: cytoscape.js lub d3-force, wszystko inline.
3. **Nakładka hotspotów** (M5, opcja): kolorowanie węzłów churn × liczba zależnych
   (wzorzec CodeScene) — wymaga integracji z git, już zaplanowanej.

## 3. Nowe pomysły wynikające z analizy (poza pytaniami)

| Pomysł | Wzorzec z rynku | Wartość / koszt |
|---|---|---|
| **Tryb MCP server** (`nx-analyzer serve --mcp`): agent pyta "kto używa X?", "co mogę usunąć?" | knip @knip/mcp | Wysoka / średni — naturalny kanał w 2026; Rust: szybkie odpowiedzi na żywym grafie |
| **Analityka inputów/outputów Angulara** (które inputy `UiButton` są używane, z jakimi wartościami literalnymi) | react-scanner / Omlet (tylko React) | Wysoka / średni — unikat na rynku Angular, mierzy adopcję design systemu |
| **Auto-fix** (usuwanie martwych eksportów/plików za flagą `--fix`) | knip | Średnia / wysoki — dopiero po zaufaniu do detekcji (M3+, za flagą eksperymentalną) |
| **Reguły reachability** ("nic z `scope:admin` nie może być osiągalne z `scope:public`") | dependency-cruiser | Średnia / niski — rozszerzenie silnika granic z M3 |
| **`--affected --base=main`** (analiza tylko projektów dotkniętych zmianą) | Nx affected | Średnia / średni — przyspiesza CI na dużych repo |
| Orphan detection (plik bez żadnych krawędzi) | dependency-cruiser | Niska / trywialny — wypada z grafu za darmo |

## 4. Wpływ na roadmapę

- **M1** (bez zmian) + domknięcie: `.tsx`/`tsx:true`, rozszerzenia `.js/.mjs/.cjs`, fixture F14 (czysty TS).
- **M2** (bez zmian): semantyka Angulara.
- **M3** (bez zmian): analizy; + orphans (za darmo).
- **M4** (rozszerzone): DOT + **Mermaid** + interaktywny HTML (composite, drill-down) — podniesiony priorytet względem SARIF (SARIF zostaje, ale HTML pierwszy).
- **M5** (bez zmian): incremental cache, watch, hotspoty git.
- **M6 (nowy)**: architektura pluginowa formalnie + **plugin React** (komponenty, JSX usage, lazy(), statystyki propsów) + fixture F15 (aplikacja React w NX).
- **M7 (nowy, opcjonalny)**: tryb MCP server + analityka inputów Angulara.

Ryzyko zakresu: React plugin i MCP nie mogą wyprzedzić M2–M3 — przewaga konkurencyjna
bierze się z głębi semantyki Angulara, nie z szerokości. Kolejność: głębia → szerokość.
