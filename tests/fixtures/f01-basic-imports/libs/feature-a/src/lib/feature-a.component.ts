import { Component } from '@angular/core';
import { UiButtonComponent, ButtonConfig } from '@fix/ui';

@Component({
  selector: 'fix-feature-a',
  templateUrl: './feature-a.component.html',
  standalone: true,
})
export class FeatureAComponent {
  primary: ButtonConfig = { label: 'Primary' };
  button = UiButtonComponent;
}
