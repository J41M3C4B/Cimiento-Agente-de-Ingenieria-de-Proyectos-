// The first start (ADR-031, ADR-042): the welcome of each person, the setup of the administrator and the indispensable
// data of the institution, step by step.
import { invoke } from "@tauri-apps/api/core";
import type { Decision, FormSpec, FormValues, QuarantineReport } from "../../lib/types";

export interface StepStatus {
  key: string;
  /** The indispensable data still missing, by field id. */
  missing: string[];
  complete: boolean;
  /** Its fields, from the catalog of forms: drawn as their window draws them. */
  form: FormSpec;
}

export interface OnboardingStatus {
  /** The institution finished it once: the app opens. */
  done: boolean;
  steps: StepStatus[];
  /** Every step complete: it can be closed. */
  ready: boolean;
  /** What is saved of the fields of the steps, by field id. */
  values: FormValues;
  records: { served: number; staff: number };
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
/** Saves one step: its fields go through their window of «Mi institución». */
export const onboardingSave = (step: string, values: FormValues, decision?: Decision) => invoke<OnboardingOutcome>("onboarding_save", { step, values, decision: decision ?? null });
export const onboardingFinish = () => invoke<OnboardingStatus>("onboarding_finish");
export const onboardingWelcomeDone = () => invoke<void>("onboarding_welcome_done");
