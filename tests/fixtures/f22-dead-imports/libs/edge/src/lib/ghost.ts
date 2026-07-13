// Nobody calls this. Its ONLY inbound reference is the leftover import
// statement in zombie.ts — which never references the binding. Counting that
// statement as a usage is what used to hide this symbol.
export function ghost(): number {
  return 1;
}
