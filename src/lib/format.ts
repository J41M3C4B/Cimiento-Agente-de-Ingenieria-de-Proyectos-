/**
 * An amount as people write it: «1800000», «1,800,000», «$1 800 000». The same reading as `parse_pesos` in Rust
 * (separators only between groups of three digits); `null` when it is not an amount.
 */
export function parsePesos(v: string): number | null {
  const s = v.trim().replace(/^\$\s*/, "");
  if (!/^(\d+|\d{1,3}([,. ]\d{3})+)$/.test(s)) return null;
  return Number(s.replace(/[,. ]/g, ""));
}
