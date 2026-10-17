import { useState } from "react";
import { Alert, Button, Modal } from "../../components/ui";
import { es } from "../../i18n/es-MX";
import { institutionRecordPlace, toAppError } from "../../lib/tauri";
import type { AttachOutcome, PlaceAnswer } from "../../lib/types";
import { markReading, titleOf } from "./recordReading";

const t = es.profile.records;
const p = t.place;

/** A file Rust did not keep because it is not sure it is the document it was put on (`key`, `kind`). */
export type Asked = { key: string; kind: string; outcome: Extract<AttachOutcome, { status: "ask" }> };

/** What the person is told after answering, if anything: the file is being kept elsewhere, or it could not be. */
export type Answered = { tone: "info" | "error"; text: string } | null;

/**
 * The question of ADR-042 §4, before anything is kept: a file whose words say it is another document goes there,
 * stays here because the person says so, or is not kept; one nothing recognizes is kept here only if the person says
 * it is this document. Rust decides the question and keeps the file; this only asks. Closing it is not keeping it.
 */
export function RecordPlaceDialog({ asked, onAnswered }: { asked: Asked; onAnswered: (said: Answered) => void }) {
  const [busy, setBusy] = useState(false);
  const [failure, setFailure] = useState<string | null>(null);
  const here = titleOf(asked.kind);
  const other = asked.outcome.looks_like ? titleOf(asked.outcome.looks_like) : null;

  async function answer(a: PlaceAnswer) {
    setBusy(true);
    setFailure(null);
    try {
      const start = await institutionRecordPlace(asked.key, a);
      if (start.status === "already_reading") {
        setFailure(t.alreadyReading);
        return;
      }
      if (start.status === "cancelled") {
        onAnswered({ tone: "info", text: p.cancelled });
        return;
      }
      // it is kept in the background, like a reading: `record-read` says how it ended
      markReading(start.key, start.kind);
      onAnswered(a === "there" && other ? { tone: "info", text: p.movedTo(other) } : null);
    } catch (e) {
      const error = toAppError(e);
      onAnswered({ tone: "error", text: error.code === "not_found" ? p.gone : error.message });
    } finally {
      setBusy(false);
    }
  }

  return (
    <Modal
      title={other ? p.otherTitle(other) : p.unsureTitle(here)}
      onClose={() => void answer("cancel")}
      footer={
        <>
          <Button onClick={() => void answer("cancel")} disabled={busy}>
            {p.cancel}
          </Button>
          {other && (
            <Button onClick={() => void answer("here")} disabled={busy}>
              {p.saveHere(here)}
            </Button>
          )}
          <Button variant="primary" onClick={() => void answer(other ? "there" : "here")} disabled={busy}>
            {other ? p.saveThere(other) : p.saveHere(here)}
          </Button>
        </>
      }
    >
      <p className="text-body">{other ? p.otherBody(here, other) : p.unsureBody}</p>
      <p className="text-small text-ink-3">{p.file(asked.outcome.file_name)}</p>
      {other && asked.outcome.there_has_file && <Alert tone="warn">{p.otherReplaces(other)}</Alert>}
      {failure && <Alert tone="error">{failure}</Alert>}
    </Modal>
  );
}
