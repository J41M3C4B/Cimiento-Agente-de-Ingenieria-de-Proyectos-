import { useEffect, useState } from "react";
import type { ReactNode } from "react";
import type { ModuleId } from "./modules";
import { Folder, PageHeader, Toast } from "./ui";

type Notice = { tone: "ok" | "error"; text: string } | null;

/** A notice that goes away by itself: what a module page says after saving. */
export function useNotice(): [Notice, (text: string, tone?: "ok" | "error") => void] {
  const [notice, setNotice] = useState<Notice>(null);
  useEffect(() => {
    if (!notice) return;
    const id = window.setTimeout(() => setNotice(null), 3500);
    return () => window.clearTimeout(id);
  }, [notice]);
  return [notice, (text, tone = "ok") => setNotice({ tone, text })];
}

/**
 * The frame of a module of the institution (ADR-032, docs/13 §6.1): the header of every page on the window (title, one
 * line and the main action), and below it a folder in the module's own color. The views of the module are the tabs of
 * the folder (`views`, the chosen one carries the accent; no tab carries an icon, the module is already told apart by the rail and its color); a module with a single view has one
 * tab named `tab`. `after` is for what goes below the folder, on the window itself (Finanzas puts its cards there).
 * Each module opens from the top bar; «Mi institución» only shows a summary of it.
 */
export function ModulePage<V extends string = string>({
  title, intro, notice, action, children, after, tab, views,
}: {
  module: Exclude<ModuleId, "projects">;
  title: string;
  intro: string;
  notice: Notice;
  /** the main thing to do here, at the right of the header */
  action?: ReactNode;
  children: ReactNode;
  after?: ReactNode;
  /** the name of the one tab, for a module with a single view */
  tab?: string;
  /** the views of the module, one tab each */
  views?: { items: { id: V; label: string; count?: number }[]; value: V; onChange: (id: V) => void };
}) {
  return (
    <div className="flex flex-col gap-6">
      <PageHeader title={title} intro={intro} action={action} />
      <div className="flex flex-col gap-4">
        {views ? (
          <Folder tone="ac" tabs={{ items: views.items, value: views.value, onChange: (id) => views.onChange(id as V), label: title }}>
            {children}
          </Folder>
        ) : (
          <Folder tone="ac" title={tab ?? title}>
            {children}
          </Folder>
        )}
        {after}
      </div>
      {notice && <Toast tone={notice.tone}>{notice.text}</Toast>}
    </div>
  );
}
