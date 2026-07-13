# Specyfikacja fixture'ów testowych — ngAnalyzer

Każdy fixture to **miniaturowy, samodzielny workspace NX** w `tests/fixtures/<nazwa>/`
(bez node_modules; pliki `@angular/*` importowane, ale nierozwiązywalne — analiza ma je
klasyfikować jako zależność zewnętrzną, nie błąd). Do każdego fixture'a należy snapshot
oczekiwanego wyniku (`insta`) + asercje celowane w konkretną regułę.

Wspólny szkielet (o ile fixture nie mówi inaczej):

```
tests/fixtures/<nazwa>/
├── nx.json
├── package.json
├── tsconfig.base.json          # paths: @fix/* → libs/*/src/index.ts
├── apps/demo/
│   ├── project.json            # type: application, tags wg potrzeb
│   ├── tsconfig.json           # extends ../../tsconfig.base.json
│   └── src/main.ts, app/…
└── libs/<lib>/
    ├── project.json
    ├── tsconfig.json
    └── src/index.ts, lib/…
```

## F01 — basic-imports (ścieżka: podstawowa rezolucja i statystyki)
- 2 liby: `feature-a` importuje `UiButtonComponent` z `ui` (2 użycia w TS), `util` nieimportowany przez nikogo.
- **Oczekiwane:** krawędź feature-a→ui z licznikiem 2; `util`: Ca=0; komponenty/serwisy wykryte z poprawnym `package_name`.
- **Testuje bugi:** B1 (dwa pliki importują `./model` z różnych katalogów), B6 (import nieistniejącej ścieżki → brak krawędzi, warning).

## F02 — barrel-exports (ścieżka: przejście po barrelach)
- `ui/src/index.ts`: `export * from './lib/button'`, `export { Card as UiCard } from './lib/card'`,
  `export * as tokens from './lib/tokens'`, re-eksport łańcuchowy 3 poziomy w głąb.
- **Oczekiwane:** każdy import z `@fix/ui` zresolvowany do pliku deklaracji (nie do index.ts); alias `UiCard` poprawnie zmapowany na `Card`.

## F03 — tsconfig-paths (ścieżka: konfiguracja TS)
- Łańcuch `tsconfig.json → tsconfig.app.json → tsconfig.base.json` (extends 2 poziomy).
- Aliasy: `@fix/ui`, **`shared/*` (bez `@`)**, alias wielowariantowy (`paths` z 2 wpisami, pierwszy nieistniejący).
- Jeden projekt **bez** `tsconfig.json` obok `project.json` (jest tylko `tsconfig.lib.json`).
- Jeden `project.json` **bez pól `name` i `sourceRoot`**.
- **Testuje bugi:** B9, B10, B11, B12.

## F04 — standalone-components (ścieżka: Angular 15+/19)
- Komponent standalone z `imports: [UiButtonComponent, DatePipe, RouterLink]`.
- Komponent **bez** `standalone:` w projekcie Angular 19 (ma być traktowany jako standalone).
- Inline `template:` (bez templateUrl), `styleUrl` (singular), sygnały: `input()`, `output()`, `model()`.
- **Oczekiwane:** tablica `imports` komponentu w wyniku; krawędzie użycia do UiButtonComponent; inputs/outputs wykryte.

## F05 — ngmodule-classic (ścieżka: aplikacje legacy)
- Lib z NgModule: `declarations: [A, B]`, `imports: [CommonModule, SharedModule]`, `exports: [A]`, `providers: [Svc]`.
- **Oczekiwane:** pełne metadane modułu ze zresolvowanymi symbolami (nie gołe stringi); `B` zadeklarowany, nieeksportowany.

## F06 — templates (ścieżka: użycia w HTML)
- `PageComponent` (templateUrl) używa w HTML: `<ui-button>` (element), `[uiTooltip]` (dyrektywa atrybutowa),
  `*uiIf` (strukturalna), `{{ x | uiCurrency }}` (pipe), `@if/@for` (nowa składnia) i `*ngIf` (stara).
- `UnusedInTemplateComponent` jest w `imports` komponentu, ale NIE występuje w HTML.
- **Oczekiwane:** krawędzie typu `template-selector`/`template-pipe`; `UnusedInTemplateComponent` zgłoszony jako "imported but not used in template".

## F07 — unused-code (ścieżka: detekcja martwego kodu)
- `DeadComponent` (nigdzie nieimportowany, nie w żadnym szablonie), `DeadService`, `deadUtil()` eksportowany z barrela,
  nieużywany eksport typu (`interface DeadModel`), cały martwy plik `orphan.ts`.
- Kontrprzykłady (NIE mogą być zgłoszone): komponent użyty tylko w szablonie; serwis użyty tylko przez `inject()`;
  symbol użyty tylko w `main.ts`; symbol użyty **tylko w `*.spec.ts`** (kategoria "test-only", nie "unused").
- **Oczekiwane:** dokładnie 5 zgłoszeń unused + 1 test-only; zero false positives.

## F08 — move-candidate (ścieżka: sugestie przeniesień)
- `shared-utils` eksportuje `formatPrice()` używane 4× wyłącznie w `feature-checkout` oraz `formatDate()`
  używane w 3 różnych pakietach.
- **Oczekiwane:** `formatPrice` → kandydat przeniesienia do `feature-checkout` (koncentracja 100%); `formatDate` → brak sugestii.

## F09 — circular-deps (ścieżka: cykle)
- Cykl plikowy wewnątrz liba (a.ts→b.ts→c.ts→a.ts) + cykl pakietowy (`feature-x` ⇄ `feature-y`)
  + cykl przechodzący przez węzeł współdzielony z innym cyklem (przypadek, który obecny DFS gubi — B7).
- **Oczekiwane:** 2 SCC plikowe, 1 SCC pakietowy, każdy z przykładową ścieżką cyklu.

## F10 — lazy-routes (ścieżka: routing i dynamiczne importy)
- `app.routes.ts`: `loadChildren: () => import('@fix/feature-lazy').then(m => m.routes)`
  oraz `loadComponent: () => import('@fix/feature-page/page.component').then(m => m.PageComponent)`.
- `feature-lazy` nie ma ŻADNEGO statycznego importu z zewnątrz.
- **Oczekiwane:** `feature-lazy` osiągalny (krawędź `lazy-route`), NIE zgłoszony jako unused.

## F11 — di-providers (ścieżka: dependency injection)
- `InjectionToken<Config>`, provider `{ provide: LOGGER, useClass: FileLogger }`, `useExisting`,
  `forwardRef(() => Svc)`, serwis wstrzykiwany wyłącznie przez `inject(ApiService)` (bez importu typu w konstruktorze),
  zależność w konstruktorze (`constructor(private api: ApiService)`).
- **Oczekiwane:** `FileLogger` używany (krawędź `di-token`), nie-unused mimo braku "zwykłego" użycia.

## F12 — edge-cases (ścieżka: warianty składni importu/eksportu)
- `export default class`, klasa dekorowana **niewyeksportowana**, `class X {}; export { X }`,
  import z aliasem (`import { A as B }`), import namespace (`import * as ui`), import side-effect (`import './polyfill'`),
  plik z błędem składni (analiza ma zgłosić warning i kontynuować, nie połknąć błędu po cichu),
  `export =`-style (ma być zignorowany z warningiem, nie crash).
- **Testuje bugi:** B5, błędy parsowania z `visitors/mod.rs:58`.

## F13 — boundaries (ścieżka: tagi NX)
- Projekty z tagami `type:feature`, `type:ui`, `type:util`, `scope:admin`, `scope:shop`.
- Reguła w konfiguracji: `type:ui` nie może importować `type:feature`; `scope:shop` nie może importować `scope:admin`.
- Fixture zawiera 1 naruszenie każdej reguły + poprawne zależności.
- **Oczekiwane:** dokładnie 2 naruszenia z lokalizacją importu.

## Infrastruktura testowa

- `tests/integration.rs`: dla każdego fixture'a uruchom pipeline in-process (nie subprocess),
  snapshot pełnego JSON przez `insta` (z sortowaniem deterministycznym).
- Testy celowane: asercje "must contain / must NOT contain" dla reguł (unused, cycles, move).
- `criterion` benchmark: syntetyczny fixture generowany (skrypt: N libów × M plików) — pilnuje NFR-1.
- CI: `cargo fmt --check && cargo clippy -- -D warnings && cargo test`.

## Kolejność implementacji fixture'ów

M0: F01, F03, F12 → M1: F02, F04, F05 → M2: F06, F10, F11 → M3: F07, F08, F09, F13.

## Fixtures dodane po M0 (wykrywanie regresji w realnych wzorcach)

### F16 — nested-projects
Projekt NX zagnieżdżony w katalogu innego projektu (`libs/parent` + `libs/parent/nested`).
**Wykrył bug:** pliki zagnieżdżonego projektu były przetwarzane dwukrotnie (raz jako `parent`, raz jako `nested`), psując statystyki i atrybucję. **Fix:** pruning korzeni zagnieżdżonych projektów w walku nadrzędnego.

### F17 — barrel-cycles
Cykliczne re-eksporty (`a.ts` ⇄ `b.ts` przez `export *`) + kolizja nazw (`Config` w dwóch pakietach, jeden użyty, drugi martwy).
**Potwierdził poprawność:** visited-set w rezolucji barreli (brak zapętlenia), klucze analiz per (plik, nazwa) — kolizje nazw rozróżniane, `usages` pokazuje obie deklaracje.

### F18 — modern-syntax
tsconfig w formacie JSONC (komentarze, trailing commas), importy w stylu NodeNext (`./helper.js` → `helper.ts`), `import type`.
**Wykrył 2 bugi:** (1) JSONC wywalał parser tsconfig → projekt tracił aliasy `paths`; fix: stripper JSONC przed serde. (2) specyfikatory `.js/.mjs/.cjs` nie mapowały się na źródła `.ts/.mts/.cts` → fałszywe "unused"; fix: mapowanie rozszerzeń w obu resolverach.

### F19 — template-advanced
Pipe'y w warunkach bloków control-flow (`@if (items | uiHas)`, `@for (... | uiSort)`), selektor złożony `button[fixBtn]` (kotwica negatywna: `<a fixBtn>` NIE może się dopasować), komponent rekurencyjny renderujący wyłącznie samego siebie.
**Wykrył 2 bugi:** (1) pipe'y w wyrażeniach `@if/@for` nie były skanowane → fałszywe "unused pipe"; fix: skan zbalansowanych nawiasów po `@słowo`. (2) samo-użycie w szablonie utrzymywało martwy komponent rekurencyjny przy życiu; fix: wykluczenie self-użyć z indeksu użyć.
