import { Injectable } from '@angular/core';
// NodeNext/bundler style: .js extension in the specifier, file is .ts
import { esmHelper } from './helper.js';
import type { ModernModel } from './model.js';

@Injectable({ providedIn: 'root' })
export class EsmStyleService {
  value = esmHelper();
  model?: ModernModel;
}
