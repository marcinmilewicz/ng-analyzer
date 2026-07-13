import { Injectable } from '@angular/core';
import { helperOne as one } from './helpers';
import * as helpers from './helpers';
import './polyfill';

@Injectable({ providedIn: 'root' })
export class ImportVariantsService {
  total = one() + helpers.helperTwo();
}
