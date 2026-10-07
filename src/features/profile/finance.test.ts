import { describe, expect, it } from "vitest";
import type { Finances } from "../../lib/types";
import { balanceMissing, balanceState, incomeShares, kindTone } from "./finance";

const base: Finances = {
  income: [],
  income_by_kind: [],
  income_fixed_annual_mxn: 0,
  income_variable_annual_mxn: 0,
  income_annual_mxn: 0,
  income_known: false,
  expenses: [],
  expenses_basis: "unknown",
  expenses_annual_mxn: null,
  balance_annual_mxn: null,
};

describe("finanzas de Mi institución (ADR-026: la pantalla solo muestra lo que Rust calculó)", () => {
  it("el balance es superávit, déficit, parejo o falta un dato", () => {
    expect(balanceState({ ...base, balance_annual_mxn: 100 })).toBe("surplus");
    expect(balanceState({ ...base, balance_annual_mxn: -1 })).toBe("deficit");
    expect(balanceState({ ...base, balance_annual_mxn: 0 })).toBe("even");
    expect(balanceState(base)).toBe("unknown");
  });

  it("dice qué lado falta mientras no hay balance", () => {
    expect(balanceMissing(base)).toBe("both");
    expect(balanceMissing({ ...base, income_known: true })).toBe("expenses");
    expect(balanceMissing({ ...base, expenses_annual_mxn: 10 })).toBe("income");
  });

  it("la barra de ingresos reparte solo lo que suma", () => {
    const f: Finances = {
      ...base,
      income_annual_mxn: 400,
      income_by_kind: [
        { kind: "recurring_donor", annual_mxn: 300 },
        { kind: "fee_estimate", annual_mxn: 0 },
        { kind: "other", annual_mxn: 100 },
      ],
    };
    expect(incomeShares(f).map((s) => [s.kind, s.share])).toEqual([["recurring_donor", 0.75], ["other", 0.25]]);
    expect(incomeShares(base)).toEqual([]);
  });

  it("cada tipo de ingreso tiene siempre el mismo color", () => {
    expect(kindTone("beneficiary_fees")).toBe(kindTone("fee_estimate"));
    expect(new Set(["recurring_donor", "occasional_donation", "project_grant", "other"].map(kindTone)).size).toBe(4);
  });
});
