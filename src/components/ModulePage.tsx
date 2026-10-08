import { useEffect, useState } from "react";
import type { ReactNode } from "react";
import { PageHeader, Toast } from "./ui";

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
 * The frame of a module of the institution (ADR-032): its title and what it is for, then its own screens. Each
 * module opens from the rail; «Mi institución» only shows a summary of it.
 */
export function ModulePage({ title, intro, notice, children }: { title: string; intro: string; notice: Notice; children: ReactNode }) {
  return (
    <div className="flex flex-col gap-4">
      <PageHeader title={title} intro={intro} />
      {children}
      {notice && <Toast tone={notice.tone}>{notice.text}</Toast>}
    </div>
  );
}
