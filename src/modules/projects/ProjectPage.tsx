import { useQuery, useQueryClient } from "@tanstack/react-query";
import { useEffect, useState } from "react";
import { Icon } from "../../components/icons";
import { Alert, Button, Card, Steps, Tag } from "../../components/ui";
import { es } from "../../i18n/es-MX";
import { projectTone } from "../../lib/palette";
import { conversationGet, projectAdvance, projectGoBack, toAppError } from "../../lib/tauri";
import type { ConversationView } from "../../lib/types";
import { CallConfirm } from "./CallConfirm";
import { CallPanel } from "./CallReading";
import { DiagnosisPanel } from "./DiagnosisPanel";
import { DraftingStage } from "./DraftingStage";
import { ObjectivesStage } from "./ObjectivesStage";
import { ReadyStage } from "./ReadyStage";
import { ReviewStage } from "./ReviewStage";
import { PROJECT_STEPS, stepIndex, stepsForView } from "./steps";
import { ProjectFrameContext } from "./Workspace";
import type { ProjectFrame } from "./Workspace";

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

  const tone = projectTone(view.project.color, view.project.id);
  const stepName = es.steps[stage];

  // the way back, the steps and the panel button: what is done with the page, not with the project
  const controls = (
    <nav aria-label="Pasos del proyecto" className="flex flex-wrap items-center justify-between gap-x-4 gap-y-2 px-1">
      <p className="sr-only">{current >= 0 ? t.stepLabel(current + 1, PROJECT_STEPS.length, stepName) : stepName}</p>
      <button type="button" onClick={onBack} className="inline-flex min-h-ctl-sm items-center gap-2 text-small text-ink font-bold underline underline-offset-4">
        <Icon name="back" size={16} />
        {t.backToProjects}
      </button>
      <div className="flex flex-wrap items-center gap-2">
        {current > 0 && (
          <details className="relative">
            <summary className="inline-flex min-h-ctl-sm cursor-pointer list-none items-center rounded-pill px-3 text-small font-bold text-ink-2 hover:text-ink">{t.goBack}</summary>
            <Card small className="absolute right-0 z-20 mt-2 w-72 !shadow-float">
              <p className="text-small text-ink-2">{t.goBackHelp}</p>
              <div className="mt-3 flex flex-wrap gap-2">
                {PROJECT_STEPS.slice(0, current).map((s) => (
                  <Button key={s} size="sm" disabled={busy} onClick={() => move(() => projectGoBack(projectId, s))}>
                    {es.steps[s]}
                  </Button>
                ))}
              </div>
            </Card>
          </details>
        )}
        {working && (
          <Button size="sm" variant="secondary" aria-pressed={panel} onClick={() => setPanel(!panel)} title={panel ? w.hideCall : w.showCall}>
            <Icon name="panel" size={17} />
            <span className="hidden xl:inline">{panel ? w.hideCall : w.showCall}</span>
          </Button>
        )}
      </div>
    </nav>
  );

  // the step and the six steps, in the project's color; where the project is shown, whatever the step
  const head = (
    <div className="space-y-4">
      <div className="flex flex-wrap items-center justify-between gap-x-4 gap-y-2">
        <div className="flex flex-wrap items-center gap-2">
          <Tag tone="pc">{current >= 0 ? t.stepLabel(current + 1, PROJECT_STEPS.length, stepName) : stepName}</Tag>
          {view.project.needs_review && <Tag tone="amber">{t.review}</Tag>}
        </div>
        {by && <span className="text-small font-semibold text-ink-3">{by}</span>}
      </div>
      <Steps steps={stepsForView} current={current} />
    </div>
  );
  const frame: ProjectFrame = { tone, title: view.project.title, head };

  if (working) {
    return (
      <div className="flex flex-col gap-3 lg:h-[calc(100vh-9rem)]">
        <div className="shrink-0">{controls}</div>
        {error && <Alert tone="warn">{error}</Alert>}
        {/* the chat is the main thing, in the project's folder; the call is a panel beside it */}
        <div className="flex min-h-0 flex-1">
          <div className="min-h-[520px] min-w-0 flex-1 lg:min-h-0">
            <ProjectFrameContext.Provider value={frame}>
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
            </ProjectFrameContext.Provider>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="mx-auto max-w-4xl space-y-4">
      {controls}
      <Card>
        <h1 className="text-title font-bold leading-tight">{view.project.title}</h1>
        <div className="mt-4">{head}</div>
      </Card>

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
      {stage === "PROFILE" && (
        <Card>
          <Alert tone="info">{t.soon}</Alert>
        </Card>
      )}
    </div>
  );
}
