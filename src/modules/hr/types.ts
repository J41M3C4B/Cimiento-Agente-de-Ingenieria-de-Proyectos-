// The staff module (ADR-027): its own types, apart from the rest of the app, so it can leave as a module of its own.
import type { ProfileTotals, ProfileView, QuarantineReport } from "../../lib/types";

export type Relation = "employee" | "fees" | "religious" | "volunteer" | "trainee" | "external";
export type PayKind = "salary" | "fees" | "assimilated" | "support" | "invoice" | "none";

export interface ModalityRules {
  relation: Relation;
  lft_benefits: boolean;
  imss: boolean;
  pay: PayKind;
  needs_end_date: boolean;
  tax_data: boolean;
}
export interface ModalityInfo {
  code: string;
  /** Only for the institution's own ones; the built-in ones are named by `es.hr.modalities`. */
  title: string | null;
  builtin: boolean;
  behaves_as: string;
  rules: ModalityRules;
}

export interface Address {
  street: string | null;
  number: string | null;
  neighborhood: string | null;
  municipality: string | null;
  state: string | null;
  zip: string | null;
}
export interface EmergencyContact {
  full_name: string;
  relationship: string | null;
  phone: string | null;
  phone_alt: string | null;
}

/**
 * One record. The four identifiers come back as `null` (covered): sending `null` leaves them as they are, `""`
 * clears them and a value replaces them.
 */
export interface PersonData {
  first_names: string;
  last_name_1: string | null;
  last_name_2: string | null;
  birth_date: string | null;
  sex: string | null;
  curp: string | null;
  marital_status: string | null;
  nationality: string | null;
  education: string | null;
  professional_license: string | null;
  address: Address;
  phone: string | null;
  email: string | null;
  position_id: string | null;
  modality: string;
  start_date: string | null;
  end_date: string | null;
  schedule: string | null;
  shift: string | null;
  work_days: string[];
  weekly_hours: number | null;
  status: string;
  left_date: string | null;
  left_reason: string | null;
  emergency_contacts: EmergencyContact[];
  pay_amount_mxn: number | null;
  pay_period: string | null;
  pay_method: string | null;
  bank: string | null;
  clabe: string | null;
  rfc: string | null;
  nss: string | null;
  tax_regime: string | null;
  tax_zip: string | null;
  infonavit_credit: boolean | null;
  extra: Record<string, string>;
}

export type SecretField = "curp" | "rfc" | "nss" | "clabe";

export interface HrIssue {
  code: string;
  field: string;
  blocking: boolean;
}
export interface StepProgress {
  filled: number;
  total: number;
}
export interface Progress {
  personal: StepProgress;
  job: StepProgress;
  emergency: StepProgress;
  pay: StepProgress;
  percent: number;
}
export interface PersonView {
  id: string;
  data: PersonData;
  secrets: Record<SecretField, boolean>;
  masked: Record<SecretField, string | null>;
  start_date_approx: boolean;
  progress: Progress;
  issues: HrIssue[];
}
export interface PersonRow {
  id: string;
  full_name: string;
  position_id: string | null;
  modality: string;
  relation: Relation | null;
  status: string;
  start_date: string | null;
  phone: string | null;
  progress: number;
  heads_up: number;
}

export interface PositionInput {
  title: string;
  area: string | null;
  duties: string | null;
  default_modality: string | null;
  default_schedule: string | null;
  reference_pay_mxn: number | null;
  authorized_seats: number | null;
  reports_to: string | null;
}
export interface PositionRow extends PositionInput {
  id: string;
  active: boolean;
  /** People in it now. */
  people: number;
}
export interface CustomField {
  key: string;
  title: string;
  kind: "text" | "select" | "number";
  options: string[];
  position: number;
}

export interface StaffOverview {
  people: PersonRow[];
  positions: PositionRow[];
  modalities: ModalityInfo[];
  custom_fields: CustomField[];
  totals: ProfileTotals;
}
export interface StaffChange {
  overview: StaffOverview;
  profile: ProfileView | null;
}
export type PersonOutcome = { status: "saved"; person: PersonView; change: StaffChange } | { status: "invalid"; issues: HrIssue[] };
export type PositionOutcome =
  | { status: "saved"; change: StaffChange }
  | { status: "invalid"; issues: HrIssue[] }
  | { status: "quarantine"; report: QuarantineReport };
