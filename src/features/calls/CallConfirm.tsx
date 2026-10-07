import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import { Icon } from "../../components/icons";
import { Alert, Button, Card, Eyebrow, FileTile, Inset, ListRow, Modal, Tag } from "../../components/ui";
import { es } from "../../i18n/es-MX";
import { callReadingConfirm, callReadingRetry, toAppError } from "../../lib/tauri";
import { CallCardView } from "./CallCardView";
import { Differences, isBusy, statusTone, Understood, useReading } from "./CallReading";

const t = es.calls;

/** «bases.pdf» → «pdf», for the colored square of the file. */
const fileExt = (name: string) => (name.includes(".") ? (name.split(".").pop() ?? "") : "·");

/**
 * The first step of a project: confirm that the call that was read is the one the person wants to use. One calm
 * column: what this is, the call and its files, anything that does not match what the person wrote, and the card of
 * the call (what it is in a few words, the figures that matter, whether it fits, the warnings that are true). The
 * whole structured reading is one link away, to consult. Confirming is the bar at the bottom (ADR-024).
 */
export function CallConfirm({ readingId, onContinue, busy: parentBusy = false }: { readingId: string | null; onContinue: () => void; busy?: boolean }) {
  const qc = useQueryClient();
  const [all, setAll] = useState(false);
  const [notice, setNotice] = useState<string | null>(null);
  const detail = useReading(readingId);
  const retry = useMutation({
    mutationFn: () => callReadingRetry(readingId as string),
    onSuccess: async () => {
      setNotice(t.retried);
      await qc.invalidateQueries({ queryKey: ["call-reading", readingId] });
    },
    onError: (e) => setNotice(toAppError(e).message),
  });
  const confirm = useMutation({
    mutationFn: () => callReadingConfirm(readingId as string),
    onSuccess: async () => {
      setNotice(null);
      await qc.invalidateQueries({ queryKey: ["call-reading", readingId] });
    },
    onError: (e) => setNotice(toAppError(e).message),
  });

  if (readingId === null) return <Alert tone="warn">{t.callMissing}</Alert>;
  const d = detail.data;
  if (!d) return <p className="text-ui text-ink-3">{es.common.loading}</p>;
  const r = d.reading;
  const canView = r.status === "ready" || r.status === "partial";
  const canRetry = (r.status === "waiting" || r.status === "partial" || r.status === "failed") && !isBusy(r);
  const notACall = d.summary?.document_kind && ["aviso", "guia", "formato", "anexo", "otro"].includes(d.summary.document_kind.class);

  return (
    <div className="space-y-4">
      <header className="space-y-2">
        <h2 className="text-subtitle font-bold tracking-tight">{t.step1Title}</h2>
        <p className="max-w-[60ch] text-body text-ink-2">{t.step1Intro}</p>
      </header>

      <Card as="section">
        <div className="flex flex-wrap items-start justify-between gap-3">
          <div className="min-w-0 space-y-1">
            <Eyebrow>{t.callTitle}</Eyebrow>
            <h3 className="text-heading font-bold leading-snug">{r.name}</h3>
            {(r.funder || r.year) && <p className="text-ui text-ink-3">{t.byFunder(r.funder, r.year)}</p>}
          </div>
          <Tag tone={statusTone(r.status)} icon={r.status === "ready" ? "check" : undefined}>
            {t.status[r.status]}
          </Tag>
        </div>
        {r.files.length > 0 && (
          <ul className="-mx-2 mt-4 space-y-1">
            {r.files.map((f) => (
              <ListRow
                key={f.document_id}
                lead={<FileTile ext={fileExt(f.name)} small />}
                title={<span title={f.name}>{f.name}</span>}
                detail={t.pagesCount(f.pages)}
                state={f.role === "main" ? <Tag tone="sky" variant="soft">{t.roleOf(f.role)}</Tag> : <Tag variant="line">{t.roleOf(f.role)}</Tag>}
              />
            ))}
          </ul>
        )}
        {(canRetry || notice) && (
          <Inset className="mt-4 flex flex-wrap items-center gap-3 !py-3">
            {notice && <p className="min-w-0 flex-1 text-small font-semibold text-ink-2">{notice}</p>}
            {canRetry && (
              <Button size="sm" variant="secondary" onClick={() => retry.mutate()} disabled={retry.isPending}>
                {t.retry}
              </Button>
            )}
          </Inset>
        )}
      </Card>

      {(r.note && t.notes[r.note]) || d.differences.length > 0 || notACall ? (
        <div className="space-y-3">
          {r.note && t.notes[r.note] && <Alert tone="warn">{t.notes[r.note]}</Alert>}
          <Differences detail={d} />
          {notACall && <Alert tone="warn">{t.notACall}</Alert>}
        </div>
      ) : null}

      {canView && d.card && (
        <Card as="section">
          <CallCardView detail={d} onSeeDetail={() => setAll(true)} />
        </Card>
      )}

      <Card small className="sticky bottom-4 z-10 flex flex-wrap items-center justify-between gap-4 shadow-float">
        {!canView && <p className="text-ui text-ink-2">{t.waitingToConfirm}</p>}
        {canView && !r.confirmed_at && (
          <>
            <div className="min-w-0">
              <p className="text-ui font-bold">{t.askConfirm}</p>
              <p className="text-small text-ink-3">{t.stageHelp}</p>
            </div>
            <Button variant="primary" onClick={() => confirm.mutate()} disabled={confirm.isPending || parentBusy}>
              {confirm.isPending ? t.confirming : t.confirm}
            </Button>
          </>
        )}
        {canView && r.confirmed_at && (
          <>
            <Alert tone="ok">{t.confirmedYes}</Alert>
            <Button variant="primary" onClick={onContinue} disabled={parentBusy}>
              {t.toDiagnosis}
              <Icon name="next" size={16} />
            </Button>
          </>
        )}
      </Card>

      {all && (
        <Modal title={r.name} size="lg" dismissable onClose={() => setAll(false)}>
          <Understood detail={d} />
        </Modal>
      )}
    </div>
  );
}
