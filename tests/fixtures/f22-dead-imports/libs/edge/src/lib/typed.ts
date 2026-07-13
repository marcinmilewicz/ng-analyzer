// Negative control for the used-import filter: `Shape` is referenced ONLY in
// a type position. If the visitor missed type identifiers, this import would
// look dead and every type-only import in a real workspace would be wrongly
// reported — so this guards the whole filter.
import { Shape } from './shape';

export function area(shape: Shape): number {
  return shape.width * shape.height;
}
