import { adminReport } from '@fix/feature-admin';
import { uiButton } from '@fix/ui-kit';

export function shopPage(): string {
  return uiButton() + adminReport();
}
