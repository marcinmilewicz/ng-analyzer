// Side-effect import across a project boundary: a real edge that names no
// symbol.
import '@fix/effects';

// Negative control: the binding IS referenced, so the import counts as a
// usage and `helper` must stay off the unused list.
import { helper } from './helper';

export function alive(): number {
  return helper() + 1;
}
