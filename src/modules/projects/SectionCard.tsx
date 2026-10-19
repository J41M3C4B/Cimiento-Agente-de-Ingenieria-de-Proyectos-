import { useEffect, useState } from "react";
import { QuarantineDialog } from "../../components/QuarantineDialog";
import { Icon } from "../../components/icons";
import { Alert, Button, Eyebrow, Inset, Tag, TextArea } from "../../components/ui";
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
      <li>
        <Inset className="flex items-center justify-between gap-3 !py-3">
          <p className="text-ui font-bold">{section.title}</p>
          <p className="text-small text-ink-3">{t.dataSection}</p>
        </Inset>
      </li>
    );
  }

  return (
    <li>
      <Inset>
        <button type="button" aria-expanded={open} onClick={() => setOpen(!open)} className="flex min-h-ctl w-full items-center gap-3 text-left">
          <span className="min-w-0 flex-1">
            <span className="block text-ui font-bold">{section.title}</span>
            <span className="block text-small text-ink-3">{section.required ? t.required : t.optional}</span>
          </span>
          <Tag tone={statusTone[section.status]} variant={section.status === "empty" ? "line" : "solid"} icon={section.status === "confirmed" ? "check" : undefined}>
            {t.status[section.status]}
          </Tag>
          <Icon name="down" size={18} className={`shrink-0 text-ink-2 transition-transform duration-200 ${open ? "rotate-180" : ""}`} />
        </button>

        {open && (
          <div className="mt-3 space-y-4 border-t border-line pt-4">
            <Alert tone="info">
              <Eyebrow className="!text-ink-2">{t.inPlainWords}</Eyebrow>
              <p className="mt-1">{section.plain ?? section.guidance}</p>
            </Alert>
            {section.plain && section.guidance !== section.plain && (
              <details className="text-small text-ink-2">
                <summary className="cursor-pointer font-bold text-ink">{t.callSays}</summary>
                <p className="mt-2 border-l-2 border-line pl-3 italic">{section.guidance}</p>
              </details>
            )}
            {section.unsupported_figures.length > 0 && section.status !== "confirmed" && <Alert tone="warn">{t.unsupported(section.unsupported_figures.join(", "))}</Alert>}
            {section.open_points.length > 0 && (
              <Alert tone="info">
                <p className="font-bold">{t.openPoints}</p>
                <ul className="list-disc pl-6">
                  {section.open_points.map((p) => (
                    <li key={p}>{p}</li>
                  ))}
                </ul>
              </Alert>
            )}
            <TextArea label={section.title} hideLabel rows={8} value={text} placeholder={t.textPlaceholder} onChange={(e) => setText(e.target.value)} />
            {notice && <Alert tone="warn">{notice}</Alert>}
            {error && <Alert tone="error">{error}</Alert>}
            <div className="flex flex-wrap gap-2">
              <Button size="sm" onClick={draft} disabled={working}>
                <Icon name="sparkles" size={16} />
                {busy ? t.drafting : t.draft}
              </Button>
              <Button size="sm" onClick={() => save()} disabled={working || !text.trim() || !changed}>
                {t.save}
              </Button>
              <Button size="sm" variant="primary" onClick={confirm} disabled={working || !section.content.trim() || changed || section.status === "confirmed"}>
                {t.confirmSection}
              </Button>
            </div>
          </div>
        )}
      </Inset>

      {report && <QuarantineDialog report={report} busy={busy} onRedact={() => save("redact")} onNotPersonal={() => save("not_personal")} onCancel={() => setReport(null)} />}
    </li>
  );
}
