import { Component } from '@angular/core';

@Component({
  selector: 'ui-unused',
  template: '<span>never rendered</span>',
  standalone: true,
})
export class UnusedInTemplateComponent {}
