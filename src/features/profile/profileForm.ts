import { z } from "zod";
import type {
  Condition,
  InstitutionKind,
  ProfileInput,
  ProfileView,
} from "../../lib/types";

// The form keeps everything as text; conversion to numbers happens here.
// Business rules (totals, limits) live in Rust, not here.

const wholeNumber = z.string().refine((v) => v.trim() === "" || /^\d+$/.test(v.trim()), {
  message: "not_a_number",
});

export const formSchema = z.object({
  name: z.string(),
  kind: z.enum(["elderly_home", "children_home", "other"]),
  mission: z.string(),
  legal_rfc: z.string(),
  contact_phone: z.string(),
  contact_email: z.string(),
  legal_rep_name: z.string(),
  capacity_total: wholeNumber,
  annual_budget_mxn: wholeNumber,
  notes: z.string(),
  facilities: z.array(
    z.object({
      kind: z.string(),
      count: wholeNumber,
      condition: z.string(),
      accessible: z.enum(["", "yes", "no"]),
      notes: z.string(),
    }),
  ),
  income: z.array(z.object({ label: z.string(), annual_amount_mxn: wholeNumber })),
});

export type FormValues = z.infer<typeof formSchema>;

export const emptyForm = (): FormValues => ({
  name: "",
  kind: "elderly_home",
  mission: "",
  legal_rfc: "",
  contact_phone: "",
  contact_email: "",
  legal_rep_name: "",
  capacity_total: "",
  annual_budget_mxn: "",
  notes: "",
  facilities: [],
  income: [],
});

const str = (v: string | null | undefined) => v ?? "";
const numStr = (v: number | null | undefined) => (v === null || v === undefined ? "" : String(v));
const textOrNull = (v: string) => (v.trim() === "" ? null : v.trim());
const numOrNull = (v: string) => (v.trim() === "" ? null : Number(v.trim()));
const numOrZero = (v: string) => (v.trim() === "" ? 0 : Number(v.trim()));

export function fromView(view: ProfileView | null): FormValues {
  if (!view) return emptyForm();
  const p = view.input;
  return {
    name: p.institution.name,
    kind: p.institution.kind,
    mission: str(p.institution.mission),
    legal_rfc: str(p.institution.legal_rfc),
    contact_phone: str(p.institution.contact_phone),
    contact_email: str(p.institution.contact_email),
    legal_rep_name: str(p.institution.legal_rep_name),
    capacity_total: numStr(p.capacity_total),
    annual_budget_mxn: numStr(p.annual_budget_mxn),
    notes: str(p.notes),
    facilities: p.facilities.map((f) => ({
      kind: f.kind,
      count: String(f.count),
      condition: f.condition ?? "",
      accessible: f.accessible === null ? "" : f.accessible ? "yes" : "no",
      notes: str(f.notes),
    })),
    income: p.income.map((i) => ({
      label: i.label,
      annual_amount_mxn: numStr(i.annual_amount_mxn),
    })),
  };
}

export function toInput(v: FormValues): ProfileInput {
  return {
    institution: {
      name: v.name.trim(),
      kind: v.kind as InstitutionKind,
      mission: textOrNull(v.mission),
      legal_rfc: textOrNull(v.legal_rfc),
      contact_phone: textOrNull(v.contact_phone),
      contact_email: textOrNull(v.contact_email),
      legal_rep_name: textOrNull(v.legal_rep_name),
    },
    capacity_total: numOrNull(v.capacity_total),
    annual_budget_mxn: numOrNull(v.annual_budget_mxn),
    notes: textOrNull(v.notes),
    // staff and people served come from the roster (ADR-020): the profile adds them up by itself
    population: [],
    staff: [],
    facilities: v.facilities.map((f) => ({
      kind: f.kind,
      count: numOrZero(f.count),
      condition: f.condition === "" ? null : (f.condition as Condition),
      accessible: f.accessible === "" ? null : f.accessible === "yes",
      notes: textOrNull(f.notes),
    })),
    income: v.income.map((i) => ({
      label: i.label,
      annual_amount_mxn: numOrNull(i.annual_amount_mxn),
    })),
  };
}
