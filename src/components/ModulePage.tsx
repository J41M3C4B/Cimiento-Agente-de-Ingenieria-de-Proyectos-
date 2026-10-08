import { useEffect, useState } from "react";
import type { ReactNode } from "react";
import { Icon } from "./icons";
import { MODULE_META } from "./modules";
import type { ModuleId } from "./modules";
import { Folder, Toast } from "./ui";

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
 * The frame of a module of the institution (ADR-032, docs/13 §6): a tray with a folder tab in the module's own
 * color (the icon and the name in the tab), what the module is for as its first line, and its screens inside.
 * `after` is for what goes below the tray, on the window itself (Finanzas puts its cards there).
 * Each module opens from the rail; «Mi institución» only shows a summary of it.
 */
export function ModulePage({
  module, title, intro, notice, action, children, after,
}: {
  module: Exclude<ModuleId, "projects">;
  title: string;
  intro: string;
  notice: Notice;
  /** a button at the right of the first line (the main thing to do here) */
  action?: ReactNode;
  children: ReactNode;
  after?: ReactNode;
}) {
  const m = MODULE_META[module];
  return (
    <div className="flex flex-col gap-4">
      <h1 className="sr-only">{title}</h1>
      <Folder
        tone={m.tone}
        title={
          <span className="inline-flex items-center gap-2.5">
            <Icon name={m.icon} size={18} strokeWidth={2.2} />
            {title}
          </span>
        }
      >
        <div className="flex flex-wrap items-center justify-between gap-x-6 gap-y-3">
          <p className="max-w-[72ch] text-ui text-ink-2">{intro}</p>
          {action}
        </div>
        {children}
      </Folder>
      {after}
      {notice && <Toast tone={notice.tone}>{notice.text}</Toast>}
    </div>
  );
}
