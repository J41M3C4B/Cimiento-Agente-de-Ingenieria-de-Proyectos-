import { describe, expect, it } from "vitest";
import type { FinanceInput } from "../../lib/types";
import { parsePesos } from "../../lib/format";
import { financeSchema, fromFinance, toFinance } from "./financeForm";

const money: FinanceInput = {
  annual_budget_mxn: null,
  income: [{ label: "Donativos", kind: "occasional_donation", amount_mxn: 1000, period: "monthly" }],
  expenses: [{ label: "Alimentos", amount_mxn: 8000, period: "monthly" }],
};

describe("money form conversion", () => {
  it("round-trips what the app sends", () => {
    expect(toFinance(fromFinance(money))).toEqual(money);
  });

  it("starts empty when there is no money yet", () => {
    expect(toFinance(fromFinance(null))).toEqual({ annual_budget_mxn: null, income: [], expenses: [] });
  });

  it("reads amounts as people write them, like Rust does", () => {
    for (const [text, n] of [["1800000", 1800000], ["1,800,000", 1800000], ["$1 800 000", 1800000], ["$ 950", 950], ["1.800.000", 1800000]] as const) {
      expect(parsePesos(text)).toBe(n);
    }
    for (const text of ["abc", "9 mil", "1,80,000", "1800,000", "-5", "12.50", "$"]) expect(parsePesos(text)).toBeNull();
    const f = fromFinance(money);
    f.annual_budget_mxn = "$1,800,000";
    f.income[0]!.amount_mxn = "12,500";
    expect(financeSchema.safeParse(f).success).toBe(true);
    expect(toFinance(f).annual_budget_mxn).toBe(1800000);
    expect(toFinance(f).income[0]!.amount_mxn).toBe(12500);
  });

  it("rejects what is not an amount", () => {
    const f = fromFinance(money);
    f.annual_budget_mxn = "-3";
    const bad = financeSchema.safeParse(f);
    expect(bad.success).toBe(false);
    expect(bad.error!.issues.map((i) => i.message)).toEqual(["not_a_number"]);
  });
});
