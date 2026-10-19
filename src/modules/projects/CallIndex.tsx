import { useState } from "react";
import { Alert, Eyebrow, Modal, Tag } from "../../components/ui";
import { es } from "../../i18n/es-MX";
import { CallCardView } from "./CallCardView";
import { statusTone, Understood, useReading } from "./CallReading";

const t = es.calls;
const w = es.workspace;

/**
 * The call, in the background of the conversation: its card, compact (what it is, the figures that matter, a few
 * points, the warnings that are true) and, one link away, the whole structured reading to consult (ADR-024).
 * It is content, not a tray: whatever holds it (the side panel) gives it its padding.
 */
export function CallIndex({ readingId }: { readingId: string | null }) {
  const detail = useReading(readingId);
  const [all, setAll] = useState(false);

  if (readingId === null) return <Alert tone="warn">{t.callMissing}</Alert>;
  const d = detail.data;
  if (!d) return <p className="text-ui text-ink-3">{es.common.loading}</p>;
  const r = d.reading;

  return (
    <div className="flex h-full min-h-0 flex-col gap-4">
      <header className="space-y-2">
        <Eyebrow>{w.panelTitle}</Eyebrow>
        <h2 className="text-body font-bold leading-snug">{r.name}</h2>
        {(r.funder || r.year) && <p className="text-small text-ink-3">{t.byFunder(r.funder, r.year)}</p>}
        <Tag tone={statusTone(r.status)} icon={r.status === "ready" ? "check" : undefined}>
          {t.status[r.status]}
        </Tag>
      </header>

      <div className="min-h-0 flex-1 overflow-y-auto">
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
