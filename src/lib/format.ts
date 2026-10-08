const pesos = new Intl.NumberFormat("es-MX", { style: "currency", currency: "MXN" });

/** Pesos as a person reads them, «$250,000.00». Display only: every figure is added up in Rust. */
export function formatMxn(n: number): string {
  return pesos.format(n);
}

/** The number written by a person («1,500.50», «$ 1500», «1 500») as a number, or NaN if it is not one. */
export function toNumber(text: string): number {
  const cleaned = text.replace(/[\s$,]/g, "");
  return cleaned === "" ? Number.NaN : Number(cleaned);
}

/** A date as a person reads it, «1 de octubre de 2026». Empty if the text is not a date. */
export function formatDate(iso: string): string {
  const d = new Date(iso);
  return Number.isNaN(d.getTime()) ? "" : d.toLocaleDateString("es-MX", { day: "numeric", month: "long", year: "numeric" });
}

/**
 * An amount as people write it: «1800000», «1,800,000», «$1 800 000». The same reading as `parse_pesos` in Rust
 * (separators only between groups of three digits); `null` when it is not an amount.
 */
export function parsePesos(v: string): number | null {
  const s = v.trim().replace(/^\$\s*/, "");
  if (!/^(\d+|\d{1,3}([,. ]\d{3})+)$/.test(s)) return null;
  return Number(s.replace(/[,. ]/g, ""));
}
