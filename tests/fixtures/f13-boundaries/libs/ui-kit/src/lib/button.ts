import { shopPage } from '@fix/feature-shop';
import { sharedUtil } from '@fix/util-lib';

export function uiButton(): string {
  return sharedUtil() + shopPage();
}
