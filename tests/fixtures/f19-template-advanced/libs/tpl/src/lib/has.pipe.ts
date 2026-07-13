import { Pipe, PipeTransform } from '@angular/core';

@Pipe({ name: 'uiHas', standalone: true })
export class UiHasPipe implements PipeTransform {
  transform(value: unknown[]): boolean {
    return value.length > 0;
  }
}
