import { useEffect, useState } from "react";
import { QuarantineDialog } from "../../components/QuarantineDialog";
import { Icon } from "../../components/icons";
import { Alert, Button, Chip, Disclosure, IconTile, Modal, TextArea } from "../../components/ui";
import { es } from "../../i18n/es-MX";
import { diagnosisSummaryConfirm, diagnosisSummaryEdit, diagnosisSummaryGenerate, toAppError } from "../../lib/tauri";
import type { AiStatus, ConversationView, Decision, QuarantineReport, SummaryEdit, SummaryJson } from "../../lib/types";
import { Said, Thinking } from "./ChatParts";
import { ConversationChat } from "./ConversationChat";
import { useProjectJob } from "./useProjectJob";
import { Workspace } from "./Workspace";

const t = es.diagnosis;

type Pending = { edit: SummaryEdit; report: QuarantineReport };

const lines = (s: string) => s.split("\n").map((x) => x.trim()).filter(Boolean);

/**
 * The diagnosis: the conversation in the middle, with its box always at the bottom like any chat, and the call beside
 * it as an index. What to do next never goes in the chat: when the root cause is confirmed the panel's footer offers
 * to make the summary; the summary opens in a window (it is read once, then it is not the main thing), where the
 * person confirms it or corrects it, and then goes on.
 */
export function DiagnosisPanel({
  view,
  onView,
  onContinue,
  busy: parentBusy,
  panelOpen = false,
}: {
  view: ConversationView;
  onView: (v: ConversationView) => void;
  onContinue: () => void;
  busy: boolean;
  /** The call's panel beside the chat (it holds the next step). */
  panelOpen?: boolean;
}) {
  const [busy, setBusy] = useState(false);
  // the summary is being written (by the AI, which takes a while, so the chat shows it working)
  const [making, setMaking] = useState(false);
  const [aiNotice, setAiNotice] = useState<AiStatus | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [pending, setPending] = useState<Pending | null>(null);
  const [editing, setEditing] = useState(false);
  const [open, setOpen] = useState(false);

  const project = view.project.id;
  // what the AI is doing is asked of the program: if the person left and came back, the work is still going
  const job = useProjectJob(project, ["turn", "summary"]);
  const working = busy || parentBusy || !job.ready || job.runningHere;
  const summarizing = making || job.running === "summary";
  const summary = view.summary;

  // a summary that is waiting for the person opens by itself, once
  const waiting = summary !== null && summary.confirmed_at === null;
  useEffect(() => {
    if (waiting) setOpen(true);
  }, [waiting]);

  async function guarded<T>(fn: () => Promise<T>): Promise<T | undefined> {
    setBusy(true);
    setError(null);
    try {
      return await fn();
    } catch (e) {
      setPending(null);
      const err = toAppError(e);
      // the AI was already doing this (it started before the person left): not an error, just wait for it
      if (err.code === "already_running") void job.refresh();
      else setError(err.message);
    } finally {
      setBusy(false);
    }
  }

  async function makeSummary() {
    job.dismiss();
    setAiNotice(null);
    setMaking(true);
    try {
      const out = await guarded(() => diagnosisSummaryGenerate(project));
      if (!out) return;
      setAiNotice(out.ai);
      onView(out.view);
    } finally {
      setMaking(false);
    }
  }

  async function saveEdit(edit: SummaryEdit, decision?: Decision) {
    const out = await guarded(() => diagnosisSummaryEdit(project, edit, decision));
    if (!out) return;
    if (out.status === "quarantine") return setPending({ edit, report: out.report });
    setPending(null);
    setEditing(false);
    onView(out.view);
  }

  async function confirm() {
    const v = await guarded(() => diagnosisSummaryConfirm(project));
    if (v) onView(v);
  }

  const noticeText = aiNotice ? es.aiNotice[aiNotice] : job.notice ? es.aiNotice[job.notice] : "";
  const confirmed = summary?.confirmed_at != null;

  // the chat keeps its voice after the conversation: it shows the summary being written, then says it is there
  const tail = summarizing ? (
    <Thinking phrases={t.makingPhrases} />
  ) : summary && !confirmed ? (
    <div className="space-y-3">
      <Said>{t.summaryReady}</Said>
      <div className="pl-11">
        <Button onClick={() => setOpen(true)}>{t.summaryOpen}</Button>
      </div>
    </div>
  ) : null;
  const tailKey = summarizing ? "making" : summary && !confirmed ? "ready" : "";

  // what to do next, in the panel: nothing yet while they talk, then the summary and the way on
  const next =
    view.phase !== "closed" && !(view.legacy && summary) ? (
      <p className="text-[13px] text-stone-600">{t.nextHint}</p>
    ) : (
      <div className="space-y-3">
        <p className="text-[12px] font-medium uppercase tracking-[0.08em] text-stone-600">{t.nextStep}</p>
        {!summary ? (
          <>
            <div className="flex items-start gap-3">
              <IconTile icon="check" tone="green" />
              <p className="text-[14px]">{t.allDone}</p>
            </div>
            <Button variant="primary" className="w-full" onClick={makeSummary} disabled={working}>
              {summarizing ? t.making : t.makeSummary}
            </Button>
          </>
        ) : (
          <>
            <div className="flex items-center gap-3">
              <IconTile icon="file" tone={confirmed ? "green" : "amber"} />
              <div className="min-w-0">
                <p className="text-[14px] font-semibold">{t.summaryCard}</p>
                <Chip tone={confirmed ? "green" : "amber"} icon={confirmed ? "check" : undefined}>
                  {confirmed ? t.statusConfirmed : t.statusPending}
                </Chip>
              </div>
            </div>
            <div className="grid gap-2">
              <Button className="w-full" onClick={() => setOpen(true)}>
                {t.viewSummary}
              </Button>
              {confirmed && (
                <Button variant="primary" className="w-full" onClick={onContinue} disabled={working}>
                  {es.projects.continue}
                  <Icon name="next" size={15} />
                </Button>
              )}
            </div>
          </>
        )}
      </div>
    );

  return (
    <>
      <Workspace panelOpen={panelOpen && view.project.kind === "call"} readingId={view.project.call_reading_id} footer={next}>
        {(noticeText || error) && (
          <div className="mx-auto w-full max-w-[720px] space-y-2 px-6 pt-4">
            {noticeText && <Alert tone="warn">{noticeText}</Alert>}
            {error && <Alert tone="warn">{error}</Alert>}
          </div>
        )}
        <div className="min-h-0 flex-1">
          <ConversationChat view={view} onView={onView} disabled={working} tail={tail} tailKey={tailKey} backgroundTurn={job.running === "turn"} />
        </div>
      </Workspace>

      {summary && open && (
        <Modal
          title={editing ? t.fix : confirmed ? t.confirmed : t.title}
          size="lg"
          onClose={() => {
            setOpen(false);
            setEditing(false);
          }}
          footer={
            editing ? (
              <>
                <Button onClick={() => setEditing(false)} disabled={working}>
                  {es.common.cancel}
                </Button>
                <Button variant="primary" form="summary-editor" type="submit" disabled={working}>
                  {t.saveEdits}
                </Button>
              </>
            ) : confirmed ? (
              <>
                <Button onClick={() => setOpen(false)}>{es.common.close}</Button>
                <Button variant="primary" onClick={onContinue} disabled={working}>
                  {es.projects.continue}
                  <Icon name="next" size={15} />
                </Button>
              </>
            ) : (
              <>
                <Button onClick={() => setEditing(true)} disabled={working}>
                  {t.fix}
                </Button>
                <Button variant="primary" onClick={confirm} disabled={working}>
                  {t.yes}
                </Button>
              </>
            )
          }
        >
          {editing ? (
            <SummaryEditor summary={summary.summary} onSave={(e) => saveEdit(e)} />
          ) : (
            <SummaryParts summary={summary.summary} confirmed={confirmed} fromAi={summary.origin === "ai_assumption"} unsupported={view.unsupported_figures} />
          )}
        </Modal>
      )}

      {pending && (
        <QuarantineDialog
          report={pending.report}
          busy={working}
          onRedact={() => saveEdit(pending.edit, "redact")}
          onNotPersonal={() => saveEdit(pending.edit, "not_personal")}
          onCancel={() => setPending(null)}
        />
      )}
    </>
  );
}

function List({ items }: { items: string[] }) {
  return (
    <ul className="space-y-1.5">
      {items.map((x, i) => (
        <li key={i} className="flex gap-2.5">
          <span aria-hidden className="mt-2 h-1.5 w-1.5 shrink-0 rounded-full bg-stone-500" />
          <span>{x}</span>
        </li>
      ))}
    </ul>
  );
}

/** The summary in parts that open and close: only the need is always in view. */
function SummaryParts({ summary, confirmed, fromAi, unsupported }: { summary: SummaryJson; confirmed: boolean; fromAi: boolean; unsupported: string[] }) {
  const f = t.fields;
  return (
    <div className="space-y-3">
      <p className="text-[13px] text-stone-700">{confirmed ? t.confirmed : fromAi ? t.suggested : t.edited}</p>
      {unsupported.length > 0 && !confirmed && <Alert tone="warn">{t.figuresWarning(unsupported.join(", "))}</Alert>}
      <div className="rounded-xl bg-blue-50 px-5 py-4">
        <h3 className="text-[12px] font-semibold uppercase tracking-[0.08em] text-blue-900">{f.need}</h3>
        <p className="mt-1 text-[16px] font-semibold leading-snug">{summary.reframed_need}</p>
      </div>
      <Disclosure title={f.problem} defaultOpen>
        <p>{summary.problem_statement}</p>
      </Disclosure>
      <Disclosure title={f.affected}>
        <p className="font-semibold">{[summary.affected.group, summary.affected.count !== null ? String(summary.affected.count) : ""].filter(Boolean).join(": ")}</p>
        <p>{summary.affected.description}</p>
      </Disclosure>
      {summary.current_consequences.length > 0 && (
        <Disclosure title={f.consequences} count={summary.current_consequences.length}>
          <List items={summary.current_consequences} />
        </Disclosure>
      )}
      {summary.root_causes.length > 0 && (
        <Disclosure title={f.causes} count={summary.root_causes.length}>
          <List items={summary.root_causes} />
        </Disclosure>
      )}
      {summary.alternatives.length > 0 && (
        <Disclosure title={f.alternatives} count={summary.alternatives.length}>
          <ul className="space-y-3">
            {summary.alternatives.map((a, i) => (
              <li key={i} className="rounded-lg bg-stone-50 p-3.5">
                <p className="font-semibold">{a.title}</p>
                {a.pros.length > 0 && (
                  <p className="mt-1 text-[13.5px]">
                    <strong className="text-green-800">{f.pros}:</strong> {a.pros.join("; ")}
                  </p>
                )}
                {a.cons.length > 0 && (
                  <p className="text-[13.5px]">
                    <strong className="text-amber-800">{f.cons}:</strong> {a.cons.join("; ")}
                  </p>
                )}
              </li>
            ))}
          </ul>
        </Disclosure>
      )}
      {summary.suggested_indicators.length > 0 && (
        <Disclosure title={f.indicators} count={summary.suggested_indicators.length}>
          <List items={summary.suggested_indicators} />
        </Disclosure>
      )}
      {summary.open_questions.length > 0 && (
        <Disclosure title={f.open} count={summary.open_questions.length}>
          <List items={summary.open_questions} />
        </Disclosure>
      )}
    </div>
  );
}

function SummaryEditor({ summary, onSave }: { summary: SummaryJson; onSave: (e: SummaryEdit) => void }) {
  const f = t.fields;
  const [v, setV] = useState({
    problem: summary.problem_statement,
    need: summary.reframed_need,
    affected: summary.affected.description,
    consequences: summary.current_consequences.join("\n"),
    causes: summary.root_causes.join("\n"),
    indicators: summary.suggested_indicators.join("\n"),
    open: summary.open_questions.join("\n"),
  });
  const set = (k: keyof typeof v) => (e: React.ChangeEvent<HTMLTextAreaElement>) => setV({ ...v, [k]: e.target.value });
  return (
    <form
      id="summary-editor"
      className="space-y-4"
      onSubmit={(e) => {
        e.preventDefault();
        onSave({
          problem_statement: v.problem,
          reframed_need: v.need,
          affected_description: v.affected,
          current_consequences: lines(v.consequences),
          root_causes: lines(v.causes),
          suggested_indicators: lines(v.indicators),
          open_questions: lines(v.open),
        });
      }}
    >
      <TextArea label={f.problem} rows={3} value={v.problem} onChange={set("problem")} />
      <TextArea label={f.affected} rows={3} value={v.affected} onChange={set("affected")} />
      <TextArea label={f.consequences} hint={t.editHint} rows={4} value={v.consequences} onChange={set("consequences")} />
      <TextArea label={f.causes} hint={t.editHint} rows={4} value={v.causes} onChange={set("causes")} />
      <TextArea label={f.need} rows={3} value={v.need} onChange={set("need")} />
      <TextArea label={f.indicators} hint={t.editHint} rows={4} value={v.indicators} onChange={set("indicators")} />
      <TextArea label={f.open} hint={t.editHint} rows={4} value={v.open} onChange={set("open")} />
    </form>
  );
}
