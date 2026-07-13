import { Injectable } from '@angular/core';
import { Config, fromB } from '@fix/alpha';

@Injectable({ providedIn: 'root' })
export class BarrelUserService {
  config?: Config;
  value = fromB;
}
