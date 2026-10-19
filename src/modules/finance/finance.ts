import type { FinanceLine, Finances } from "../../lib/types";
import type { Tone } from "../../components/ui";

/** The color of each kind of income: the same in the bar, the legend and the rows (docs/13 §3.4, «el color nombra»). */
export const KIND_TONE: Record<string, Tone> = {
  beneficiary_fees: "green",
  fee_estimate: "green",
  recurring_donor: "sky",
  occasional_donation: "violet",
  project_grant: "cyan",
  other: "teal",
};
export const kindTone = (kind: string): Tone => KIND_TONE[kind] ?? "teal";

/** Literal class names so Tailwind sees them. */
export const TONE_BG: Partial<Record<Tone, string>> = { ink: "bg-ink", sky: "bg-sky", violet: "bg-violet", cyan: "bg-cyan", teal: "bg-teal", green: "bg-green", rose: "bg-rose", amber: "bg-amber" };

/** The color of each kind of expense: the payroll is amber (as everywhere), what goes to people teal, the rest ink. */
export const expenseTone = (kind: string): Tone => (kind === "payroll" ? "amber" : kind === "staff_support" || kind === "external_staff" ? "teal" : "ink");

/** What part of a total a line is (0..1), for the small «39 %» next to it. Display only: the sums are Rust's. */
export const shareOf = (annual: number | null, total: number | null): number | null => (annual === null || !total || total <= 0 ? null : annual / total);
export const percent = (share: number): string => (share > 0 && share < 0.01 ? "<1 %" : `${Math.round(share * 100)} %`);

export const INCOME_KINDS = ["recurring_donor", "occasional_donation", "project_grant", "fee_estimate", "other"] as const;

export type BalanceState = "surplus" | "deficit" | "even" | "unknown";

/** What the balance says, from the figures Rust computed. Nothing is added up here (ADR-026). */
export function balanceState(f: Finances): BalanceState {
  const b = f.balance_annual_mxn;
  return b === null ? "unknown" : b > 0 ? "surplus" : b < 0 ? "deficit" : "even";
}

/** Which side of the balance is missing while it is `unknown`. */
export function balanceMissing(f: Finances): "income" | "expenses" | "both" {
  const noIncome = !f.income_known;
  const noExpenses = f.expenses_annual_mxn === null;
  return noIncome && noExpenses ? "both" : noIncome ? "income" : "expenses";
}

/** The share of the income each kind brings, for the composition bar: only what counts. */
export function incomeShares(f: Finances): { kind: string; annual_mxn: number; share: number }[] {
  const total = f.income_annual_mxn;
  return total > 0 ? f.income_by_kind.filter((k) => k.annual_mxn > 0).map((k) => ({ ...k, share: k.annual_mxn / total })) : [];
}

export const isEditable = (l: FinanceLine): l is FinanceLine & { index: number } => l.index !== null;
