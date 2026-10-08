import { describe, expect, it } from "vitest";
import type { ProfileInput, ProfileView } from "../../lib/types";
import { emptyForm, formSchema, fromView, toInput } from "./profileForm";

const input: ProfileInput = {
  institution: {
    name: "Casa Hogar Ficticia",
    kind: "children_home",
    mission: null,
    legal_rfc: null,
    contact_phone: "55 5555 0202",
    contact_email: null,
    legal_rep_name: null,
    state: "cdmx",
    municipality: "Coyoacán",
    founded_year: 2001,
    legal_form: "iap",
    authorized_donee: "yes",
    cluni: "in_progress",
  },
  capacity_total: 40,
  notes: null,
  served_estimate: 30,
  staff_paid_estimate: null,
  staff_volunteer_estimate: 4,
  population: [],
  staff: [],
};

const view = (i: ProfileInput): ProfileView => ({
  institution_id: "inst_1",
  version: 1,
  confirmed_at: null,
  is_draft: true,
  input: i,
  totals: {
    population: 14, staff_paid: 1, staff_volunteer: 0,
    payroll_monthly_mxn: 8500, payroll_annual_mxn: 102000, payroll_benefits_annual_mxn: 5100, payroll_cost_annual_mxn: 107100,
    benefits_assumed: 0, staff_support_annual_mxn: 0, external_staff_annual_mxn: 0, fee_payers: 4, fees_monthly_mxn: 6000, fees_annual_mxn: 72000,
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

  it("never sends money with the profile: it lives in the finance module", () => {
    expect(toInput(fromView(view(input)))).not.toHaveProperty("expenses");
  });

  it("accepts only whole numbers in number fields", () => {
    const ok = formSchema.safeParse(fromView(view(input)));
    expect(ok.success).toBe(true);
    const f = fromView(view(input));
    f.capacity_total = "abc";
    f.founded_year = "-3";
    const bad = formSchema.safeParse(f);
    expect(bad.success).toBe(false);
    expect(bad.error!.issues.map((i) => i.message)).toEqual(["not_a_number", "not_a_number"]);
  });
});
