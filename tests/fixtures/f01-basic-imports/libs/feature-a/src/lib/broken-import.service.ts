import { Injectable } from '@angular/core';
import { Missing } from './does-not-exist';

@Injectable({ providedIn: 'root' })
export class BrokenImportService {
  missing?: Missing;
}
