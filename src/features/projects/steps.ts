import { es } from "../../i18n/es-MX";
import type { StageName } from "../../lib/types";

/**
 * The steps a person sees in a project. «Datos de la institución» is not one of them: the project is born after
 * it, so it would always show as done. A project that is still in that stage counts as being before the first.
 */
export const PROJECT_STEPS: StageName[] = ["CALL_SELECTION", "DIAGNOSIS", "PRIORITIZATION", "DRAFTING", "REVIEW", "READY"];

/** Position of a stage among the steps, or -1 before the first. */
export function stepIndex(stage: StageName): number {
  return PROJECT_STEPS.indexOf(stage);
}

export const stepsForView = PROJECT_STEPS.map((key) => ({ key, label: es.steps[key] }));
