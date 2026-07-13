import { Component } from '@angular/core';
import { FixBtnDirective } from './btn.directive';
import { UiHasPipe } from './has.pipe';
import { UiSortPipe } from './sort.pipe';

@Component({
  selector: 'fix-host',
  templateUrl: './host.component.html',
  standalone: true,
  imports: [FixBtnDirective, UiHasPipe, UiSortPipe],
})
export class HostComponent {
  items = [3, 1, 2];
}
