import { Injectable } from '@angular/core';
import { CustomerModel } from './model';

@Injectable({ providedIn: 'root' })
export class CustomersService {
  customers: CustomerModel[] = [];
}
