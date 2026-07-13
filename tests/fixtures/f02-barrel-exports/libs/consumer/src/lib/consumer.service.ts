import { Injectable } from '@angular/core';
import { DeepButton, UiCard } from '@fix/ui-kit';

@Injectable({ providedIn: 'root' })
export class BarrelConsumerService {
  button = new DeepButton();
  card?: UiCard;
}
