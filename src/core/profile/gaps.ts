import { useQuery } from "@tanstack/react-query";
import { es } from "../../i18n/es-MX";
import { facilitiesOverview } from "../../modules/facilities/api";
import { FACILITIES_KEY } from "../../modules/facilities/FacilitiesTab";
import { ONBOARDING_KEY, onboardingStatus } from "../onboarding/api";

/** Where a piece that is still missing gets filled in: a window of «Mi institución» or the page of a module. */
export type Where = "institution" | "contact" | "legal" | "capacity" | "finance" | "facilities" | "staff" | "people";

export type Gap = { code: string; text: string; where: Where };

// Rust says what each step of the data still needs (domain/onboarding.rs); this says where each one is filled in
const WHERE: Record<string, Where> = {
  name: "institution",
  mission: "institution",
  state: "contact",
  municipality: "contact",
  contact: "contact",
  legal_form: "legal",
  founded_year: "legal",
  authorized_donee: "legal",
  cluni: "legal",
  capacity_total: "capacity",
  served: "capacity",
  staff: "capacity",
  expenses: "finance",
  income: "finance",
  floors: "facilities",
  tenure: "facilities",
};

/**
 * What the institution has not filled in yet, in the order of the steps of the first start: first the data Rust asks
 * for, then the people and the spaces nobody has registered yet (unless the data above already covers them). With
 * nothing left, the data of the institution are complete. It is what «Mi institución» and Inicio remind the person of.
 */
export function useFillGaps(): { ready: boolean; gaps: Gap[] } {
  const status = useQuery({ queryKey: ONBOARDING_KEY, queryFn: onboardingStatus });
  const facilities = useQuery({ queryKey: FACILITIES_KEY, queryFn: facilitiesOverview });
  const s = status.data;
  if (!s) return { ready: false, gaps: [] };

  const gaps: Gap[] = s.steps.flatMap((step) => step.missing).map((code) => ({ code, text: es.onboarding.missing[code] ?? code, where: WHERE[code] ?? "institution" }));
  const has = (...codes: string[]) => gaps.some((g) => codes.includes(g.code));
  const t = es.profile.todo;
  if (s.records.staff === 0 && !has("staff")) gaps.push({ code: "staff_records", text: t.staff, where: "staff" });
  if (s.records.served === 0 && !has("served")) gaps.push({ code: "served_records", text: t.population, where: "people" });
  if (facilities.isSuccess && facilities.data.indicators.spaces === 0 && !has("floors", "tenure")) gaps.push({ code: "spaces", text: t.facilities, where: "facilities" });
  return { ready: true, gaps };
}
