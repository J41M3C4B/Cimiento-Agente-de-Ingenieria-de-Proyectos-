import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useEffect, useRef } from "react";
import { Alert } from "../../components/ui";
import { es } from "../../i18n/es-MX";
import { callBriefMake, toAppError } from "../../lib/tauri";
import type { CallCard, CardPoint, ReadingDetail } from "../../lib/types";
import { Thinking } from "../projects/ChatParts";
import { useCallJob } from "../projects/useProjectJob";
import { Cite } from "./CallReading";

const t = es.calls;
const c = es.calls.card;

/**
 * Writes the «en pocas palabras» of a call the first time the person looks at it and nobody has tried yet (also for
 * calls read before it existed). One try per visit: the program refuses a second request while one runs, and keeps
 * that a try was made, so a failure is not paid for again. The card is complete without it.
 */
function useBrief(readingId: string, pending: boolean) {
  const qc = useQueryClient();
  const job = useCallJob(readingId, ["brief"]);
  const asked = useRef(false);
  const make = useMutation({
    mutationFn: () => callBriefMake(readingId),
    onSuccess: (detail) => qc.setQueryData(["call-reading", readingId], detail),
    onError: (e) => {
      // already being written (it started before the person left): not a failure, wait for it
      if (toAppError(e).code === "already_running") void job.refresh();
    },
  });
  useEffect(() => {
    if (pending && job.ready && !job.runningHere && !asked.current) {
      asked.current = true;
      make.mutate();
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [pending, job.ready, job.runningHere]);
  return make.isPending || job.runningHere;
}

function Points({ points, more, limit }: { points: CardPoint[]; more: number; limit: number }) {
  const shown = points.slice(0, limit);
  const rest = more + (points.length - shown.length);
  return (
    <ul className="space-y-1.5">
      {shown.map((p, i) => (
        <li key={i} className="flex gap-2.5 leading-relaxed">
          <span aria-hidden="true" className="mt-2 h-1.5 w-1.5 shrink-0 rounded-full bg-stone-400" />
          <span>
            {p.text}
            <Cite file={p.file} page={p.page} />
            {p.applies_to && <span className="block text-[13px] text-stone-600">{t.appliesTo(p.applies_to)}</span>}
          </span>
        </li>
      ))}
      {rest > 0 && <li className="pl-4 text-[13px] text-stone-600">{c.more(rest)}</li>}
    </ul>
  );
}

/**
 * What the person reads first about a call (ADR-024): in a few words what it is, the figures that matter, a few
 * points on whether it fits them, and only the warnings that are true. The whole structured reading is behind
 * «Consultar el detalle». It is the card of the first step and, `compact`, the call in the panel of the project.
 */
export function CallCardView({ detail, compact = false, onSeeDetail }: { detail: ReadingDetail; compact?: boolean; onSeeDetail: () => void }) {
  const card = detail.card as CallCard;
  const writing = useBrief(detail.reading.id, detail.brief_pending);
  const limit = compact ? 2 : 3;
  const gap = compact ? "space-y-5" : "space-y-7";
  const text = card.brief ?? card.lead;
  const pays = card.blocks.filter((b) => b.key === "fundable" || b.key === "not_fundable");
  const others = card.blocks.filter((b) => b.key === "who_can" || b.key === "supported");
  const missing = card.alerts.find((a) => a.kind === "missing");
  const conflicts = card.alerts.find((a) => a.kind === "conflicts");
  const doubts = card.alerts.find((a) => a.kind === "doubts");

  return (
    <div className={gap}>
      {(text || writing) && (
        <section aria-label={c.briefTitle} className="space-y-2">
          <h3 className="text-[12px] font-medium uppercase tracking-[0.08em] text-stone-600">{c.briefTitle}</h3>
          {text && <p className="max-w-[64ch] text-[15px] leading-relaxed text-stone-900">{text}</p>}
          {card.brief && <p className="text-[12.5px] text-stone-600">{c.briefNote}</p>}
          {writing && !card.brief && <Thinking phrases={c.writing} />}
        </section>
      )}

      {card.facts.length > 0 && (
        <section aria-label={c.factsTitle} className="space-y-2">
          <h3 className="text-[12px] font-medium uppercase tracking-[0.08em] text-stone-600">{c.factsTitle}</h3>
          <dl className={`grid gap-2.5 ${compact ? "grid-cols-1" : "grid-cols-2"}`}>
            {card.facts.map((f) => (
              <div key={f.kind} className="rounded-xl bg-stone-50 px-4 py-3">
                <dt className="text-[12px] text-stone-600">{c.facts[f.kind] ?? f.kind}</dt>
                <dd className="mt-0.5 text-[15px] font-semibold leading-snug">
                  {f.value}
                  <Cite file={f.file} page={f.page} />
                </dd>
              </div>
            ))}
          </dl>
        </section>
      )}

      {card.blocks.length > 0 && (
        <section aria-label={c.fitTitle} className="space-y-4">
          <h3 className="text-[12px] font-medium uppercase tracking-[0.08em] text-stone-600">{c.fitTitle}</h3>
          {others.map((b) => (
            <div key={b.key} className="space-y-1.5">
              <h4 className="text-[14px] font-semibold">{c.blocks[b.key]}</h4>
              <Points points={b.points} more={b.more} limit={limit} />
            </div>
          ))}
          {pays.map((b) => (
            <div key={b.key} className="space-y-1.5">
              <h4 className="text-[14px] font-semibold">{c.blocks[b.key]}</h4>
              <Points points={b.points} more={b.more} limit={limit} />
            </div>
          ))}
        </section>
      )}

      {(conflicts || missing || doubts) && (
        <div className="space-y-2.5">
          {conflicts && <Alert tone="warn">{c.conflicts(conflicts.count)}</Alert>}
          {missing && (
            <Alert tone="info">
              <p className="font-semibold">{c.missingTitle}</p>
              <ul className="mt-1.5 list-disc space-y-1 pl-5">
                {missing.fields.map((m) => (
                  <li key={m}>{t.missing[m] ?? m}</li>
                ))}
                {missing.count > missing.fields.length && <li className="list-none text-[13px] text-stone-600">{c.more(missing.count - missing.fields.length)}</li>}
              </ul>
            </Alert>
          )}
          {doubts && <p className="text-[13px] text-stone-700">{c.doubts(doubts.count)}</p>}
        </div>
      )}

      <button type="button" onClick={onSeeDetail} className="text-[14px] font-semibold text-blue-800 hover:underline">
        {c.seeDetail}
      </button>
    </div>
  );
}
