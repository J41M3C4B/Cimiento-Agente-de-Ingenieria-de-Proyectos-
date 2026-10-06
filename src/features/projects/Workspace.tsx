import { useState } from "react";
import type { ReactNode } from "react";
import { Tabs } from "../../components/ui";
import { es } from "../../i18n/es-MX";
import { CallIndex } from "../calls/CallIndex";

type Tab = "call" | "project";

/**
 * The shape of a working step: what happens in the middle (the chat) and, beside it, a panel. The panel always has
 * the call as an index; from the drafting on it also has the project (`project`), which opens first. What to do
 * next, whatever the step, is the `footer` of the panel, whichever part is open, and never goes inside the chat.
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
  const showing: Tab = project ? tab : "call";
  return (
    <div className="flex h-full min-h-0 gap-4">
      <div className="flex min-h-0 min-w-0 flex-1 flex-col">{children}</div>
      {panelOpen && (
        <aside
          aria-label={es.workspace.panelTitle}
          className="flex w-[380px] shrink-0 flex-col overflow-hidden rounded-2xl bg-white shadow-card max-lg:fixed max-lg:inset-y-3 max-lg:right-3 max-lg:z-30 max-lg:w-[min(380px,92vw)] max-lg:shadow-lift"
        >
          {project && (
            <div className="shrink-0 px-6 pt-5">
              <Tabs
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
          <div role="tabpanel" id={`panel-${showing}`} aria-labelledby={`tab-${showing}`} className="min-h-0 flex-1">
            {showing === "project" ? project : <CallIndex readingId={readingId} />}
          </div>
          {footer && <footer className="shrink-0 border-t border-stone-200 px-6 py-5">{footer}</footer>}
        </aside>
      )}
    </div>
  );
}
