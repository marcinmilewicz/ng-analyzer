import { Component } from '@angular/core';
import { ButtonConfig } from './button.model';

@Component({
  selector: 'fix-button',
  templateUrl: './button.component.html',
  styleUrls: ['./button.component.css'],
  standalone: true,
})
export class UiButtonComponent {
  config?: ButtonConfig;
}
