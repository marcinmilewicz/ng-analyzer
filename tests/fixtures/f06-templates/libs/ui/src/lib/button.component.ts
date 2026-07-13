import { Component } from '@angular/core';

@Component({
  selector: 'ui-button',
  template: '<button><ng-content /></button>',
  standalone: true,
})
export class UiButtonComponent {}
