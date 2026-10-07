import { useQuery } from "@tanstack/react-query";
import { useState } from "react";
import type { ReactNode } from "react";
import { Icon } from "../../components/icons";
import { Alert, Bar, Button, Card, Disclosure, Facts, Inset, Modal, Tag, Tile, Tip } from "../../components/ui";
import { es } from "../../i18n/es-MX";
import { callReadingGet } from "../../lib/tauri";
import type { CallSummary, ReadingDetail, ReadingRow, SummaryItem } from "../../lib/types";

const t = es.calls;

/** A reading that is still going, or was just saved and has not started: the screen keeps asking. */
export function isBusy(r: ReadingRow): boolean {
  return r.status === "reading" || (r.status === "waiting" && !r.note);
}

/** Where a line was read, as a small mark that says it when pointed at: «p. 7» → «bases.pdf, página 7». */
export function Cite({ file, page }: { file: string | null; page: number | null }) {
  const text = t.source(file, page);
  if (!text) return null;
  return (
    <span className="ml-1.5 inline-block align-middle">
      <Tip text={text}>
        <button
          type="button"
          aria-label={text}
          className="inline-flex min-h-6 items-center gap-1 rounded-tick bg-inset px-2 text-caption font-bold text-ink-3 transition-colors hover:bg-sky/20 hover:text-ink"
        >
          <Icon name="file" size={12} />
          {page ? `p. ${page}` : ""}
        </button>
      </Tip>
    </span>
  );
}

export function Line({ item }: { item: SummaryItem }) {
  return (
    <li className="py-3 first:pt-0 last:pb-0">
      <p className="text-ui leading-relaxed">
        {item.requirement && t.requirement[item.requirement] && (
          <span className="mr-2 align-middle">
            <Tag tone={item.requirement === "obligatorio" ? "sky" : "neutral"} variant={item.requirement === "obligatorio" ? "soft" : "line"}>
              {t.requirement[item.requirement]}
            </Tag>
          </span>
        )}
        {item.text}
        <Cite file={item.file} page={item.page} />
      </p>
      {item.applies_to && <p className="mt-1 text-small text-ink-3">{t.appliesTo(item.applies_to)}</p>}
    </li>
  );
}

/** What matters most of a call, at a glance: the amounts and the closing date. */
function KeyFacts({ summary }: { summary: CallSummary }) {
  const tiles: { label: string; value: string; file: string | null; page: number | null }[] = [
    ...summary.amounts.filter((a) => a.kind !== "modality" && a.value).map((a) => ({ label: t.amountKinds[a.kind], value: a.value, file: a.file, page: a.page })),
    ...summary.dates.filter((d) => d.kind === "cierre").map((d) => ({ label: d.label || t.closing, value: d.when, file: d.file, page: d.page })),
  ];
  if (tiles.length === 0) return null;
  return (
    <Inset>
      <Facts
        columns={2}
        items={tiles.map((x, i): [string, ReactNode] => [
          // the same label may come twice (two closing dates): the index keeps them apart
          i > 0 && tiles.slice(0, i).some((y) => y.label === x.label) ? `${x.label} (${i + 1})` : x.label,
          <>
            {x.value}
            <Cite file={x.file} page={x.page} />
          </>,
        ])}
      />
    </Inset>
  );
}

/** What was understood of a call, in parts that open and close. The sources are marks with a note, not lines. */
export function CallSummaryView({ summary }: { summary: CallSummary }) {
  const facts: [string, string | null][] = [
    [t.funder, summary.funder],
    [t.edition, summary.edition],
    [t.objective, summary.objective],
  ];
  const dates = summary.dates.filter((d) => d.kind !== "cierre");
  const modalities = summary.amounts.filter((a) => a.kind === "modality");
  const known = facts.filter(([, v]) => v) as [string, string][];
  return (
    <div className="space-y-4">
      {summary.title && <h3 className="text-heading font-bold leading-snug">{summary.title}</h3>}
      {summary.document_kind && (
        <p className="text-small text-ink-2">
          {t.documentKind}: {t.documentKinds[summary.document_kind.class] ?? summary.document_kind.words}
        </p>
      )}
      {summary.document_kind && ["aviso", "guia", "formato", "anexo", "otro"].includes(summary.document_kind.class) && <Alert tone="warn">{t.notACall}</Alert>}
      <KeyFacts summary={summary} />
      {known.length > 0 && (
        <Inset>
          <Facts columns={1} items={known} />
        </Inset>
      )}

      {summary.conflicts.length > 0 && (
        <Disclosure title={t.conflictsTitle} count={summary.conflicts.length} tone="warn" defaultOpen>
          <p className="mb-3 text-small text-ink-2">{t.conflictsHelp}</p>
          <ul className="space-y-4">
            {summary.conflicts.map((c, i) => (
              <li key={i}>
                <p className="text-ui font-bold">{c.note || c.field}</p>
                <ul className="mt-1 space-y-1">
                  {c.versions.map((v, j) => (
                    <li key={j} className="text-ui">
                      «{v.text}»
                      <Cite file={v.file} page={v.page} />
                    </li>
                  ))}
                </ul>
              </li>
            ))}
          </ul>
        </Disclosure>
      )}

      {dates.length > 0 && (
        <Disclosure title={t.datesTitle} count={dates.length}>
          <ul className="divide-y divide-line">
            {dates.map((d, i) => (
              <li key={i} className="py-3 text-ui first:pt-0 last:pb-0">
                <strong className="font-bold">{d.label}</strong>
                {d.label && " — "}
                {d.when}
                <Cite file={d.file} page={d.page} />
              </li>
            ))}
          </ul>
        </Disclosure>
      )}

      {modalities.length > 0 && (
        <Disclosure title={t.amountsTitle} count={modalities.length}>
          <ul className="divide-y divide-line">
            {modalities.map((a, i) => (
              <li key={i} className="py-3 text-ui first:pt-0 last:pb-0">
                <strong className="font-bold">{a.label ?? t.amountKinds[a.kind]}</strong>
                {a.value && ` — ${a.value}`}
                <Cite file={a.file} page={a.page} />
              </li>
            ))}
          </ul>
        </Disclosure>
      )}

      {summary.groups.map((g) => (
        <Disclosure key={g.key} title={t.groups[g.key] ?? g.key} count={g.items.length}>
          <ul className="divide-y divide-line">
            {g.items.map((item, i) => (
              <Line key={i} item={item} />
            ))}
          </ul>
        </Disclosure>
      ))}

      {summary.missing.length > 0 && (
        <Disclosure title={t.missingTitle} count={summary.missing.length}>
          <p className="mb-3 text-small text-ink-2">{t.missingHelp}</p>
          <ul className="list-disc space-y-1 pl-6 text-ui">
            {summary.missing.map((m) => (
              <li key={m}>{t.missing[m] ?? m}</li>
            ))}
          </ul>
        </Disclosure>
      )}

      {summary.doubts.length > 0 && (
        <Disclosure title={t.doubtsTitle} count={summary.doubts.length}>
          <ul className="list-disc space-y-1 pl-6 text-ui">
            {summary.doubts.map((d, i) => (
              <li key={i}>{d}</li>
            ))}
          </ul>
        </Disclosure>
      )}
    </div>
  );
}

export function Differences({ detail }: { detail: ReadingDetail }) {
  if (detail.differences.length === 0) return null;
  return (
    <Alert tone="info">
      <h4 className="text-ui font-bold">{t.differencesTitle}</h4>
      <ul className="mt-2 list-disc space-y-1 pl-6">
        {detail.differences.map((d) => (
          <li key={d.field}>{d.field === "funder" ? t.differenceFunder(d.said, d.read) : t.differenceYear(d.said, d.read)}</li>
        ))}
      </ul>
    </Alert>
  );
}

/** The whole understanding of a call: the differences with what the person wrote, the parts and how complete it was. */
export function Understood({ detail, title = true }: { detail: ReadingDetail; title?: boolean }) {
  const q = detail.quality;
  const read = q && q.blocks_total > 0 ? (q.blocks_read / q.blocks_total) * 100 : 0;
  return (
    <div className="space-y-4">
      {title && (
        <div>
          <h3 className="text-heading font-bold">{t.summaryTitle}</h3>
          <p className="mt-1 max-w-[64ch] text-ui text-ink-2">{t.summaryHelp}</p>
        </div>
      )}
      <Differences detail={detail} />
      {detail.summary ? <CallSummaryView summary={detail.summary} /> : <p className="text-ui text-ink-2">{t.notReadyYet}</p>}
      {q && (
        <Disclosure title={t.qualityTitle}>
          <div className="space-y-3">
            <Bar percent={read} tone={q.blocks_read >= q.blocks_total ? "green" : "amber"} label={t.qualityTitle} />
            <p className="text-ui">{t.quality(q.blocks_read, q.blocks_total, q.quotes_verified_percent, q.pages_with_quotes, q.pages)}</p>
          </div>
        </Disclosure>
      )}
    </div>
  );
}

/** Loads the reading of a call and keeps asking while it is busy. */
export function useReading(readingId: string | null) {
  return useQuery<ReadingDetail>({
    queryKey: ["call-reading", readingId],
    queryFn: () => callReadingGet(readingId as string),
    enabled: readingId !== null,
    refetchInterval: (q) => (q.state.data && isBusy(q.state.data.reading) ? 3000 : false),
  });
}

export const statusTone = (s: ReadingRow["status"]) => (s === "ready" ? "green" : s === "failed" ? "red" : s === "partial" || s === "waiting" ? "amber" : "sky");

/**
 * The call of a project, after its first step: one line with its name and who calls, and the whole understanding
 * opens in a window. (In its first step the call is confirmed in `CallConfirm`.)
 */
export function CallPanel({ readingId }: { readingId: string | null }) {
  const [open, setOpen] = useState(false);
  const detail = useReading(readingId);

  if (readingId === null) return <Alert tone="warn">{t.callMissing}</Alert>;
  const d = detail.data;
  if (!d) return <p className="text-ui text-ink-3">{es.common.loading}</p>;
  const r = d.reading;
  const canView = r.status === "ready" || r.status === "partial";

  return (
    <>
      <Card as="section" small className="flex flex-wrap items-center gap-3">
        <Tile icon="file" />
        <div className="min-w-0 flex-1 basis-48">
          <b className="block truncate text-ui font-bold">{r.name}</b>
          <small className="block truncate text-small font-medium text-ink-3">{[t.callTitle, t.byFunder(r.funder, r.year)].filter(Boolean).join(" · ")}</small>
        </div>
        {r.confirmed_at ? (
          <Tag tone="green" icon="check">
            {t.confirmedShort}
          </Tag>
        ) : (
          r.status !== "ready" && <Tag tone={statusTone(r.status)}>{t.status[r.status]}</Tag>
        )}
        {canView && (
          <Button size="sm" onClick={() => setOpen(true)}>
            {t.viewSummary}
          </Button>
        )}
      </Card>
      {open && (
        <Modal title={r.name} size="lg" dismissable onClose={() => setOpen(false)}>
          <Understood detail={d} />
        </Modal>
      )}
    </>
  );
}
