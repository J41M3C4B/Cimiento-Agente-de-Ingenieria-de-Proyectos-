import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import { Icon } from "../../components/icons";
import { Alert, Button, Tag, Tile, Modal } from "../../components/ui";
import { es } from "../../i18n/es-MX";
import { callReadingConfirm, callReadingRetry, toAppError } from "../../lib/tauri";
import { CallCardView } from "./CallCardView";
import { Differences, isBusy, statusTone, Understood, useReading } from "./CallReading";

const t = es.calls;

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
  if (!d) return <p>{es.common.loading}</p>;
  const r = d.reading;
  const canView = r.status === "ready" || r.status === "partial";
  const canRetry = (r.status === "waiting" || r.status === "partial" || r.status === "failed") && !isBusy(r);
  const notACall = d.summary?.document_kind && ["aviso", "guia", "formato", "anexo", "otro"].includes(d.summary.document_kind.class);

  return (
    <div className="mx-auto max-w-2xl space-y-8">
      <header className="space-y-2">
        <h2 className="text-[22px] font-semibold leading-tight tracking-tight">{t.step1Title}</h2>
        <p className="max-w-[60ch] text-stone-700">{t.step1Intro}</p>
      </header>

      <section aria-label={t.callTitle} className="overflow-hidden rounded-2xl bg-white shadow-card">
        <div className="flex flex-wrap items-start justify-between gap-3 px-6 py-5">
          <div className="min-w-0 space-y-1">
            <p className="text-[12px] font-medium uppercase tracking-[0.08em] text-stone-600">{t.callTitle}</p>
            <h3 className="text-[16px] font-semibold leading-snug">{r.name}</h3>
            {(r.funder || r.year) && <p className="text-[13px] text-stone-700">{t.byFunder(r.funder, r.year)}</p>}
          </div>
          <Tag tone={statusTone(r.status)} icon={r.status === "ready" ? "check" : undefined}>
            {t.status[r.status]}
          </Tag>
        </div>
        <ul className="divide-y divide-stone-200 border-t border-stone-200">
          {r.files.map((f) => (
            <li key={f.document_id} className="flex flex-wrap items-center gap-3 px-6 py-3.5">
              <Tile icon="file" small />
              <div className="min-w-0 flex-1">
                <p className="truncate text-[14px] font-medium">{f.name}</p>
                <p className="text-[12.5px] text-stone-600">{t.pagesCount(f.pages)}</p>
              </div>
              <Tag tone={f.role === "main" ? "sky" : "neutral"} variant="soft">{t.roleOf(f.role)}</Tag>
            </li>
          ))}
        </ul>
        {(canRetry || notice) && (
          <div className="flex flex-wrap items-center gap-3 border-t border-stone-200 bg-stone-50 px-6 py-3.5">
            {notice && <p className="min-w-0 flex-1 text-[13px] text-stone-700">{notice}</p>}
            {canRetry && (
              <Button size="sm" variant="primary" onClick={() => retry.mutate()} disabled={retry.isPending}>
                {t.retry}
              </Button>
            )}
          </div>
        )}
      </section>

      {(r.note && t.notes[r.note]) || d.differences.length > 0 || notACall ? (
        <div className="space-y-3">
          {r.note && t.notes[r.note] && <Alert tone="warn">{t.notes[r.note]}</Alert>}
          <Differences detail={d} />
          {notACall && <Alert tone="warn">{t.notACall}</Alert>}
        </div>
      ) : null}

      {canView && d.card && (
        <section className="rounded-2xl bg-white px-6 py-6 shadow-card">
          <CallCardView detail={d} onSeeDetail={() => setAll(true)} />
        </section>
      )}

      <div className="sticky bottom-4 z-10 flex flex-wrap items-center justify-between gap-4 rounded-2xl bg-white px-6 py-4 shadow-float">
        {!canView && <p className="text-stone-700">{t.waitingToConfirm}</p>}
        {canView && !r.confirmed_at && (
          <>
            <div>
              <p className="font-semibold">{t.askConfirm}</p>
              <p className="text-[13px] text-stone-700">{t.stageHelp}</p>
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
      </div>

      {all && (
        <Modal title={r.name} size="lg" dismissable onClose={() => setAll(false)}>
          <Understood detail={d} />
        </Modal>
      )}
    </div>
  );
}
