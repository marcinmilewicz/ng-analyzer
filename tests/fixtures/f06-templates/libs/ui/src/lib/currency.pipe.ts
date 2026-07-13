import { Pipe, PipeTransform } from '@angular/core';

@Pipe({
  name: 'uiCurrency',
  standalone: true,
})
export class UiCurrencyPipe implements PipeTransform {
  transform(value: number): string {
    return `${value} PLN`;
  }
}
