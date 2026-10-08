// The first start (ADR-031): the welcome of each person, the setup of the administrator and the data of the institution.
import { invoke } from "@tauri-apps/api/core";
import type { Decision, IncomeSourceInput, InstitutionInput, QuarantineReport } from "../../lib/types";

export interface OnboardingData {
  institution: InstitutionInput;
  capacity_total: number | null;
  served_estimate: number | null;
  staff_paid_estimate: number | null;
  staff_volunteer_estimate: number | null;
  annual_budget_mxn: number | null;
  income: IncomeSourceInput[];
  floors: number | null;
  built_m2: number | null;
  tenure: string | null;
  tenure_until: number | null;
  tenure_documented: boolean | null;
}

export interface StepStatus {
  key: string;
  /** The required data still missing (codes). */
  missing: string[];
  complete: boolean;
}

export interface OnboardingStatus {
  /** The institution finished it once: the app opens. */
  done: boolean;
  steps: StepStatus[];
  /** Every step complete: it can be closed. */
  ready: boolean;
  data: OnboardingData;
  records: { served: number; staff: number; fee_payers: number };
  welcomed: boolean;
  /** The administrator may leave the data to the direction. */
  can_postpone: boolean;
  setup: { ai_ready: boolean; managers: number } | null;
}

export interface OnboardingIssue {
  code: string;
  field: string;
}

export type OnboardingOutcome =
  | { status: "saved"; onboarding: OnboardingStatus }
  | { status: "invalid"; issues: OnboardingIssue[] }
  | { status: "quarantine"; report: QuarantineReport };

export const ONBOARDING_KEY = ["onboarding"] as const;
export const onboardingStatus = () => invoke<OnboardingStatus>("onboarding_status");
export const onboardingSave = (data: OnboardingData, decision?: Decision) => invoke<OnboardingOutcome>("onboarding_save", { data, decision: decision ?? null });
export const onboardingFinish = () => invoke<OnboardingStatus>("onboarding_finish");
export const onboardingWelcomeDone = () => invoke<void>("onboarding_welcome_done");
