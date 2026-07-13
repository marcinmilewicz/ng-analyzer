// Next.js 16 proxy convention (renamed from middleware.ts) — the framework
// consumes `proxy` and `config` without importing them anywhere.
export function proxy(): void {}

export const config = {
  matcher: ['/((?!_next/static|favicon.ico).*)'],
};
