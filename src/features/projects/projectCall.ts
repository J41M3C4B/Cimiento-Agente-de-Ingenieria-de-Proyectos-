import { useReading } from "../calls/CallReading";
import { es } from "../../i18n/es-MX";
import type { CallCard, ProjectRow } from "../../lib/types";
import { PROJECT_STEPS, stepIndex } from "./steps";

const MONTHS = ["enero", "febrero", "marzo", "abril", "mayo", "junio", "julio", "agosto", "septiembre", "octubre", "noviembre", "diciembre"];

/** The closing date a call card gives, as the call writes it («23 de mayo de 2027» or `2027-05-23`); `null` when it is not a date. */
export function closingDate(card: CallCard | null | undefined): Date | null {
  const text = card?.facts.find((f) => f.kind === "closing")?.value.trim();
  if (!text) return null;
  const iso = /^(\d{4})-(\d{2})-(\d{2})/.exec(text);
  if (iso) return new Date(Number(iso[1]), Number(iso[2]) - 1, Number(iso[3]));
  const words = /(\d{1,2})\s+de\s+([a-záéíóú]+)\s+(?:de\s+)?(\d{4})/i.exec(text);
  if (!words) return null;
  const month = MONTHS.indexOf(words[2]!.toLowerCase());
  return month < 0 ? null : new Date(Number(words[3]), month, Number(words[1]));
}

/** Whole days from today to a date (negative when it has passed). */
export function daysUntil(date: Date, today = new Date()): number {
  const a = Date.UTC(today.getFullYear(), today.getMonth(), today.getDate());
  const b = Date.UTC(date.getFullYear(), date.getMonth(), date.getDate());
  return Math.round((b - a) / 86_400_000);
}

export const shortDate = (d: Date) => d.toLocaleDateString("es-MX", { day: "numeric", month: "short" }).replace(".", "");

/** What a project's call says that a card shows: who calls and when it closes. Nothing is made up: what is not there is `null`. */
export function useProjectCall(project: ProjectRow) {
  const reading = useReading(project.call_reading_id);
  const funder = reading.data?.reading.funder?.trim() || reading.data?.card?.funder?.trim() || null;
  const closes = closingDate(reading.data?.card);
  return { funder, closes, days: closes ? daysUntil(closes) : null };
}

/** Where a project is: its step as a person reads it (a project still before the first counts as in the first). */
export function stepInfo(project: ProjectRow) {
  const done = project.stage === "READY";
  const at = Math.max(stepIndex(project.stage), 0);
  return { done, at, filled: done ? PROJECT_STEPS.length : at + 1, name: es.steps[PROJECT_STEPS[at]!]!, tag: es.home.stepTag(at + 1, es.steps[PROJECT_STEPS[at]!]!) };
}
