import { yHelper } from '@fix/feature-y';

export function xHelper(): string {
  return 'x' + yHelper();
}

export function xOnly(): string {
  return 'x';
}
