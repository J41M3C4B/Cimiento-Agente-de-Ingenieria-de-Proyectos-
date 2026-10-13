import { es } from "../../i18n/es-MX";
import type { FillOutcome } from "../../lib/types";

/** What the person is told after «Llenar con mis documentos» (C2): how many suggestions were left, or why none. */
export type FillNotice = { tone: "ok" | "info" | "error"; text: string };

const why = (reason: string) => es.assistant.reasons[reason] ?? es.assistant.reasons.unavailable!;

/** The notice of one window. The words are the program's, not the AI's: they count what the code kept. */
export function fillNotice(out: FillOutcome): FillNotice {
  if (out.status === "unavailable") return { tone: "error", text: why(out.reason) };
  if (out.status === "nothing") return { tone: "info", text: es.suggestions.nothingFromDocuments[out.reason] ?? es.suggestions.fromDocuments(0) };
  return { tone: out.fields.length > 0 ? "ok" : "info", text: es.suggestions.fromDocuments(out.fields.length) };
}

/** The notice of the four windows together: the suggestions left and in how many windows. */
export function fillAllNotice(outs: FillOutcome[]): FillNotice {
  const failed = outs.find((o) => o.status === "unavailable");
  if (failed && failed.status === "unavailable") return { tone: "error", text: why(failed.reason) };
  if (outs.length > 0 && outs.every((o) => o.status === "nothing" && o.reason === "no_documents")) {
    return { tone: "info", text: es.suggestions.nothingFromDocuments.no_documents! };
  }
  const left = outs.flatMap((o) => (o.status === "proposed" ? [o.fields.length] : []));
  const total = left.reduce((a, b) => a + b, 0);
  return { tone: total > 0 ? "ok" : "info", text: es.suggestions.fromDocumentsAll(total, left.filter((n) => n > 0).length) };
}
