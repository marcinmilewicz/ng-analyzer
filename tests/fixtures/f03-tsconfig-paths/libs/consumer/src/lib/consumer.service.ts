import { Injectable } from '@angular/core';
import { sharedHelper } from 'shared/helper';
import { MultiExport } from '@fix/multi';

@Injectable({ providedIn: 'root' })
export class ConsumerService {
  value = sharedHelper();
  multi?: MultiExport;
}
