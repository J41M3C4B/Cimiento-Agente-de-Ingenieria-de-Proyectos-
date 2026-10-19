import { listen } from "@tauri-apps/api/event";
import { useQueryClient } from "@tanstack/react-query";
import { useEffect, useState, useSyncExternalStore } from "react";
import { Toast } from "../../components/ui";
import { es } from "../../i18n/es-MX";
import type { ReadingProgress } from "../../lib/files";
import type { AttachOutcome, RecordRead, RecordSuggested } from "../../lib/types";
import { type Asked, RecordPlaceDialog } from "./RecordPlaceDialog";

/**
 * The documents of the Expediente being read in the background (ADR-038). Rust reads a file while the person goes on
 * working, even with the window of the document closed, and says how it ended with `record-read`; this keeps which
 * ones are being read, how far they are, how each one ended, and which windows are open, so nothing ends in silence.
 */
export type Reading = { kind: string; progress: ReadingProgress | null };
export type Ended = RecordRead & { n: number };
type Snapshot = { reading: ReadonlyMap<string, Reading>; ended: readonly Ended[] };

let snapshot: Snapshot = { reading: new Map(), ended: [] };
const listeners = new Set<() => void>();
const open = new Map<string, number>();
let count = 0;

function set(next: Snapshot) {
  snapshot = next;
  listeners.forEach((l) => l());
}

const subscribe = (l: () => void) => {
  listeners.add(l);
  return () => listeners.delete(l);
};

/** What names a document while it is read: its type, or `other:<id>` (as Rust names it). */
export const recordKey = (kind: string, id: string | null) => (kind === "other" && id ? `other:${id}` : kind);

/** A file of this document started to be read. */
export function markReading(key: string, kind: string) {
  const reading = new Map(snapshot.reading);
  reading.set(key, { kind, progress: null });
  set({ ...snapshot, reading });
}

export function markProgress(p: ReadingProgress) {
  const now = p.key ? snapshot.reading.get(p.key) : undefined;
  if (!p.key || !now) return;
  const reading = new Map(snapshot.reading);
  reading.set(p.key, { ...now, progress: { done: p.done, total: p.total } });
  set({ ...snapshot, reading });
}

/** A reading ended: it leaves the ones being read, and how it ended is kept (the last few). */
export function markEnded(r: RecordRead): Ended {
  const reading = new Map(snapshot.reading);
  reading.delete(r.key);
  const ended = { ...r, n: ++count };
  set({ reading, ended: [...snapshot.ended.slice(-9), ended] });
  return ended;
}

export const useRecordReadings = () => useSyncExternalStore(subscribe, () => snapshot);

/** While a window of a document is open it says itself how its reading ended; the notice of the app does not. */
export function useOpenWindow(key: string) {
  useEffect(() => {
    open.set(key, (open.get(key) ?? 0) + 1);
    return () => {
      const n = (open.get(key) ?? 1) - 1;
      if (n > 0) open.set(key, n);
      else open.delete(key);
    };
  }, [key]);
}

/** For the tests: as when the program starts. */
export function resetRecordReadings() {
  snapshot = { reading: new Map(), ended: [] };
  open.clear();
}

/** The name of a document in a notice: the one of its type, or «Otro documento». */
export const titleOf = (kind: string) => es.profile.records.types[kind] ?? es.profile.records.newOther;

/** What a reading that ended says, in a notice. `attached` gives the words of a file put on its document. */
export function endedText(r: RecordRead, attached: (out: Extract<AttachOutcome, { status: "attached" }>) => string): string {
  const t = es.profile.records;
  const out = r.outcome;
  // a file that was not kept is asked about, never told in a notice (ADR-042 §4)
  const what = r.error
    ? r.error.message
    : out?.status === "attached"
      ? attached(out)
      : out?.status === "unreadable"
        ? (t.unreadable[out.reason] ?? t.unreadable.damaged!)
        : t.unreadable.damaged!;
  return t.readEnded(titleOf(r.kind), what);
}

/** The windows of «Mi institución» by their form, as the person knows them. */
const windowName = (form: string) => {
  const c = es.profile.cards;
  return ({ "institution.identity": c.institution, "institution.contact": c.contact, "institution.legal": c.legal, "institution.capacity": c.capacity } as Record<string, string>)[form] ?? form;
};

/**
 * What a file just kept left in «Mi institución» (ADR-042 §5), in a notice: the suggestions and their windows, or
 * that the help could not look. Nothing to say when it found nothing or there is no help configured.
 */
export function suggestedText(r: RecordSuggested): string | null {
  const t = es.profile.records;
  const out = r.outcome;
  if (r.error || (out?.status === "unavailable" && out.reason !== "not_configured")) return t.suggestFailed(titleOf(r.kind));
  if (out?.status !== "proposed" || out.fields.length === 0) return null;
  const windows = [...new Set(out.fields.map(([form]) => windowName(form)))].map((w) => `«${w}»`);
  const listed = windows.length > 1 ? `${windows.slice(0, -1).join(", ")} y ${windows[windows.length - 1]}` : windows[0]!;
  return t.suggestedFromFile(titleOf(r.kind), out.fields.length, listed);
}

/**
 * Hears the readings of the Expediente for the whole app, wherever the person is: how far each one is, and, when
 * one ends, what changed is fetched again and, if its window is not open, a notice says how it ended, or the question
 * of a file that was not kept (ADR-042 §4) is asked here. What a file kept left in «Mi institución» comes after, in
 * its own notice (ADR-042 §5).
 */
export function RecordReadingWatcher({ attachedText }: { attachedText: (out: Extract<AttachOutcome, { status: "attached" }>) => string }) {
  const qc = useQueryClient();
  const [notice, setNotice] = useState<{ tone: "ok" | "error"; text: string } | null>(null);
  const [asked, setAsked] = useState<Asked | null>(null);

  useEffect(() => {
    let gone = false;
    const stops: (() => void)[] = [];
    const hear = <T,>(event: string, f: (payload: T) => void) =>
      listen<T>(event, (e) => f(e.payload))
        .then((unlisten) => (gone ? unlisten() : stops.push(unlisten)))
        // outside the program (the tests) there is nobody to hear
        .catch(() => {});
    void hear<ReadingProgress>("reading-progress", markProgress);
    void hear<RecordRead>("record-read", (r) => {
      markEnded(r);
      // what a file changes: the record, its window, the suggestions of «Mi institución», Documentos
      for (const key of [["institution"], ["form"], ["documents"], ["profile"]]) void qc.invalidateQueries({ queryKey: key });
      if (open.has(r.key)) return;
      if (r.outcome?.status === "ask") {
        setAsked({ key: r.key, kind: r.kind, outcome: r.outcome });
        return;
      }
      const failed = !!r.error || r.outcome?.status === "unreadable";
      setNotice({ tone: failed ? "error" : "ok", text: endedText(r, attachedText) });
    });
    void hear<RecordSuggested>("record-suggested", (r) => {
      for (const key of [["form"], ["profile"], ["institution"]]) void qc.invalidateQueries({ queryKey: key });
      const text = suggestedText(r);
      if (text) setNotice({ tone: r.outcome?.status === "proposed" ? "ok" : "error", text });
    });
    return () => {
      gone = true;
      stops.forEach((s) => s());
    };
  }, [qc, attachedText]);

  useEffect(() => {
    if (!notice) return;
    const id = window.setTimeout(() => setNotice(null), 9000);
    return () => window.clearTimeout(id);
  }, [notice]);

  return (
    <>
      {notice && <Toast tone={notice.tone}>{notice.text}</Toast>}
      {asked && (
        <RecordPlaceDialog
          asked={asked}
          onAnswered={(said) => {
            setAsked(null);
            // what is kept says itself when it ends; only a failure needs a word here
            if (said?.tone === "error") setNotice({ tone: "error", text: said.text });
          }}
        />
      )}
    </>
  );
}
