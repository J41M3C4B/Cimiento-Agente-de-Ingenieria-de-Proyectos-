import { Icon } from "../../components/icons";
import { Button } from "../../components/ui";
import { es } from "../../i18n/es-MX";
import type { DraftMode, DraftingView } from "../../lib/types";
import { textCounts } from "./ProjectIndex";
import { SectionCard } from "./SectionCard";

const t = es.drafting;

/**
 * The texts of the project, read and corrected in a window: whether the call asks for a proposal of its own, the
 * buttons to have the rest drafted or confirmed in one go, and each section as a row that opens.
 */
export function TextsEditor({
  view,
  onView,
  disabled,
  onWrite,
  onConfirmReady,
  onAnswer,
}: {
  view: DraftingView;
  onView: (v: DraftingView) => void;
  disabled: boolean;
  onWrite: (mode: DraftMode) => void;
  onConfirmReady: () => void;
  onAnswer: (asks: boolean) => void;
}) {
  const counts = textCounts(view);
  const texts = view.sections.filter((s) => s.kind === "text");
  const readyToConfirm = texts.filter((s) => (s.status === "draft_ai" || s.status === "draft_user") && s.unsupported_figures.length === 0).length;

  return (
    <div className="space-y-5">
      <p className="text-stone-700">{t.sectionsIntro}</p>

      <div className="flex flex-wrap items-center justify-between gap-3 rounded-xl border border-stone-200 px-4 py-3">
        <p className="text-[14px]">{t.proposalConfirmed(view.asks_for_proposal)}</p>
        <Button size="sm" variant="plain" disabled={disabled} onClick={() => onAnswer(!view.asks_for_proposal)}>
          {t.proposalChange}
        </Button>
      </div>

      <div className="flex flex-wrap items-center justify-between gap-3">
        <p className="text-[14px] text-stone-700">{t.texts.progress(counts.confirmed, counts.total)}</p>
        <div className="flex flex-wrap gap-2">
          <Button size="sm" disabled={disabled} onClick={() => onWrite("full")}>
            <Icon name="sparkles" size={15} />
            {t.texts.draftPending}
          </Button>
          <Button size="sm" disabled={disabled} onClick={() => onWrite("guide")}>
            {t.texts.guideAgain}
          </Button>
          <Button size="sm" variant="primary" disabled={disabled || readyToConfirm === 0} onClick={onConfirmReady}>
            <Icon name="check" size={15} strokeWidth={2.6} />
            {t.texts.confirmReady(readyToConfirm)}
          </Button>
        </div>
      </div>

      <ul className="space-y-2.5">
        {view.sections.map((s) => (
          <SectionCard key={s.key} projectId={view.project.id} section={s} onView={onView} disabled={disabled} />
        ))}
      </ul>
    </div>
  );
}
