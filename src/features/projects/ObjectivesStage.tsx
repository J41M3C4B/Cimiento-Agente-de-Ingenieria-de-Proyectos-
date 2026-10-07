import { useQuery, useQueryClient } from "@tanstack/react-query";
import { useEffect, useRef, useState } from "react";
import { Icon } from "../../components/icons";
import { Alert, Button, Tag } from "../../components/ui";
import { es } from "../../i18n/es-MX";
import { needAdd, needSelect, needsGet, needsPropose, toAppError } from "../../lib/tauri";
import type { AiStatus, ConversationView, Decision, NeedRow, NeedsView } from "../../lib/types";
import { Mark, Thinking } from "./ChatParts";
import { ConversationChat } from "./ConversationChat";
import { useProjectJob } from "./useProjectJob";
import { Workspace } from "./Workspace";

const t = es.objectives;

/**
 * Choosing the objective, as the next part of the same chat. When the person arrives the assistant proposes three
 * objectives, already in order of importance (the order is the assistant's; nobody fills in ratings), and one click
 * chooses. Whoever prefers another one writes it in the chat box. Going on is the footer of the panel, not the chat.
 */
export function ObjectivesStage({
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
  const needs = useQuery({ queryKey: ["needs", projectId], queryFn: () => needsGet(projectId) });
  const [suggesting, setSuggesting] = useState(false);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [aiNotice, setAiNotice] = useState<AiStatus | null>(null);
  const asked = useRef(false);
  // what the AI is doing is asked of the program: if the person left and came back, the work is still going
  const job = useProjectJob(projectId, ["needs"]);
  const proposing = suggesting || job.running === "needs";

  const v = needs.data;
  const setNeeds = (n: NeedsView) => qc.setQueryData(["needs", projectId], n);
  const chosen = v?.needs.find((n) => n.selected) ?? null;
  const working = busy || parentBusy;

  async function suggest() {
    job.dismiss();
    setSuggesting(true);
    setError(null);
    setAiNotice(null);
    try {
      const out = await needsPropose(projectId);
      setNeeds(out.view);
      if (out.ai !== "used" && out.ai !== "skipped") setAiNotice(out.ai);
    } catch (e) {
      const err = toAppError(e);
      // already being proposed (it started before the person left): not an error, wait for it
      if (err.code === "already_running") void job.refresh();
      else setError(err.message);
    } finally {
      setSuggesting(false);
    }
  }

  // the objectives are proposed as soon as the person arrives, once, and only if the AI is not already doing it
  // or did it while they were away (if that went wrong they decide whether to try again)
  useEffect(() => {
    if (v && v.needs.length === 0 && !asked.current && !parentBusy && job.ready && !job.runningHere && !job.finishedAway) {
      asked.current = true;
      void suggest();
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [v?.needs.length, job.ready, job.runningHere, job.finishedAway]);

  async function choose(n: NeedRow) {
    setBusy(true);
    setError(null);
    try {
      setNeeds(await needSelect(projectId, n.id));
    } catch (e) {
      setError(toAppError(e).message);
    } finally {
      setBusy(false);
    }
  }

  // an objective the person writes in the chat box is added and chosen in one go
  async function write(text: string, decision?: Decision) {
    const out = await needAdd(projectId, text, "", decision);
    if (out.status === "quarantine") return out;
    const added = out.view.needs[out.view.needs.length - 1];
    setNeeds(added ? await needSelect(projectId, added.id) : out.view);
    return { status: "saved" as const };
  }

  const list = v?.needs ?? [];
  const noticeText = aiNotice ? es.aiNotice[aiNotice] : job.notice ? es.aiNotice[job.notice] : "";
  const firstAi = list.find((n) => n.origin === "ai_assumption")?.id;
  const proposedByAi = list.filter((n) => n.origin === "ai_assumption").length;

  const tail = (
    <div className="space-y-5">
      <div className="anim-rise flex gap-3">
        <Mark active={proposing} />
        <div className="min-w-0 flex-1 space-y-1">
          <p className="text-[12px] font-medium text-stone-600">{es.conversation.assistant}</p>
          <p className="max-w-[64ch] text-[15px] leading-relaxed text-stone-900">{t.intro}</p>
        </div>
      </div>

      {proposing && <Thinking phrases={t.thinking} />}
      {noticeText && <Alert tone="warn">{noticeText}</Alert>}
      {error && <Alert tone="error">{error}</Alert>}

      {!proposing && list.length === 0 && v && (
        <div className="space-y-3">
          <p className="text-[14px] text-stone-700">{t.empty}</p>
          <Button variant="primary" onClick={suggest} disabled={working || !job.ready}>
            {t.retry}
          </Button>
        </div>
      )}

      {list.length > 0 && (
        <ul className="space-y-3">
          {list.map((n) => (
            <li
              key={n.id}
              className={`anim-rise rounded-2xl bg-white p-5 shadow-card transition-shadow ${n.selected ? "ring-2 ring-stone-900" : "hover:shadow-panel"}`}
            >
              <div className="flex flex-wrap items-start justify-between gap-3">
                <div className="min-w-0 flex-1 space-y-1.5">
                  <div className="flex flex-wrap items-center gap-2">
                    <h3 className="text-[15px] font-semibold">{n.title}</h3>
                    {proposedByAi > 1 && n.id === firstAi && (
                      <Tag tone="green" icon="check">
                        {t.recommended}
                      </Tag>
                    )}
                    {n.origin === "user" && <Tag>{t.yours}</Tag>}
                  </div>
                  {n.description && <p className="leading-relaxed text-stone-800">{n.description}</p>}
                </div>
                <Button variant={n.selected ? "secondary" : "primary"} onClick={() => choose(n)} disabled={working || n.selected}>
                  {n.selected && <Icon name="check" size={16} strokeWidth={3} />}
                  {n.selected ? t.chosen : t.choose}
                </Button>
              </div>
            </li>
          ))}
        </ul>
      )}

      {chosen && (
        <div className="anim-rise flex gap-3">
          <Mark />
          <div className="min-w-0 flex-1 space-y-1">
            <p className="text-[12px] font-medium text-stone-600">{es.conversation.assistant}</p>
            <p className="max-w-[64ch] text-[15px] leading-relaxed text-stone-900">{t.chosenNote(chosen.title)}</p>
          </div>
        </div>
      )}
    </div>
  );

  const next = chosen ? (
    <div className="space-y-3">
      <p className="text-[12px] font-medium uppercase tracking-[0.08em] text-stone-600">{es.diagnosis.nextStep}</p>
      <div className="rounded-xl bg-stone-100 px-3.5 py-3">
        <p className="text-[12px] text-stone-600">{t.yourChoice}</p>
        <p className="text-[14px] font-semibold leading-snug">{chosen.title}</p>
      </div>
      <Button variant="primary" className="w-full" onClick={onContinue} disabled={working}>
        {es.projects.continue}
        <Icon name="next" size={15} />
      </Button>
    </div>
  ) : (
    <div className="space-y-1.5">
      <p className="text-[12px] font-medium uppercase tracking-[0.08em] text-stone-600">{es.diagnosis.nextStep}</p>
      <p className="text-[13px] text-stone-600">{t.nextHint}</p>
    </div>
  );

  return (
    <Workspace panelOpen={panelOpen} readingId={view.project.call_reading_id} footer={next}>
      <ConversationChat view={view} onView={onView} disabled={working || proposing} tail={tail} later={{ placeholder: t.placeholder, hint: t.hint, send: write }} />
    </Workspace>
  );
}
