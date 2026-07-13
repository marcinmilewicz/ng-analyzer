import { Injectable } from '@angular/core';
import { formatDate, formatPrice } from '@fix/shared-utils';

@Injectable({ providedIn: 'root' })
export class CheckoutService {
  total(): string {
    return formatPrice(100) + formatDate(new Date());
  }
}
