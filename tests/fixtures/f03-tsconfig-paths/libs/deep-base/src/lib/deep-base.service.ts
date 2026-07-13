import { Injectable } from '@angular/core';
import { sharedHelper } from 'shared/helper';

@Injectable({ providedIn: 'root' })
export class DeepBaseService {
  value = sharedHelper();
}
