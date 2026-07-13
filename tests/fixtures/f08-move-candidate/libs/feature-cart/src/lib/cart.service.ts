import { Injectable } from '@angular/core';
import { formatDate, formatLocal } from '@fix/shared-utils';

@Injectable({ providedIn: 'root' })
export class CartService {
  when(): string {
    return formatDate(new Date()) + formatLocal('cart');
  }
}
