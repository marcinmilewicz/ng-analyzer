import { Injectable } from '@angular/core';
import { OrderModel } from './model';

@Injectable({ providedIn: 'root' })
export class OrdersService {
  orders: OrderModel[] = [];
}
