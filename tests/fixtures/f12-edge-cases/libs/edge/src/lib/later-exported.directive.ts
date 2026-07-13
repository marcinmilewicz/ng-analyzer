import { Directive } from '@angular/core';

@Directive({
  selector: '[fixLaterExported]',
  standalone: true,
})
class LaterExportedDirective {}

export { LaterExportedDirective };
