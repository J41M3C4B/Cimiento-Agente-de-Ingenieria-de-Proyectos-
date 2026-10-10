// Types that mirror what the Rust side sends (src-tauri/src/domain, service, scanner/guard).

export type InstitutionKind = "elderly_home" | "children_home" | "other";
export type DependencyLevel = "low" | "medium" | "high" | "total";

export interface InstitutionInput {
  name: string;
  kind: InstitutionKind;
  mission: string | null;
  legal_rfc: string | null;
  contact_phone: string | null;
  contact_email: string | null;
  legal_rep_name: string | null;
  /** Where it is and what it is, legally (ADR-031). */
  state: string | null;
  municipality: string | null;
  founded_year: number | null;
  legal_form: string | null;
  authorized_donee: string | null;
  cluni: string | null;
  /** Whom and how it serves (ADR-033); left out when a window does not edit it, so what is saved stays. */
  attention?: Attention | null;
}
/** The attention profile: codes of `core::institution::catalog`. */
export interface Attention {
  populations: string[];
  sex_served: string | null;
  modalities: string[];
  care_areas: string[];
}

// Forms described as data (ADR-033, common/forms.rs): the screen draws them, Rust validates and saves them.
export type FieldKind = "text" | "long_text" | "number" | "money" | "year" | "date" | "select" | "multi_select" | "yes_no" | "email" | "phone";
export type Condition = { when: "always" } | { when: "filled"; field: string } | { when: "any_of"; field: string; values: string[] };
export interface FieldSpec {
  id: string;
  kind: FieldKind;
  options: string[];
  required: boolean;
  applies_when: Condition;
  sensitivity: "public" | "internal" | "institutional_private" | "personal";
  ai: "as_is" | "aggregate_only" | "never";
  used_by: string[];
  min: number | null;
  max: number | null;
}
export interface SectionSpec {
  id: string;
  columns: number;
  fields: FieldSpec[];
}
export interface FormSpec {
  id: string;
  sections: SectionSpec[];
}
export type FormValue = string | number | boolean | string[];
export type FormValues = Record<string, FormValue>;
// The institution at a glance (ADR-033, core/overview.rs): composed in Rust, the screen only draws it.
export type Place = "institution" | "contact" | "legal" | "capacity" | "finance" | "facilities" | "staff" | "people";
export interface Figure {
  value: number;
  /** the quick figure of the first start, while the module has no records */
  approx: boolean;
}
export interface InstitutionOverview {
  people: Figure;
  capacity: number | null;
  occupied_percent: number | null;
  vacant: number | null;
  staff: Figure;
  spaces: number;
  balance_annual_mxn: number | null;
  completion: { gaps: { code: string; place: Place }[]; percent: number };
}
export interface FormView {
  spec: FormSpec;
  values: FormValues;
  missing: string[];
}
export interface PopulationGroupInput {
  label: string;
  age_min: number | null;
  age_max: number | null;
  count: number;
  dependency_level: DependencyLevel | null;
  notes: string | null;
  paying_count: number | null;
  monthly_fee_mxn: number | null;
}
export type ContractKind = "permanent" | "temporary" | "fees";
export interface StaffGroupInput {
  role: string;
  count: number;
  shift: string | null;
  paid: boolean;
  monthly_salary_mxn: number | null;
  contract: ContractKind | null;
  start_year: number | null;
  notes: string | null;
}
/** Whether an amount is written per month or per year; Rust turns it into a year (ADR-026). */
export type Period = "monthly" | "annual";
export type IncomeKind = "fee_estimate" | "recurring_donor" | "occasional_donation" | "project_grant" | "other";
export interface IncomeSourceInput {
  label: string;
  kind: IncomeKind;
  amount_mxn: number | null;
  period: Period;
}
export interface ExpenseItemInput {
  label: string;
  amount_mxn: number | null;
  period: Period;
}
export interface ProfileInput {
  institution: InstitutionInput;
  capacity_total: number | null;
  notes: string | null;
  /** Quick figures while the records are not in the modules yet (ADR-031). */
  served_estimate: number | null;
  staff_paid_estimate: number | null;
  staff_volunteer_estimate: number | null;
  population: PopulationGroupInput[];
  staff: StaffGroupInput[];
}

/** Everything the person writes about the money; it lives in the finance module (ADR-032). */
export interface FinanceInput {
  /** «Gasto anual aproximado»: the quick way to start; once the list has a line, the list is the total. */
  annual_budget_mxn: number | null;
  income: IncomeSourceInput[];
  expenses: ExpenseItemInput[];
}

export interface ProfileIssue {
  code: string;
  field: string;
  blocking: boolean;
}
export interface ProfileTotals {
  population: number;
  staff_paid: number;
  staff_volunteer: number;
  payroll_monthly_mxn: number;
  payroll_annual_mxn: number;
  payroll_benefits_annual_mxn: number;
  payroll_cost_annual_mxn: number;
  benefits_assumed: number;
  /** Contributions to the congregation and social-service grants, in a year (not payroll; ADR-027). */
  staff_support_annual_mxn: number;
  /** What outside companies bill for their staff, in a year (not payroll; ADR-027). */
  external_staff_annual_mxn: number;
  fee_payers: number;
  fees_monthly_mxn: number;
  fees_annual_mxn: number;
}

export interface ProfileView {
  institution_id: string;
  version: number;
  confirmed_at: string | null;
  is_draft: boolean;
  input: ProfileInput;
  totals: ProfileTotals;
  issues: ProfileIssue[];
}

/** The money as the finance module shows it: what was written, the sums made by Rust and the heads-ups. */
export interface FinanceView {
  input: FinanceInput;
  finances: Finances;
  issues: ProfileIssue[];
}

/** One line of money, as Rust computed it. `index` points into `FinanceInput.income` / `.expenses`; `null` is a line
 * the app computes from the roster (`beneficiary_fees`, `payroll`) and nobody edits. */
export interface FinanceLine {
  label: string;
  kind: IncomeKind | "beneficiary_fees" | "expense" | "payroll" | "staff_support" | "external_staff";
  index: number | null;
  annual_mxn: number | null;
  counted: boolean;
}
export type ExpenseBasis = "list" | "estimate" | "unknown";
export interface Finances {
  income: FinanceLine[];
  income_by_kind: { kind: FinanceLine["kind"]; annual_mxn: number }[];
  income_fixed_annual_mxn: number;
  income_variable_annual_mxn: number;
  income_annual_mxn: number;
  income_known: boolean;
  expenses: FinanceLine[];
  expenses_basis: ExpenseBasis;
  expenses_annual_mxn: number | null;
  balance_annual_mxn: number | null;
}

export type Decision = "redact" | "not_personal";

export interface FieldReport {
  path: string;
  counts: Record<string, number>;
  blocking: boolean;
  redacted_preview: string;
}
export interface QuarantineReport {
  fields: FieldReport[];
  counts: Record<string, number>;
  has_blocking: boolean;
}

export type SaveProfileOutcome =
  | { status: "saved"; profile: ProfileView }
  | { status: "quarantine"; report: QuarantineReport }
  | { status: "invalid"; issues: ProfileIssue[] };

export type FinanceOutcome =
  | { status: "saved"; finance: FinanceView }
  | { status: "quarantine"; report: QuarantineReport }
  | { status: "invalid"; issues: ProfileIssue[] };

export interface DocumentSummary {
  id: string;
  kind: string;
  display_name: string;
  data_level: string;
  redactions_count: number;
  chunks: number;
  created_at: string;
}
export type AddDocumentOutcome =
  | { status: "saved"; document: DocumentSummary }
  | { status: "quarantine"; report: QuarantineReport }
  | { status: "rejected_roster" };

export interface DeleteSummary {
  chunks: number;
  derived_rows: number;
}

// Calls (convocatorias): the files, the reading in the background, and what was understood of them.
export type ReadingStatus = "waiting" | "reading" | "ready" | "partial" | "failed";
/** What a file is inside the package of a call, as the person marked it. A package has exactly one `main`. */
export type FileRole = "main" | "annex" | "guide" | "form" | "notice" | "other";
export interface ReadingFile {
  document_id: string;
  name: string;
  pages: number;
  role: FileRole;
}
export interface ReadingRow {
  id: string;
  name: string;
  /** Who gives the call and in which year, as the person wrote them. */
  funder: string | null;
  year: number | null;
  status: ReadingStatus;
  /** Why it is waiting, partial or failed (a code the screen words). */
  note: string | null;
  created_at: string;
  finished_at: string | null;
  /** When the person confirmed that this is the right call (cleared when it is read again). */
  confirmed_at: string | null;
  files: ReadingFile[];
}
export type UnreadableReason =
  | "unsupported_type"
  | "too_large"
  | "damaged"
  | "scanned"
  | "empty"
  | "roster"
  | "no_files"
  | "too_many_files"
  | "no_main_file";
export type NewProjectOutcome =
  | { status: "created"; project: ProjectRow; reading: ReadingRow }
  | { status: "quarantine"; report: QuarantineReport }
  | { status: "unreadable"; file: string; reason: UnreadableReason };
export interface SummaryItem {
  text: string;
  applies_to: string | null;
  requirement: string | null;
  page: number | null;
  file: string | null;
}
export interface SummaryGroup {
  key: string;
  items: SummaryItem[];
}
export interface SummaryDate {
  label: string;
  when: string;
  kind: string;
  page: number | null;
  file: string | null;
}
export interface SummaryAmount {
  kind: "max_amount" | "min_amount" | "cofunding" | "admin_cap" | "modality";
  label: string | null;
  value: string;
  page: number | null;
  file: string | null;
}
export interface SummaryConflict {
  field: string;
  note: string;
  versions: SummaryItem[];
}
export interface SummaryKind {
  /** What the document calls itself, as written. */
  words: string;
  /** The class the code reads from those words: only describes the document. */
  class: string;
}
export interface CallSummary {
  document_kind: SummaryKind | null;
  title: string | null;
  funder: string | null;
  edition: string | null;
  objective: string | null;
  dates: SummaryDate[];
  amounts: SummaryAmount[];
  groups: SummaryGroup[];
  conflicts: SummaryConflict[];
  doubts: string[];
  /** Fields the documents do not say, as `section.field`. */
  missing: string[];
}
export interface ReadingQuality {
  pages: number;
  pages_with_quotes: number;
  quotes_verified_percent: number;
  blocks_read: number;
  blocks_total: number;
}
/** Something the person wrote about the call that the documents do not seem to say (only a heads-up). */
export interface ReadingDifference {
  field: "funder" | "year";
  said: string;
  read: string;
}
/** What the person reads first about a call (ADR-024): short, in plain words, built by the program. */
export interface CardFact {
  kind: "max_amount" | "min_amount" | "cofunding" | "admin_cap" | "duration" | "closing" | "registration" | "modalities";
  value: string;
  page: number | null;
  file: string | null;
}
export interface CardPoint {
  text: string;
  applies_to: string | null;
  page: number | null;
  file: string | null;
}
export interface CardBlock {
  key: "who_can" | "supported" | "fundable" | "not_fundable";
  points: CardPoint[];
  /** How many more points the detail has. */
  more: number;
}
export interface CardAlert {
  kind: "not_a_call" | "conflicts" | "missing" | "doubts";
  count: number;
  /** For `missing`: the first data not found, as `section.field`. */
  fields: string[];
}
export interface CallCard {
  title: string | null;
  funder: string | null;
  edition: string | null;
  /** The «en pocas palabras» written with the automatic help, once checked. */
  brief: string | null;
  /** What stands in for it: the first sentence of what the document says the call is for. */
  lead: string | null;
  facts: CardFact[];
  blocks: CardBlock[];
  alerts: CardAlert[];
}

export interface ReadingDetail {
  reading: ReadingRow;
  card: CallCard | null;
  /** The call was read and nobody has tried yet to write its «en pocas palabras». */
  brief_pending: boolean;
  /** The whole understanding, group by group: for looking something up, not for reading from start to end. */
  summary: CallSummary | null;
  quality: ReadingQuality | null;
  differences: ReadingDifference[];
}

export interface AppError {
  code: string;
  message: string;
}

// ---------------------------------------------------------------- AI, projects, diagnosis

export type StageName =
  | "PROFILE"
  | "DIAGNOSIS"
  | "PRIORITIZATION"
  | "CALL_SELECTION"
  | "DRAFTING"
  | "REVIEW"
  | "READY";

export const STAGES: StageName[] = [
  "PROFILE",
  "CALL_SELECTION",
  "DIAGNOSIS",
  "PRIORITIZATION",
  "DRAFTING",
  "REVIEW",
  "READY",
];

export type AiProvider = "gemini" | "anthropic";

export interface AiStatusView {
  provider: AiProvider;
  /** A key is saved for the active provider. */
  has_key: boolean;
  keys: { gemini: boolean; anthropic: boolean };
  model_light: string;
  model_strong: string;
  spent_mxn: number;
  cap_mxn: number;
  percent: number;
  near_cap: boolean;
  paused: boolean;
}

/** What the AI is doing for a project: a message of the conversation, the summary, or the suggested objectives. */
export type JobKind = "turn" | "summary" | "needs" | "brief";

export interface ProjectJob {
  running: JobKind | null;
  /** How the last job ended; the program hands it over only once. */
  finished: { kind: JobKind; ai: AiStatus } | null;
}

export interface AiModelsView {
  provider: AiProvider;
  light: string;
  strong: string;
  /** Thinking depth of the strong tier; empty = the service's default. */
  effort: string;
  known: string[];
  strong_chain: string[];
}

export type AiStatus =
  | "used"
  | "not_configured"
  | "offline"
  | "busy"
  | "budget_exhausted"
  | "quota_reached"
  | "key_rejected"
  | "unavailable"
  | "skipped";

export interface ModelCheck {
  model: string;
  exists: boolean;
  can_generate: boolean;
}

export interface AiCheckView {
  ok: boolean;
  models: ModelCheck[];
  /** Why it could not be checked (same keys as `AiStatus`). */
  problem: AiStatus | null;
}

export interface RateLimit {
  per_minute: number;
  tokens_per_minute: number;
  per_day: number;
}

export interface ModelUsage {
  provider: string;
  model: string;
  /** Which job the active provider gives this model, if any. */
  tier: "light" | "strong" | null;
  /** It is a backup of that job, not the main model. */
  is_fallback: boolean;
  calls_last_minute: number;
  calls_last_day: number;
  tokens_last_minute: number;
  limit: RateLimit | null;
  calls_total: number;
  failed_total: number;
  input_tokens: number;
  output_tokens: number;
  thought_tokens: number;
  cached_tokens: number;
  avg_latency_ms: number | null;
  p95_latency_ms: number | null;
  cost_mxn: number;
}

export interface TaskUsage {
  task: string;
  calls: number;
  avg_latency_ms: number | null;
  avg_input_tokens: number;
  avg_output_tokens: number;
  avg_thought_tokens: number;
  cost_mxn: number;
}

export interface RecentCall {
  at: string;
  task: string;
  model: string;
  latency_ms: number | null;
  input_tokens: number;
  output_tokens: number;
  thought_tokens: number;
  ok: boolean;
  error_kind: string | null;
}

export interface UsageReport {
  provider: AiProvider;
  month_spend_mxn: number;
  cap_mxn: number;
  models: ModelUsage[];
  tasks: TaskUsage[];
  recent: RecentCall[];
}

export interface ProjectRow {
  id: string;
  institution_id: string;
  profile_id: string;
  title: string;
  initial_request: string | null;
  stage: StageName;
  needs_review: boolean;
  created_at: string;
  /** How it began: from a call, or from an everyday need of the institution (switched off for now). */
  kind: "call" | "internal";
  /** The reading of the call this project was born from. */
  call_reading_id: string | null;
  /** The color of its folder; `null` until the person picks one. */
  color: ProjectColor | null;
  /** Who gives the support; `null` if it was not said. */
  donor_kind: DonorKind | null;
}
export type DonorKind = "institutional" | "private" | "individual";
export type ProjectColor = "blue" | "violet" | "teal" | "green" | "amber" | "orange" | "pink" | "red";

export interface SummaryJson {
  problem_statement: string;
  affected: { group: string; count: number | null; description: string };
  current_consequences: string[];
  root_causes: string[];
  reframed_need: string;
  alternatives: { title: string; pros: string[]; cons: string[] }[];
  suggested_indicators: string[];
  open_questions: string[];
}

export interface StoredSummary {
  summary: SummaryJson;
  origin: string;
  confirmed_at: string | null;
}

export type ConversationPhase = "needs_opening" | "awaiting_answer" | "awaiting_ai" | "root_proposed" | "closed";
export type TurnKind = "opening" | "why" | "root_proposal" | "root_reply";
export interface TurnView {
  turn: number;
  role: "assistant" | "person";
  kind: TurnKind;
  /** Which «why» the assistant asks or the person answers. */
  level: number | null;
  text: string;
  /** Quick replies the assistant offered. */
  options: string[];
}
/** The guided conversation of the diagnosis and, once it ends, the summary (ADR-017). */
export interface ConversationView {
  project: ProjectRow;
  call: { name: string; funder: string | null; year: number | null } | null;
  turns: TurnView[];
  phase: ConversationPhase;
  /** The «why» being asked or answered now (0 before the first). */
  why_level: number;
  max_whys: number;
  /** How well the idea fits what the call funds, as judged when the person answered the opening. */
  fit: { fit: "fits" | "partial" | "mismatch"; note: string } | null;
  root: { text: string; confirmed: boolean } | null;
  summary: StoredSummary | null;
  unsupported_figures: string[];
  /** Started with the seven fixed questions of the earlier method: it has no conversation. */
  legacy: boolean;
}

export type CreateProjectOutcome =
  | { status: "created"; project: ProjectRow }
  | { status: "quarantine"; report: QuarantineReport };

export type AnswerOutcome =
  | { status: "saved"; view: ConversationView; ai: AiStatus }
  | { status: "quarantine"; report: QuarantineReport };

export interface SummaryOutcome {
  view: ConversationView;
  ai: AiStatus;
}

export interface SummaryEdit {
  problem_statement: string;
  reframed_need: string;
  affected_description: string;
  current_consequences: string[];
  root_causes: string[];
  suggested_indicators: string[];
  open_questions: string[];
}

export type EditOutcome =
  | { status: "saved"; view: ConversationView }
  | { status: "quarantine"; report: QuarantineReport };

export interface Scores {
  beneficiaries: number;
  severity: number;
  mission: number;
  feasibility: number;
  sustainability: number;
}

export interface NeedRow {
  id: string;
  title: string;
  description: string | null;
  scores: Scores | null;
  total_score: number | null;
  selected: boolean;
  origin: string;
  confirmed: boolean;
}

export interface NeedsView {
  project: ProjectRow;
  needs: NeedRow[];
  ranking: string[];
  beneficiaries_suggestion: number | null;
}

export type AddNeedOutcome =
  | { status: "saved"; view: NeedsView }
  | { status: "quarantine"; report: QuarantineReport };

// ---------------------------------------------------------------- drafting, review and the guide (ADR-018)

export interface Sourced<T> {
  value: T;
  page: number | null;
  file: string | null;
}
export interface CallLine {
  text: string;
  applies_to: string | null;
  page: number | null;
  file: string | null;
}
/** What the call asks of a project, read from its confirmed reading. What it does not say is null or empty. */
export interface CallRequirements {
  max_amount_mxn: Sourced<number> | null;
  min_amount_mxn: Sourced<number> | null;
  foreign_currency: boolean;
  cofunding_percent: Sourced<number> | null;
  admin_cap_percent: Sourced<number> | null;
  max_duration_months: Sourced<number> | null;
  closing_date: Sourced<string> | null;
  required_docs: CallLine[];
  conditional_docs: CallLine[];
  optional_docs: CallLine[];
  formats: CallLine[];
  evaluation_criteria: CallLine[];
  project_requirements: CallLine[];
  fundable: CallLine[];
  not_fundable: CallLine[];
  indicators: CallLine[];
  how_to_deliver: CallLine[];
  contact: CallLine[];
}

export type SectionKind = "data" | "text" | "budget" | "schedule";
export type SectionStatus = "empty" | "draft_ai" | "draft_user" | "confirmed" | "needs_review";
export interface SectionView {
  key: string;
  title: string;
  guidance: string;
  kind: SectionKind;
  required: boolean;
  source: "base" | "call";
  content: string;
  status: SectionStatus;
  /** What the AI says is missing in its text. */
  open_points: string[];
  /** Numbers in the AI's text that nobody gave. */
  unsupported_figures: string[];
  /** What the section asks, in plain words, as the assistant explained it when the drafting began. */
  plain: string | null;
}

export type Funder = "requested" | "institution" | "other";
export interface BudgetItemView {
  id: string;
  category: string;
  description: string;
  quantity: number;
  unit: string | null;
  unit_price_mxn: number;
  vat_included: boolean;
  funded_by: Funder;
  administrative: boolean;
  /** `ai_assumption` while it is the assistant's proposal; `user` once the person touched it. */
  origin: string;
  line: { subtotal: number; vat: number; total: number };
}
export interface BudgetTotals {
  subtotal: number;
  vat: number;
  total: number;
  requested: number;
  institution: number;
  other: number;
  administrative_requested: number;
  counterpart_percent: number;
  administrative_percent: number;
}
export interface ActivityView {
  id: string;
  title: string;
  start_month: number;
  end_month: number;
  origin: string;
}
export interface DraftingView {
  project: ProjectRow;
  asks_for_proposal: boolean;
  asks_confirmed: boolean;
  requirements: CallRequirements;
  sections: SectionView[];
  budget: { items: BudgetItemView[]; totals: BudgetTotals; confirmed: boolean; missing_prices: number };
  schedule: { activities: ActivityView[]; duration_months: number; confirmed: boolean };
  objective: string | null;
  /** The assistant already prepared the draft (explanations, budget lines and schedule). */
  plan_ready: boolean;
}
export type DraftMode = "full" | "guide";
export interface BudgetItemInput {
  id: string | null;
  category: string;
  description: string;
  quantity: number;
  unit: string | null;
  unit_price_mxn: number;
  vat_included: boolean;
  funded_by: Funder;
  administrative: boolean;
}
export type DraftEditOutcome =
  | { status: "saved"; view: DraftingView }
  | { status: "quarantine"; report: QuarantineReport };
export interface DraftOutcome {
  view: DraftingView;
  ai: AiStatus;
}

export type CheckLevel = "error" | "warn" | "info";
export interface ReviewCheck {
  code: string;
  level: CheckLevel;
  args: string[];
  /** What fixes it: a section key, or `budget`, `schedule`, `call`. */
  target: string | null;
  /** The check in plain Spanish. */
  text: string;
}
export interface ReviewView {
  project: ProjectRow;
  report: { checks: ReviewCheck[]; errors: number; warnings: number };
}
export interface Exported {
  file_name: string;
  path: string;
}

// ---------------------------------------------------------------- security: PIN, backup, scan (ADR-019)

export interface BackupFile {
  file_name: string;
  path: string;
}
export interface ScanTable {
  table: string;
  texts: number;
  findings: number;
}
export interface ScanSummary {
  tables: ScanTable[];
  texts: number;
  findings: number;
}
