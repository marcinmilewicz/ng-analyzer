// Workspace package: NOT in tsconfig paths — reachable only through the
// node_modules symlink + package.json "main" pointing at src/index.ts.
import { widgetThing, DefaultWidget } from '@fix/wpkg';
// Alias re-export chain: relative file whose export comes from '@fix/models'.
import type { Rect } from '../../../bridge/src/lib/bridge-types';

export function Consumer() {
  const rect: Rect = { x: 0, y: 0, width: Number(widgetThing()), height: 1 };
  return <DefaultWidget key={rect.width} />;
}
