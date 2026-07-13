import { add, Calculation } from '@fix/toolbox';

export function calculate(a: number, b: number): Calculation {
  return { result: add(a, b) };
}

export async function lazyRounding(): Promise<unknown> {
  const mod = await import('@fix/toolbox');
  return mod.RoundingMode;
}
