/**
 * src/lib/utils/currency.ts — Currency normalization and aggregation helpers.
 */

export interface CurrencyValueInput {
  currency?: string | null;
  value?: number | null;
}

export interface CurrencyTotal {
  currency: string;
  total: number;
  count: number;
}

const ISO_CURRENCY_RE = /^[A-Z]{3}$/;

/**
 * Normalize a user/backend currency code to uppercase ISO-like format.
 * Falls back to USD when missing/invalid.
 */
export function normalizeCurrencyCode(currency: string | null | undefined, fallback = 'USD'): string {
  const normalized = (currency ?? '').trim().toUpperCase();
  return ISO_CURRENCY_RE.test(normalized) ? normalized : fallback;
}

/**
 * Aggregate numeric values by currency code.
 */
export function sumByCurrency(items: CurrencyValueInput[]): CurrencyTotal[] {
  const totals = new Map<string, CurrencyTotal>();

  for (const item of items) {
    const value = Number(item.value ?? 0);
    if (!Number.isFinite(value)) continue;

    const currency = normalizeCurrencyCode(item.currency);
    const current = totals.get(currency);

    if (current) {
      current.total += value;
      current.count += 1;
      continue;
    }

    totals.set(currency, {
      currency,
      total: value,
      count: 1,
    });
  }

  return [...totals.values()].sort((a, b) => {
    const byMagnitude = Math.abs(b.total) - Math.abs(a.total);
    if (byMagnitude !== 0) return byMagnitude;
    return a.currency.localeCompare(b.currency);
  });
}
