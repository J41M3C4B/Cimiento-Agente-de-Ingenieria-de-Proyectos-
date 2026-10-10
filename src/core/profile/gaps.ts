import { useQuery } from "@tanstack/react-query";
import { es } from "../../i18n/es-MX";
import { institutionOverview } from "../../lib/tauri";
import type { InstitutionOverview, Place } from "../../lib/types";

/** The institution at a glance (ADR-033): its figures, how full it is and what is missing, all decided in Rust. */
export const OVERVIEW_KEY = ["institution", "overview"];

export const useOverview = () => useQuery({ queryKey: OVERVIEW_KEY, queryFn: institutionOverview });

/** Where a piece that is still missing gets filled in: a window of «Mi institución» or the page of a module. */
export type Where = Place;

export type Gap = { code: string; text: string; where: Where };

const t = es.profile.todo;
const RECORDS: Record<string, string> = { staff_records: t.staff, served_records: t.population, spaces: t.facilities };

/** What a missing piece is called, in the words of the person. */
export const gapText = (code: string) => es.onboarding.missing[code] ?? RECORDS[code] ?? code;

/**
 * What the institution has not filled in yet, as Rust lists it (`core/overview.rs`): in the order of the first start,
 * then the people and the spaces nobody registered. This only puts it in words. «Mi institución» and Inicio remind
 * the person of it.
 */
export function useFillGaps(): { ready: boolean; gaps: Gap[]; percent: number } {
  const overview = useOverview();
  return fromOverview(overview.data);
}

export function fromOverview(o: InstitutionOverview | undefined): { ready: boolean; gaps: Gap[]; percent: number } {
  if (!o) return { ready: false, gaps: [], percent: 0 };
  return { ready: true, gaps: o.completion.gaps.map((g) => ({ code: g.code, text: gapText(g.code), where: g.place })), percent: o.completion.percent };
}
