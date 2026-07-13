import { Component } from '@angular/core';
import { BadgeComponent } from './badge.component';

@Component({
  selector: 'fix-panel',
  templateUrl: './panel.component.html',
  imports: [BadgeComponent],
  providers: [PanelStateService],
})
export class PanelComponent {}

export class PanelStateService {}
