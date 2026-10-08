import { z } from "zod";
import type { InstitutionKind, ProfileInput, ProfileView } from "../../lib/types";

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
  state: z.string(),
  municipality: z.string(),
  founded_year: wholeNumber,
  legal_form: z.string(),
  authorized_donee: z.string(),
  cluni: z.string(),
  served_estimate: wholeNumber,
  staff_paid_estimate: wholeNumber,
  staff_volunteer_estimate: wholeNumber,
  capacity_total: wholeNumber,
  notes: z.string(),
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
  state: "",
  municipality: "",
  founded_year: "",
  legal_form: "",
  authorized_donee: "",
  cluni: "",
  served_estimate: "",
  staff_paid_estimate: "",
  staff_volunteer_estimate: "",
  capacity_total: "",
  notes: "",
});

const str = (v: string | null | undefined) => v ?? "";
const numStr = (v: number | null | undefined) => (v === null || v === undefined ? "" : String(v));
const textOrNull = (v: string) => (v.trim() === "" ? null : v.trim());
const numOrNull = (v: string) => (v.trim() === "" ? null : Number(v.trim()));

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
    state: str(p.institution.state),
    municipality: str(p.institution.municipality),
    founded_year: numStr(p.institution.founded_year),
    legal_form: str(p.institution.legal_form),
    authorized_donee: str(p.institution.authorized_donee),
    cluni: str(p.institution.cluni),
    served_estimate: numStr(p.served_estimate),
    staff_paid_estimate: numStr(p.staff_paid_estimate),
    staff_volunteer_estimate: numStr(p.staff_volunteer_estimate),
    capacity_total: numStr(p.capacity_total),
    notes: str(p.notes),
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
      state: textOrNull(v.state),
      municipality: textOrNull(v.municipality),
      founded_year: numOrNull(v.founded_year),
      legal_form: textOrNull(v.legal_form),
      authorized_donee: textOrNull(v.authorized_donee),
      cluni: textOrNull(v.cluni),
    },
    capacity_total: numOrNull(v.capacity_total),
    served_estimate: numOrNull(v.served_estimate),
    staff_paid_estimate: numOrNull(v.staff_paid_estimate),
    staff_volunteer_estimate: numOrNull(v.staff_volunteer_estimate),
    notes: textOrNull(v.notes),
    // staff and people served come from the roster (ADR-020): the profile adds them up by itself
    population: [],
    staff: [],
  };
}

