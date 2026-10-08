import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useEffect, useRef } from "react";
import type { ReactNode } from "react";
import { Icon } from "../../components/icons";
import { Alert, Button, Eyebrow, Facts, Inset } from "../../components/ui";
import { es } from "../../i18n/es-MX";
import { callBriefMake, toAppError } from "../../lib/tauri";
import type { CallCard, CardPoint, ReadingDetail } from "../../lib/types";
import { Thinking } from "./ChatParts";
import { useCallJob } from "./useProjectJob";
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
    <ul className="space-y-2">
      {shown.map((p, i) => (
        <li key={i} className="flex gap-3 text-ui leading-relaxed">
          <span aria-hidden="true" className="mt-2 h-1.5 w-1.5 shrink-0 rounded-pill bg-ink-3" />
          <span className="min-w-0">
            {p.text}
            <Cite file={p.file} page={p.page} />
            {p.applies_to && <span className="block text-small text-ink-3">{t.appliesTo(p.applies_to)}</span>}
          </span>
        </li>
      ))}
      {rest > 0 && <li className="pl-4 text-small text-ink-3">{c.more(rest)}</li>}
    </ul>
  );
}

function Block({ title, children }: { title: string; children: ReactNode }) {
  return (
    <section aria-label={title} className="space-y-3">
      <Eyebrow>{title}</Eyebrow>
      {children}
    </section>
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
  const text = card.brief ?? card.lead;
  const pays = card.blocks.filter((b) => b.key === "fundable" || b.key === "not_fundable");
  const others = card.blocks.filter((b) => b.key === "who_can" || b.key === "supported");
  const missing = card.alerts.find((a) => a.kind === "missing");
  const conflicts = card.alerts.find((a) => a.kind === "conflicts");
  const doubts = card.alerts.find((a) => a.kind === "doubts");

  return (
    <div className="space-y-6">
      {(text || writing) && (
        <section aria-label={c.briefTitle}>
          <Inset className="space-y-2">
            <Eyebrow>{c.briefTitle}</Eyebrow>
            {text && <p className={`max-w-[64ch] font-semibold leading-relaxed ${compact ? "text-ui" : "text-body"}`}>{text}</p>}
            {card.brief && <p className="text-caption text-ink-3">{c.briefNote}</p>}
            {writing && !card.brief && <Thinking phrases={c.writing} />}
          </Inset>
        </section>
      )}

      {card.facts.length > 0 && (
        <Block title={c.factsTitle}>
          <Facts
            columns={compact ? 1 : 2}
            items={card.facts.map((f): [string, ReactNode] => [
              c.facts[f.kind] ?? f.kind,
              <>
                {f.value}
                <Cite file={f.file} page={f.page} />
              </>,
            ])}
          />
        </Block>
      )}

      {card.blocks.length > 0 && (
        <Block title={c.fitTitle}>
          <div className="space-y-4">
            {[...others, ...pays].map((b) => (
              <div key={b.key} className="space-y-2">
                <h4 className="text-ui font-bold">{c.blocks[b.key]}</h4>
                <Points points={b.points} more={b.more} limit={limit} />
              </div>
            ))}
          </div>
        </Block>
      )}

      {(conflicts || missing || doubts) && (
        <div className="space-y-3">
          {conflicts && <Alert tone="warn">{c.conflicts(conflicts.count)}</Alert>}
          {missing && (
            <Alert tone="info">
              <p className="font-bold">{c.missingTitle}</p>
              <ul className="mt-2 list-disc space-y-1 pl-5">
                {missing.fields.map((m) => (
                  <li key={m}>{t.missing[m] ?? m}</li>
                ))}
                {missing.count > missing.fields.length && <li className="list-none text-small text-ink-3">{c.more(missing.count - missing.fields.length)}</li>}
              </ul>
            </Alert>
          )}
          {doubts && <p className="text-small text-ink-2">{c.doubts(doubts.count)}</p>}
        </div>
      )}

      <Button size="sm" onClick={onSeeDetail}>
        {c.seeDetail}
        <Icon name="next" size={16} />
      </Button>
    </div>
  );
}
