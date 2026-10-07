import { createContext, useContext, useState } from "react";
import type { ReactNode } from "react";
import { Card, Folder, Segmented } from "../../components/ui";
import type { Tone } from "../../components/ui";
import { es } from "../../i18n/es-MX";
import { CallIndex } from "../calls/CallIndex";

type Tab = "call" | "project";

/**
 * What the page that holds a working step gives its chat: the folder it lives in (the project's color and title) and
 * what goes at the top of its body (the step and the steps). Without it (a step drawn by itself) the chat has no folder.
 */
export type ProjectFrame = { tone: Tone; title: string; chip?: ReactNode; head: ReactNode };
export const ProjectFrameContext = createContext<ProjectFrame | null>(null);

/**
 * The shape of a working step: what happens in the middle (the chat, in the project's folder) and, beside it, a panel.
 * The panel always has the call as an index; from the drafting on it also has the project (`project`), which opens
 * first. What to do next, whatever the step, is the `footer` of the panel, whichever part is open, and never goes
 * inside the chat.
 */
export function Workspace({
  children,
  panelOpen,
  readingId,
  project,
  projectLabel,
  footer,
}: {
  children: ReactNode;
  panelOpen: boolean;
  readingId: string | null;
  project?: ReactNode;
  /** What the project part of the panel is called at this step (the project, the review, the guide). */
  projectLabel?: string;
  footer?: ReactNode;
}) {
  const [tab, setTab] = useState<Tab>(project ? "project" : "call");
  const frame = useContext(ProjectFrameContext);
  const showing: Tab = project ? tab : "call";
  const chat = <div className="flex min-h-0 min-w-0 flex-1 flex-col">{children}</div>;
  return (
    <div className="flex h-full min-h-0 gap-4">
      {frame ? (
        <Folder tone={frame.tone} title={frame.title} chip={frame.chip} className="min-h-[560px] min-w-0 flex-1 lg:min-h-0" bodyClassName="min-h-0 !gap-4 !pb-4">
          {frame.head}
          {chat}
        </Folder>
      ) : (
        <Card className="flex min-h-0 min-w-0 flex-1 flex-col">{chat}</Card>
      )}
      {panelOpen && (
        <aside
          aria-label={es.workspace.panelTitle}
          className="card flex w-[380px] shrink-0 flex-col max-lg:fixed max-lg:inset-y-3 max-lg:right-3 max-lg:z-30 max-lg:w-[min(380px,92vw)] max-lg:shadow-float lg:self-stretch"
        >
          {project && (
            <div className="mb-4 shrink-0">
              <Segmented
                label={es.workspace.panelTabs}
                value={showing}
                onChange={setTab}
                items={[
                  { id: "project", label: projectLabel ?? es.workspace.tabProject },
                  { id: "call", label: es.workspace.tabCall },
                ]}
              />
            </div>
          )}
          <div id={`panel-${showing}`} className="min-h-0 flex-1">
            {showing === "project" ? project : <CallIndex readingId={readingId} />}
          </div>
          {footer && <footer className="mt-4 shrink-0 border-t border-line pt-4">{footer}</footer>}
        </aside>
      )}
    </div>
  );
}
