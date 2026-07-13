# Angular Semantics

nx-analyzer understands Angular beyond the TypeScript surface — this is what makes its dead-code and usage analysis trustworthy in Angular workspaces.

## Entities and metadata

| Entity | Extracted metadata |
|---|---|
| `@Component` | selector, `templateUrl` / inline `template`, `styleUrls`/`styleUrl`, `standalone` (with the Angular 19 default), `imports` (standalone scope), `providers`, inputs & outputs |
| `@Directive` | selector, `standalone`, host bindings, inputs & outputs |
| `@Pipe` | name, `pure`, `standalone` |
| `@Injectable` | `providedIn` (missing argument object handled — `@Injectable()` is valid) |
| `@NgModule` | `declarations`, `imports`, `exports`, `providers`, `bootstrap` |

Inputs/outputs cover **both** styles:

```ts
@Input() legacyTitle = '';
@Output() legacyClosed = new EventEmitter<void>();

label = input.required<string>();     // signal inputs
variant = input<'a' | 'b'>('a');
counter = model(0);
dismissed = output<void>();
```

Classes are found in every position: `export class`, non-exported `class`, `export default class`, and `class X {} … export { X }`.

## Template analysis

Both external (`templateUrl`) and inline (`template:`) templates are scanned with a Angular-aware HTML scanner that handles:

- binding sugar: `[prop]`, `(event)`, `[(model)]`, `*structural`, `[attr.x]`,
- the new control flow (`@if`, `@for`, `@switch`, `@defer` blocks) **and** classic `*ngIf`/`*ngFor`,
- interpolations `{{ … }}` and pipes in binding expressions (`[title]="price | uiCurrency"`), including `||` disambiguation.

Selectors are matched with real CSS-selector semantics: element (`ui-button`), attribute (`[uiTooltip]`), compound (`button[uiBtn]`), class parts, comma-separated alternatives. `:not(...)` is ignored *conservatively* — it can only over-match, never miss a usage, which is the safe direction for dead-code analysis.

A template match creates a dependency edge (component file → target entity file), so template-only usage keeps entities alive and participates in cycles and statistics.

## Dependency injection

Usage is recognized through:

- `inject(ApiService)` calls,
- constructor parameter types (`constructor(private api: ApiService)`),
- provider shapes: `useClass:`, `useExisting:`, provider arrays, `InjectionToken<T>` generic arguments,
- any other identifier or type reference to an imported symbol.

## Lazy routes

```ts
{ path: 'shop', loadChildren: () => import('@scope/feature-shop').then(m => m.routes) }
{ path: 'page', loadComponent: () => import('@scope/page').then(m => m.PageComponent) }
```

Dynamic imports become **lazy edges**. Everything transitively re-exported by a lazy-loaded barrel counts as reachable — a feature library wired only through routes is never reported dead.

## Standalone default (Angular 19+)

The workspace `package.json` is checked for `@angular/core`; on major ≥ 19 components/directives/pipes without an explicit `standalone:` flag are treated as standalone, matching compiler behavior.
