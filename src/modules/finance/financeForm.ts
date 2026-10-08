import { z } from "zod";
import type { FinanceInput, IncomeKind, Period } from "../../lib/types";
import { parsePesos } from "../../lib/format";

// The form keeps everything as text; conversion to numbers happens here. The sums and limits live in Rust.

const pesos = z.string().refine((v) => v.trim() === "" || parsePesos(v) !== null, { message: "not_a_number" });

export const financeSchema = z.object({
  annual_budget_mxn: pesos,
  income: z.array(z.object({ label: z.string(), kind: z.string(), amount_mxn: pesos, period: z.enum(["monthly", "annual"]) })),
  expenses: z.array(z.object({ label: z.string(), amount_mxn: pesos, period: z.enum(["monthly", "annual"]) })),
});

export type FinanceValues = z.infer<typeof financeSchema>;

const numStr = (v: number | null | undefined) => (v === null || v === undefined ? "" : String(v));
const pesosOrNull = (v: string) => (v.trim() === "" ? null : parsePesos(v));

/** The money as the form keeps it. */
export function fromFinance(m: FinanceInput | null): FinanceValues {
  return {
    annual_budget_mxn: numStr(m?.annual_budget_mxn),
    income: (m?.income ?? []).map((i) => ({ label: i.label, kind: i.kind, amount_mxn: numStr(i.amount_mxn), period: i.period })),
    expenses: (m?.expenses ?? []).map((e) => ({ label: e.label, amount_mxn: numStr(e.amount_mxn), period: e.period })),
  };
}

/** The money of the form, as the finance module keeps it. */
export function toFinance(v: FinanceValues): FinanceInput {
  return {
    annual_budget_mxn: pesosOrNull(v.annual_budget_mxn),
    income: v.income.map((i) => ({ label: i.label, kind: i.kind as IncomeKind, amount_mxn: pesosOrNull(i.amount_mxn), period: i.period as Period })),
    expenses: v.expenses.map((e) => ({ label: e.label, amount_mxn: pesosOrNull(e.amount_mxn), period: e.period as Period })),
  };
}
