import { useQuery, useQueryClient } from "@tanstack/react-query";
import { useEffect, useState } from "react";
import { Icon } from "../../components/icons";
import { Alert, Button, Steps } from "../../components/ui";
import { es } from "../../i18n/es-MX";
import { conversationGet, projectAdvance, projectGoBack, toAppError } from "../../lib/tauri";
import type { ConversationView } from "../../lib/types";
import { CallConfirm } from "../calls/CallConfirm";
import { CallPanel } from "../calls/CallReading";
import { DiagnosisPanel } from "./DiagnosisPanel";
import { DraftingStage } from "./DraftingStage";
import { ObjectivesStage } from "./ObjectivesStage";
import { ReadyStage } from "./ReadyStage";
import { ReviewStage } from "./ReviewStage";
import { PROJECT_STEPS, stepIndex, stepsForView } from "./steps";

const t = es.projects;
const w = es.workspace;

const wideScreen = () => typeof window === "undefined" || typeof window.matchMedia !== "function" || window.matchMedia("(min-width: 1024px)").matches;

export function ProjectPage({ projectId, onBack }: { projectId: string; onBack: () => void }) {
  const qc = useQueryClient();
  const diagnosis = useQuery({ queryKey: ["diagnosis", projectId], queryFn: () => conversationGet(projectId) });
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [panel, setPanel] = useState(wideScreen);

  const view = diagnosis.data;
  const closed = view?.phase === "closed";

  // what to do next lives in the panel, so when the conversation ends the panel opens to show it
  useEffect(() => {
    if (closed) setPanel(true);
  }, [closed]);
  const setView = (v: ConversationView) => qc.setQueryData(["diagnosis", projectId], v);
  const refresh = () => qc.invalidateQueries({ queryKey: ["diagnosis", projectId] });

  async function move(action: () => Promise<unknown>) {
    setBusy(true);
    setError(null);
    try {
      await action();
      await refresh();
      await qc.invalidateQueries({ queryKey: ["needs", projectId] });
      await qc.invalidateQueries({ queryKey: ["drafting", projectId] });
      await qc.invalidateQueries({ queryKey: ["review", projectId] });
    } catch (e) {
      setError(toAppError(e).message);
    } finally {
      setBusy(false);
    }
  }

  if (!view) return <p>{es.common.loading}</p>;
  const stage = view.project.stage;
  const current = stepIndex(stage);
  const by = view.call ? es.calls.byFunder(view.call.funder, view.call.year) : "";

  // the conversation is a workspace: the chat in the middle, free, and the call beside it as an index
  const working = ((stage === "DIAGNOSIS" || stage === "PRIORITIZATION") && view.project.kind === "call") || stage === "DRAFTING" || stage === "REVIEW" || stage === "READY";

  const header = (
    <header className="flex flex-wrap items-start justify-between gap-x-6 gap-y-3">
      <div className="min-w-0 space-y-2">
        <button type="button" onClick={onBack} className="inline-flex items-center gap-1.5 text-[13px] font-semibold text-stone-700 hover:text-blue-800">
          <Icon name="back" size={15} />
          {t.backToProjects}
        </button>
        <div className="space-y-0.5">
          <h1 className="text-[22px] font-semibold leading-tight tracking-tight">{view.project.title}</h1>
          {(by || view.project.needs_review) && (
            <p className="text-[13px] text-stone-700">
              {by}
              {view.project.needs_review && <span className={`font-semibold text-amber-800 ${by ? "ml-2" : ""}`}>{t.review}</span>}
            </p>
          )}
        </div>
      </div>
      <nav aria-label="Pasos del proyecto" className="flex items-center gap-4">
        <p className="sr-only">{current >= 0 ? t.stepLabel(current + 1, PROJECT_STEPS.length, es.steps[stage]) : es.steps[stage]}</p>
        <div className="space-y-1">
          <Steps steps={stepsForView} current={current} compact />
          {current > 0 && (
            <details className="relative">
              <summary className="cursor-pointer text-[13px] font-semibold text-blue-800">{t.goBack}</summary>
              <div className="absolute left-0 z-20 mt-2 w-72 rounded-xl border border-stone-200 bg-white p-4 shadow-lift">
                <p className="text-[13px] text-stone-700">{t.goBackHelp}</p>
                <div className="mt-3 flex flex-wrap gap-2">
                  {PROJECT_STEPS.slice(0, current).map((s) => (
                    <Button key={s} size="sm" disabled={busy} onClick={() => move(() => projectGoBack(projectId, s))}>
                      {es.steps[s]}
                    </Button>
                  ))}
                </div>
              </div>
            </details>
          )}
        </div>
        {working && (
          <Button size="sm" variant="plain" aria-pressed={panel} onClick={() => setPanel(!panel)} title={panel ? w.hideCall : w.showCall}>
            <Icon name="panel" size={17} />
            <span className="hidden xl:inline">{panel ? w.hideCall : w.showCall}</span>
          </Button>
        )}
      </nav>
    </header>
  );

  if (working) {
    return (
      <div className="flex flex-col gap-5 lg:h-[calc(100vh-3.5rem)]">
        <div className="shrink-0">{header}</div>
        {error && <Alert tone="warn">{error}</Alert>}
        {/* edge to edge, under the header: the chat is the main thing and the call is its background */}
        <div className="-mx-8 -mb-7 flex min-h-0 flex-1 border-t border-stone-200">
          <div className="min-h-[520px] min-w-0 flex-1 lg:min-h-0">
            {stage === "DIAGNOSIS" && <DiagnosisPanel view={view} onView={setView} onContinue={() => move(() => projectAdvance(projectId))} busy={busy} panelOpen={panel} />}
            {stage === "PRIORITIZATION" && <ObjectivesStage view={view} onView={setView} onContinue={() => move(() => projectAdvance(projectId))} busy={busy} panelOpen={panel} />}
            {stage === "REVIEW" && (
              <ReviewStage
                view={view}
                onView={setView}
                onContinue={() => move(() => projectAdvance(projectId))}
                onBack={() => move(() => projectGoBack(projectId, "DRAFTING"))}
                busy={busy}
                panelOpen={panel}
              />
            )}
            {stage === "READY" && <ReadyStage view={view} onView={setView} onBack={() => move(() => projectGoBack(projectId, "DRAFTING"))} busy={busy} panelOpen={panel} />}
            {stage === "DRAFTING" && <DraftingStage view={view} onView={setView} onContinue={() => move(() => projectAdvance(projectId))} busy={busy} panelOpen={panel} />}
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="mx-auto max-w-4xl space-y-6">
      {header}

      {error && <Alert tone="warn">{error}</Alert>}

      {stage === "CALL_SELECTION" && view.project.kind === "call" && (
        <CallConfirm readingId={view.project.call_reading_id} onContinue={() => move(() => projectAdvance(projectId))} busy={busy} />
      )}
      {view.project.kind === "call" && stage !== "CALL_SELECTION" && <CallPanel readingId={view.project.call_reading_id} />}

      {(stage === "DIAGNOSIS" || stage === "PRIORITIZATION") && (
        <div className="min-h-[520px] lg:h-[calc(100vh-14rem)]">
          {stage === "DIAGNOSIS" ? (
            <DiagnosisPanel view={view} onView={setView} onContinue={() => move(() => projectAdvance(projectId))} busy={busy} />
          ) : (
            <ObjectivesStage view={view} onView={setView} onContinue={() => move(() => projectAdvance(projectId))} busy={busy} panelOpen={false} />
          )}
        </div>
      )}
      {stage === "PROFILE" && <Alert tone="info">{t.soon}</Alert>}
    </div>
  );
}
