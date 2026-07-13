import { Injectable } from '@angular/core';

@Injectable({ providedIn: 'root' })
class InternalService {}

export function makeInternal(): InternalService {
  return new InternalService();
}
