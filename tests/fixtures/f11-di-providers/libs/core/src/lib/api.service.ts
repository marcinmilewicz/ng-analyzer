import { Injectable } from '@angular/core';

@Injectable({ providedIn: 'root' })
export class ApiService {
  fetch(): string {
    return 'data';
  }
}
