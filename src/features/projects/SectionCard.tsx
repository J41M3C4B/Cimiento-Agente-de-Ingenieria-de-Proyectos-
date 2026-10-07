import { useEffect, useState } from "react";
import { QuarantineDialog } from "../../components/QuarantineDialog";
import { Icon } from "../../components/icons";
import { Alert, Button, Tag } from "../../components/ui";
import { es } from "../../i18n/es-MX";
import { sectionConfirm, sectionDraft, sectionSave, toAppError } from "../../lib/tauri";
import type { Decision, DraftingView, QuarantineReport, SectionView } from "../../lib/types";

const t = es.drafting;

const statusTone: Record<string, "neutral" | "amber" | "green" | "red"> = {
  empty: "neutral",
  draft_ai: "amber",
  draft_user: "amber",
  confirmed: "green",
  needs_review: "red",
};

/**
 * One text of the project, as a row that opens. Closed it is its title and its state; open it explains in plain
 * words what is asked (and shows what the call says, apart), and holds the text: the assistant drafts it, the person
 * corrects it and confirms it.
 */
export function SectionCard({
  projectId,
  section,
  onView,
  disabled,
}: {
  projectId: string;
  section: SectionView;
  onView: (v: DraftingView) => void;
  disabled: boolean;
}) {
  const [open, setOpen] = useState(false);
  const [text, setText] = useState(section.content);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const [report, setReport] = useState<QuarantineReport | null>(null);
  const working = busy || disabled;

  // what the AI wrote, or what was confirmed elsewhere, replaces what is in the box
  useEffect(() => setText(section.content), [section.content]);

  async function guarded(fn: () => Promise<void>) {
    setBusy(true);
    setError(null);
    try {
      await fn();
    } catch (e) {
      setReport(null);
      setError(toAppError(e).message);
    } finally {
      setBusy(false);
    }
  }

  const draft = () =>
    guarded(async () => {
      const out = await sectionDraft(projectId, section.key);
      setNotice(out.ai === "used" ? null : es.aiNotice[out.ai]);
      onView(out.view);
    });

  const save = (decision?: Decision) =>
    guarded(async () => {
      const out = await sectionSave(projectId, section.key, text, decision);
      if (out.status === "quarantine") return setReport(out.report);
      setReport(null);
      onView(out.view);
    });

  const confirm = () => guarded(async () => onView(await sectionConfirm(projectId, section.key)));

  const isText = section.kind === "text";
  const changed = text.trim() !== section.content.trim();

  if (!isText) {
    return (
      <li className="flex items-center justify-between gap-3 rounded-xl border border-stone-200 px-4 py-3">
        <p className="font-medium">{section.title}</p>
        <p className="text-[13px] text-stone-600">{t.dataSection}</p>
      </li>
    );
  }

  return (
    <li className="rounded-xl border border-stone-200 bg-white">
      <button type="button" aria-expanded={open} onClick={() => setOpen(!open)} className="flex w-full items-center gap-3 px-4 py-3 text-left">
        <span className="min-w-0 flex-1">
          <span className="block text-[14px] font-semibold">{section.title}</span>
          <span className="block text-[12.5px] text-stone-600">{section.required ? t.required : t.optional}</span>
        </span>
        <Tag tone={statusTone[section.status]} icon={section.status === "confirmed" ? "check" : undefined}>
          {t.status[section.status]}
        </Tag>
        <Icon name="down" size={16} className={`shrink-0 text-stone-600 transition-transform duration-200 ${open ? "rotate-180" : ""}`} />
      </button>

      {open && (
        <div className="space-y-3 border-t border-stone-200 px-4 py-4">
          <div className="rounded-lg bg-blue-50 px-4 py-3">
            <p className="text-[12px] font-semibold uppercase tracking-[0.08em] text-blue-900">{t.inPlainWords}</p>
            <p className="mt-0.5 leading-relaxed">{section.plain ?? section.guidance}</p>
          </div>
          {section.plain && section.guidance !== section.plain && (
            <details className="text-[13px] text-stone-700">
              <summary className="cursor-pointer font-semibold text-stone-800">{t.callSays}</summary>
              <p className="mt-1.5 border-l-2 border-stone-300 pl-3 italic leading-relaxed">{section.guidance}</p>
            </details>
          )}
          {section.unsupported_figures.length > 0 && section.status !== "confirmed" && <Alert tone="warn">{t.unsupported(section.unsupported_figures.join(", "))}</Alert>}
          {section.open_points.length > 0 && (
            <Alert tone="info">
              <p className="font-semibold">{t.openPoints}</p>
              <ul className="list-disc pl-6">
                {section.open_points.map((p) => (
                  <li key={p}>{p}</li>
                ))}
              </ul>
            </Alert>
          )}
          <textarea
            aria-label={section.title}
            rows={8}
            value={text}
            placeholder={t.textPlaceholder}
            onChange={(e) => setText(e.target.value)}
            className="w-full rounded-lg border border-stone-300 bg-white px-4 py-3 text-[15px] leading-relaxed text-stone-900 hover:border-stone-400 focus-visible:border-blue-800 focus-visible:outline-none focus-visible:ring-[3px] focus-visible:ring-blue-100"
          />
          {notice && <Alert tone="warn">{notice}</Alert>}
          {error && <Alert tone="warn">{error}</Alert>}
          <div className="flex flex-wrap gap-2">
            <Button onClick={draft} disabled={working}>
              <Icon name="sparkles" size={16} />
              {busy ? t.drafting : t.draft}
            </Button>
            <Button onClick={() => save()} disabled={working || !text.trim() || !changed}>
              {t.save}
            </Button>
            <Button variant="primary" onClick={confirm} disabled={working || !section.content.trim() || changed || section.status === "confirmed"}>
              {t.confirmSection}
            </Button>
          </div>
        </div>
      )}

      {report && <QuarantineDialog report={report} busy={busy} onRedact={() => save("redact")} onNotPersonal={() => save("not_personal")} onCancel={() => setReport(null)} />}
    </li>
  );
}
