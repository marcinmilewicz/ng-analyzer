import { Component } from '@angular/core';

@Component({
  selector: 'fix-tree',
  template: '<div><fix-tree *ngIf="hasChildren"></fix-tree></div>',
  standalone: true,
})
export class OrphanTreeComponent {
  hasChildren = false;
}
