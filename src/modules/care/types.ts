// The module of the people served (ADR-029): its own types, apart from the rest of the app.
import type { ProfileTotals, ProfileView, QuarantineReport } from "../../lib/types";

export type CareFlavor = "elderly_home" | "children_home" | "other";

export interface ResponsibleContact {
  full_name: string;
  relationship: string | null;
  phone: string | null;
  phone_alt: string | null;
  legal_guardian: boolean;
}

/** One record. `curp` comes back as `null` (covered): `null` keeps it, `""` clears it, a value replaces it. */
export interface BeneficiaryData {
  first_names: string;
  last_name_1: string | null;
  last_name_2: string | null;
  birth_date: string | null;
  birth_date_approx: boolean;
  /** When the birth date is not known: Rust turns it into an approximate date. */
  approx_age: number | null;
  sex: string | null;
  curp: string | null;
  origin_municipality: string | null;
  origin_state: string | null;
  indigenous_language: string | null;
  education: string | null;
  literate: boolean | null;
  attends_school: boolean | null;
  school_grade: string | null;
  school_lag: boolean | null;
  group_id: string | null;
  entry_date: string | null;
  entry_date_approx: boolean;
  stay_mode: string | null;
  referred_by: string | null;
  admission_reasons: string[];
  status: string;
  status_date: string | null;
  discharge_reason: string | null;
  dependency: string | null;
  mobility: string | null;
  disabilities: string[];
  chronic_conditions: string[];
  continence: string | null;
  orientation: string | null;
  psych_care: boolean | null;
  vaccines_up_to_date: boolean | null;
  contacts: ResponsibleContact[];
  visits: string | null;
  legal_status: string | null;
  monthly_fee_mxn: number | null;
  fee_payer: string | null;
  programs: string[];
  consent_date: string | null;
  consent_signer: string | null;
  extra: Record<string, string>;
}

export interface CareIssue {
  code: string;
  field: string;
  blocking: boolean;
}
export interface StepProgress {
  filled: number;
  total: number;
}
export interface CareProgress {
  identification: StepProgress;
  stay: StepProgress;
  care: StepProgress;
  family: StepProgress;
  contribution: StepProgress;
  percent: number;
}
export interface BeneficiaryView {
  id: string;
  data: BeneficiaryData;
  curp_stored: boolean;
  curp_masked: string | null;
  group: string;
  age: number | null;
  progress: CareProgress;
  issues: CareIssue[];
}
export interface BeneficiaryRow {
  id: string;
  full_name: string;
  group: string;
  age: number | null;
  sex: string | null;
  status: string;
  dependency: string | null;
  entry_date: string | null;
  progress: number;
  heads_up: number;
}
export interface CareGroup {
  id: string;
  title: string;
  active: boolean;
}
export interface CareField {
  key: string;
  title: string;
  kind: "text" | "select" | "number";
  options: string[];
  position: number;
}
export interface WaitlistInput {
  requested_on: string;
  name: string | null;
  phone: string | null;
  sex: string | null;
  approx_age: number | null;
  dependency: string | null;
  reason: string | null;
  status: string;
}
export interface WaitlistRow extends WaitlistInput {
  id: string;
  person_id: string | null;
}

export interface Count {
  code: string;
  count: number;
}
export interface Indicators {
  served: number;
  hospitalized: number;
  by_group: Count[];
  by_sex: Count[];
  pyramid: [string, string, number][];
  without_age: number;
  average_age: number | null;
  dependency: Count[];
  mobility: Count[];
  disabilities: Count[];
  chronic: Count[];
  stay_modes: Count[];
  referred_by: Count[];
  admission_reasons: Count[];
  average_years: number | null;
  admitted_this_year: number;
  discharged_this_year: number;
  deceased_this_year: number;
  discharge_reasons_this_year: Count[];
  few_visits: number;
  without_contact: number;
  with_program: number;
  elderly_without_program: number;
  programs: Count[];
  exempt: number;
  paying: number;
  average_fee: number | null;
  fees_monthly: number;
  without_consent: number;
  indigenous_language: number;
  attending_school: number;
  school_lag: number;
  legal: Count[];
  incomplete: number;
}
export interface Insight {
  code: string;
  values: Record<string, number>;
  items: string[];
  for_ai: boolean;
}
export interface Board {
  indicators: Indicators;
  waiting: number;
  capacity: number | null;
  free_seats: number | null;
  occupancy_percent: number | null;
  cost_per_person_monthly: number | null;
  insights: Insight[];
}
export interface CareOverview {
  people: BeneficiaryRow[];
  groups: CareGroup[];
  custom_fields: CareField[];
  waitlist: WaitlistRow[];
  totals: ProfileTotals;
  board: Board;
  flavor: CareFlavor;
}
export interface CareChange {
  overview: CareOverview;
  profile: ProfileView | null;
}
export type BeneficiaryOutcome = { status: "saved"; person: BeneficiaryView; change: CareChange } | { status: "invalid"; issues: CareIssue[] };
export type WaitlistOutcome = { status: "saved"; waitlist: WaitlistRow[]; change: CareChange } | { status: "invalid"; issues: CareIssue[] };
export type GroupOutcome = { status: "saved"; groups: CareGroup[] } | { status: "quarantine"; report: QuarantineReport };
