import { xOnly } from '@fix/feature-x';

export function yHelper(): string {
  return 'y' + xOnly();
}
