import { Injectable } from '@angular/core';
import { NestedService } from '@fix/nested';
import { ParentService } from '@fix/parent';

@Injectable({ providedIn: 'root' })
export class NestedConsumerService {
  constructor(
    private nested: NestedService,
    private parent: ParentService
  ) {}
}
