import { useQuery, useQueryClient } from "@tanstack/react-query";
import { useEffect, useRef, useState } from "react";
import { Icon } from "../../components/icons";
import type { IconName } from "../../components/icons";
import { Alert, Bar, Button, Eyebrow, Modal, Tag, Tile } from "../../components/ui";
import { es } from "../../i18n/es-MX";
import { projectTone } from "../../lib/palette";
import { draftingGet, draftingPrepare, draftingSetAsks, sectionsConfirmAll, sectionsDraftAll, toAppError } from "../../lib/tauri";
import type { AiStatus, ConversationView, DraftMode, DraftingView } from "../../lib/types";
import { BudgetEditor } from "./BudgetEditor";
import { Mine, Said, Thinking } from "./ChatParts";
import { ConversationChat } from "./ConversationChat";
import { missingParts, ProjectIndex, textCounts } from "./ProjectIndex";
import type { ProjectPart } from "./ProjectIndex";
import { ScheduleEditor } from "./ScheduleEditor";
import { TextsEditor } from "./TextsEditor";
import { Workspace } from "./Workspace";

const t = es.drafting;

type Writing = "prepare" | DraftMode | null;

const textChoices: [DraftMode | "mine", IconName, "primary" | "secondary"][] = [
  ["full", "sparkles", "primary"],
  ["guide", "file", "secondary"],
  ["mine", "pencil", "secondary"],
];

/**
 * The drafting, as the next part of the same chat. The assistant says it will build the project and, while it works,
 * shows what it is doing; the objective, the budget, the schedule and the texts fill the «Proyecto» part of the panel.
 * In the chat the person only answers the two questions that are theirs (does the call ask for a proposal of its own;
 * how to write the texts: complete draft, short guide, or by themselves). Everything is read and corrected in windows
 * opened from the panel, and going on is the footer of the panel.
 */
export function DraftingStage({
  view,
  onView,
  onContinue,
  busy: parentBusy,
  panelOpen,
}: {
  view: ConversationView;
  onView: (v: ConversationView) => void;
  onContinue: () => void;
  busy: boolean;
  panelOpen: boolean;
}) {
  const projectId = view.project.id;
  const qc = useQueryClient();
  const drafting = useQuery({ queryKey: ["drafting", projectId], queryFn: () => draftingGet(projectId) });
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [aiNotice, setAiNotice] = useState<AiStatus | null>(null);
  const [writing, setWriting] = useState<Writing>(null);
  const [mine, setMine] = useState(false);
  const [open, setOpen] = useState<ProjectPart | null>(null);
  const prepared = useRef(false);
  const working = busy || parentBusy;
  const asking = writing !== null;

  const v = drafting.data;
  const setDrafting = (d: DraftingView) => qc.setQueryData(["drafting", projectId], d);

  async function prepare() {
    setWriting("prepare");
    setAiNotice(null);
    setError(null);
    try {
      const out = await draftingPrepare(projectId);
      setDrafting(out.view);
      if (out.ai !== "used" && out.ai !== "skipped") setAiNotice(out.ai);
    } catch (e) {
      setError(toAppError(e).message);
    } finally {
      setWriting(null);
    }
  }

  // the draft is prepared as soon as the person arrives, once
  useEffect(() => {
    if (v && !v.plan_ready && !prepared.current && !parentBusy) {
      prepared.current = true;
      void prepare();
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [v?.plan_ready]);

  async function guarded(fn: () => Promise<void>) {
    setBusy(true);
    setError(null);
    try {
      await fn();
    } catch (e) {
      setError(toAppError(e).message);
    } finally {
      setBusy(false);
    }
  }

  const answer = (asks: boolean) =>
    guarded(async () => {
      setDrafting(await draftingSetAsks(projectId, asks));
    });
  const confirmReady = () =>
    guarded(async () => {
      setDrafting(await sectionsConfirmAll(projectId));
    });

  async function writeAll(mode: DraftMode) {
    setWriting(mode);
    setAiNotice(null);
    setError(null);
    try {
      const out = await sectionsDraftAll(projectId, mode);
      setDrafting(out.view);
      if (out.ai !== "used" && out.ai !== "skipped") setAiNotice(out.ai);
    } catch (e) {
      setError(toAppError(e).message);
    } finally {
      setWriting(null);
    }
  }

  const counts = v ? textCounts(v) : null;
  const missing = v ? missingParts(v) : 0;
  const noticeText = aiNotice ? es.aiNotice[aiNotice] : "";
  const choosing = v !== undefined && v.plan_ready && v.asks_confirmed && counts?.written === 0 && !mine && writing !== "full" && writing !== "guide";

  const tail = (
    <div className="space-y-6">
      <Said active={writing === "prepare"}>{t.chat.start(v?.objective ?? null)}</Said>
      {writing === "prepare" && <Thinking phrases={t.prep.phrases} />}
      {noticeText && (
        <Alert tone="warn">
          <p>{noticeText}</p>
          {v && !v.plan_ready && (
            <div className="mt-2">
              <Button size="sm" onClick={prepare} disabled={asking}>
                {t.prep.retry}
              </Button>
            </div>
          )}
        </Alert>
      )}
      {error && <Alert tone="error">{error}</Alert>}

      {v?.plan_ready && (
        <>
          <Said>{t.chat.prepared(v.budget.items.length, v.schedule.activities.length)}</Said>

          <Said>
            {t.proposalTitle} {t.proposalHelp(v.asks_for_proposal)}
          </Said>
          {!v.asks_confirmed ? (
            <div className="flex flex-wrap gap-2">
              <Button variant="primary" disabled={working} onClick={() => answer(true)}>
                {t.proposalYes}
              </Button>
              <Button disabled={working} onClick={() => answer(false)}>
                {t.proposalNo}
              </Button>
            </div>
          ) : (
            <Mine text={v.asks_for_proposal ? t.proposalYes : t.proposalNo} />
          )}

          {v.asks_confirmed && (
            <>
              <Said>{t.texts.chooseTitle}</Said>
              {choosing && (
                <div className="grid grid-cols-[repeat(auto-fit,minmax(170px,1fr))] gap-3">
                  {textChoices.map(([id, icon, variant]) => (
                    <button
                      key={id}
                      type="button"
                      disabled={asking || working}
                      onClick={() => (id === "mine" ? setMine(true) : void writeAll(id))}
                      className={`radio-card-face !items-start text-left disabled:opacity-60 ${variant === "primary" ? "!border-ink bg-inset" : ""}`}
                    >
                      <Tile icon={icon} tone={variant === "primary" ? "ink" : "neutral"} />
                      <span className="min-w-0 flex-1">
                        <b className="block text-ui font-bold">{t.texts[id].title}</b>
                        <span className="mt-1 block text-small font-medium text-ink-3">{t.texts[id].text}</span>
                      </span>
                    </button>
                  ))}
                </div>
              )}
              {(writing === "full" || writing === "guide") && <Thinking phrases={t.texts.working} />}
              {mine && counts?.written === 0 && (
                <>
                  <Mine text={t.texts.mine.title} />
                  <Said>{t.chat.mineDone}</Said>
                </>
              )}
              {counts && counts.written > 0 && <Said>{t.chat.textsDone(counts.written)}</Said>}
            </>
          )}
        </>
      )}
    </div>
  );

  const totalParts = v ? v.sections.filter((s) => s.kind === "text" && s.required).length + 2 : 0;
  const doneParts = Math.max(0, totalParts - missing);

  const next = (
    <div className="space-y-3">
      <Eyebrow>{t.index.nextStep}</Eyebrow>
      {v && (
        <>
          <div className="flex items-baseline justify-between gap-3 text-ui font-semibold">
            {missing === 0 ? <Tag tone="green" icon="check">{t.allConfirmed}</Tag> : <span>{t.progress(missing)}</span>}
            <b className="tabular font-bold">{t.index.countOf(doneParts, totalParts)}</b>
          </div>
          <Bar percent={totalParts > 0 ? (doneParts / totalParts) * 100 : 0} tone={projectTone(v.project.color, v.project.id)} label={t.index.countOf(doneParts, totalParts)} />
        </>
      )}
      <Button variant="primary" className="w-full" onClick={onContinue} disabled={working || asking || !v || missing > 0}>
        {t.toReview}
        <Icon name="next" size={16} />
      </Button>
      {missing > 0 && <p className="text-small text-ink-3">{t.index.locked}</p>}
    </div>
  );

  const windows: Record<ProjectPart, { title: string; body: React.ReactNode }> | null = v
    ? {
        budget: { title: t.budgetTitle, body: <BudgetEditor view={v} onView={setDrafting} disabled={parentBusy || asking} /> },
        schedule: { title: t.scheduleTitle, body: <ScheduleEditor view={v} onView={setDrafting} disabled={parentBusy || asking} /> },
        texts: {
          title: t.sectionsTitle,
          body: <TextsEditor view={v} onView={setDrafting} disabled={working || asking} onWrite={writeAll} onConfirmReady={confirmReady} onAnswer={answer} />,
        },
      }
    : null;

  return (
    <>
      <Workspace
        panelOpen={panelOpen}
        readingId={view.project.call_reading_id}
        project={<ProjectIndex view={v} preparing={writing === "prepare"} onOpen={setOpen} />}
        footer={next}
      >
        <ConversationChat view={view} onView={onView} disabled={working || asking} tail={tail} later={{ placeholder: t.chat.placeholder, hint: t.chat.hint }} />
      </Workspace>

      {open && windows && (
        <Modal title={windows[open].title} size="xl" onClose={() => setOpen(null)}>
          {windows[open].body}
        </Modal>
      )}
    </>
  );
}
