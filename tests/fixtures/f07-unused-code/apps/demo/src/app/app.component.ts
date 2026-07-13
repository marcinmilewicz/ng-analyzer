import { Component, inject } from '@angular/core';
import {
  InjectOnlyService,
  TemplateOnlyComponent,
  WiredNotRenderedComponent,
} from '@fix/stuff';

@Component({
  selector: 'fix-root',
  templateUrl: './app.component.html',
  standalone: true,
  imports: [TemplateOnlyComponent, WiredNotRenderedComponent],
})
export class AppComponent {
  private service = inject(InjectOnlyService);
}
