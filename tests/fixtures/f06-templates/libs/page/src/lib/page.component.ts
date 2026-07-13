import { Component } from '@angular/core';
import {
  UiButtonComponent,
  UiCurrencyPipe,
  UiIfDirective,
  UiTooltipDirective,
  UnusedInTemplateComponent,
} from '@fix/ui';

@Component({
  selector: 'fix-page',
  templateUrl: './page.component.html',
  standalone: true,
  imports: [
    UiButtonComponent,
    UiTooltipDirective,
    UiIfDirective,
    UiCurrencyPipe,
    UnusedInTemplateComponent,
  ],
})
export class PageComponent {
  price = 42;
  items = [1, 2, 3];
  show = true;
}
