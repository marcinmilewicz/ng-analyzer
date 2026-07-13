export function formatPrice(value: number): string {
  return `${value} PLN`;
}

export function formatDate(value: Date): string {
  return value.toISOString();
}

export function formatLocal(value: string): string {
  return value.toUpperCase();
}
