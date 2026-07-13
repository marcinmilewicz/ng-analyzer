import { Pipe, PipeTransform } from '@angular/core';

@Pipe({ name: 'uiSort', standalone: true })
export class UiSortPipe implements PipeTransform {
  transform(value: number[]): number[] {
    return [...value].sort();
  }
}
