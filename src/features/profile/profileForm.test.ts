import { describe, expect, it } from "vitest";
import type { ProfileInput, ProfileView } from "../../lib/types";
import { emptyForm, formSchema, fromView, parsePesos, toInput } from "./profileForm";

const input: ProfileInput = {
  institution: {
    name: "Casa Hogar Ficticia",
    kind: "children_home",
    mission: null,
    legal_rfc: null,
    contact_phone: "55 5555 0202",
    contact_email: null,
    legal_rep_name: null,
  },
  capacity_total: 40,
  annual_budget_mxn: null,
  notes: null,
  population: [],
  staff: [],
  facilities: [{ kind: "Baño", count: 4, condition: "poor", accessible: false, notes: "Humedad" }],
  income: [{ label: "Donativos", kind: "occasional_donation", amount_mxn: 1000, period: "monthly" }],
  expenses: [{ label: "Alimentos", amount_mxn: 8000, period: "monthly" }],
};

const view = (i: ProfileInput): ProfileView => ({
  institution_id: "inst_1",
  version: 1,
  confirmed_at: null,
  is_draft: true,
  input: i,
  totals: {
    population: 14, staff_paid: 1, staff_volunteer: 0, income_annual_mxn: 1000,
    payroll_monthly_mxn: 8500, payroll_annual_mxn: 102000, payroll_benefits_annual_mxn: 5100, payroll_cost_annual_mxn: 107100,
    benefits_assumed: 0, fee_payers: 4, fees_monthly_mxn: 6000, fees_annual_mxn: 72000,
  },
  finances: {
    income: [], income_by_kind: [], income_fixed_annual_mxn: 0, income_variable_annual_mxn: 0, income_annual_mxn: 0,
    income_known: false, expenses: [], expenses_basis: "unknown", expenses_annual_mxn: null, balance_annual_mxn: null,
  },
  issues: [],
});

describe("profile form conversion", () => {
  it("round-trips what the app sends", () => {
    expect(toInput(fromView(view(input)))).toEqual(input);
  });

  it("never sends staff or people served: they come from the roster", () => {
    const f = fromView({ ...view(input), input: { ...input, staff: [{ role: "x", count: 3, shift: null, paid: true, monthly_salary_mxn: null, contract: null, start_year: null, notes: null }] } });
    expect(f).not.toHaveProperty("staff");
    expect(toInput(f).staff).toEqual([]);
    expect(toInput(f).population).toEqual([]);
  });

  it("turns empty text into null and empty counts into zero", () => {
    const f = emptyForm();
    f.name = "  Asilo  ";
    const out = toInput(f);
    expect(out.institution.name).toBe("Asilo");
    expect(out.institution.mission).toBeNull();
    expect(out.capacity_total).toBeNull();
  });

  it("keeps the expenses when another card is saved", () => {
    const f = fromView(view(input));
    f.name = "Otro nombre";
    expect(toInput(f).expenses).toEqual(input.expenses);
  });

  it("reads amounts as people write them, like Rust does", () => {
    for (const [text, n] of [["1800000", 1800000], ["1,800,000", 1800000], ["$1 800 000", 1800000], ["$ 950", 950], ["1.800.000", 1800000]] as const) {
      expect(parsePesos(text)).toBe(n);
    }
    for (const text of ["abc", "9 mil", "1,80,000", "1800,000", "-5", "12.50", "$"]) expect(parsePesos(text)).toBeNull();
    const f = fromView(view(input));
    f.annual_budget_mxn = "$1,800,000";
    f.income[0].amount_mxn = "12,500";
    expect(formSchema.safeParse(f).success).toBe(true);
    expect(toInput(f).annual_budget_mxn).toBe(1800000);
    expect(toInput(f).income[0].amount_mxn).toBe(12500);
  });

  it("accepts only whole numbers in number fields", () => {
    const ok = formSchema.safeParse(fromView(view(input)));
    expect(ok.success).toBe(true);
    const f = fromView(view(input));
    f.capacity_total = "abc";
    f.facilities[0].count = "-3";
    const bad = formSchema.safeParse(f);
    expect(bad.success).toBe(false);
    expect(bad.error!.issues.map((i) => i.message)).toEqual(["not_a_number", "not_a_number"]);
  });
});
