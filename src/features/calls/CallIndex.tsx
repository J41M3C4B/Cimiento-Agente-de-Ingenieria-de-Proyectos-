import { useState } from "react";
import { Alert, Modal } from "../../components/ui";
import { es } from "../../i18n/es-MX";
import { CallCardView } from "./CallCardView";
import { Understood, useReading } from "./CallReading";

const t = es.calls;
const w = es.workspace;

/**
 * The call, in the background of the conversation: its card, compact (what it is, the figures that matter, a few
 * points, the warnings that are true) and, one link away, the whole structured reading to consult (ADR-024).
 */
export function CallIndex({ readingId }: { readingId: string | null }) {
  const detail = useReading(readingId);
  const [all, setAll] = useState(false);

  if (readingId === null) return <Alert tone="warn">{t.callMissing}</Alert>;
  const d = detail.data;
  if (!d) return <p className="px-6 py-5 text-stone-700">{es.common.loading}</p>;
  const r = d.reading;

  return (
    <div className="flex h-full min-h-0 flex-col">
      <header className="space-y-1 px-6 pb-4 pt-6">
        <p className="text-[12px] font-medium uppercase tracking-[0.08em] text-stone-600">{w.panelTitle}</p>
        <h2 className="text-[16px] font-semibold leading-snug">{r.name}</h2>
        {(r.funder || r.year) && <p className="text-[13px] text-stone-700">{t.byFunder(r.funder, r.year)}</p>}
      </header>

      <div className="min-h-0 flex-1 overflow-y-auto px-6 pb-6">
        {d.card ? (
          <CallCardView detail={d} compact onSeeDetail={() => setAll(true)} />
        ) : (
          <Alert tone="info">{r.note && t.notes[r.note] ? t.notes[r.note] : w.stillReading}</Alert>
        )}
      </div>

      {all && (
        <Modal title={r.name} size="lg" dismissable onClose={() => setAll(false)}>
          <Understood detail={d} />
        </Modal>
      )}
    </div>
  );
}
