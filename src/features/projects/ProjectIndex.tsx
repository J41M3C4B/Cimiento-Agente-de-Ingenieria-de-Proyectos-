import { Icon } from "../../components/icons";
import { Tag } from "../../components/ui";
import { es } from "../../i18n/es-MX";
import { formatMxn } from "../../lib/format";
import type { DraftingView } from "../../lib/types";

const t = es.drafting;
const x = t.index;

export type ProjectPart = "budget" | "schedule" | "texts";

type Tone = "neutral" | "green" | "amber";

/** How many texts there are to write, how many are confirmed, and how many are written at all. */
export function textCounts(v: DraftingView) {
  const texts = v.sections.filter((s) => s.kind === "text");
  return {
    total: texts.length,
    confirmed: texts.filter((s) => s.status === "confirmed").length,
    written: texts.filter((s) => s.content.trim() !== "").length,
    pendingRequired: texts.filter((s) => s.required && s.status !== "confirmed").length,
  };
}

/** How many parts still wait for the person: required texts, the budget and the schedule. */
export function missingParts(v: DraftingView): number {
  return textCounts(v).pendingRequired + (v.budget.confirmed ? 0 : 1) + (v.schedule.confirmed ? 0 : 1);
}

/**
 * The project, in the background of the chat: one line for each part (the objective, the budget, the schedule and the
 * texts) with how it stands. The assistant fills it in while it works; the person opens a part in a window to read
 * it and correct it.
 */
export function ProjectIndex({ view, preparing, onOpen }: { view: DraftingView | undefined; preparing: boolean; onOpen: (part: ProjectPart) => void }) {
  const counts = view ? textCounts(view) : null;
  const rows: { id: ProjectPart; title: string; preview: string; chip: [string, Tone] }[] = view
    ? [
        {
          id: "budget",
          title: t.budgetTitle,
          preview: view.budget.items.length > 0 ? x.budgetPreview(view.budget.items.length, formatMxn(view.budget.totals.total)) : preparing ? x.preparing : x.noItems,
          chip: view.budget.confirmed ? [x.confirmed, "green"] : view.budget.missing_prices > 0 ? [x.missingCosts, "amber"] : [x.toConfirm, "amber"],
        },
        {
          id: "schedule",
          title: t.scheduleTitle,
          preview: view.schedule.activities.length > 0 ? x.schedulePreview(view.schedule.activities.length, view.schedule.duration_months) : preparing ? x.preparing : x.noActivities,
          chip: view.schedule.confirmed ? [x.confirmed, "green"] : [x.toConfirm, "amber"],
        },
        {
          id: "texts",
          title: t.tabs.texts,
          preview: counts && counts.written > 0 ? x.textsPreview(counts.confirmed, counts.total) : x.notWritten,
          chip: counts && counts.pendingRequired === 0 && counts.written > 0 ? [x.confirmed, "green"] : counts && counts.written > 0 ? [x.toConfirm, "amber"] : [x.notWritten, "neutral"],
        },
      ]
    : [];

  return (
    <div className="flex h-full min-h-0 flex-col">
      <header className="space-y-1 px-6 pb-5 pt-6">
        <p className="text-[12px] font-medium uppercase tracking-[0.08em] text-stone-600">{es.workspace.tabProject}</p>
        {view?.objective && (
          <>
            <p className="text-[12px] text-stone-600">{x.objective}</p>
            <h2 className="text-[15px] font-semibold leading-snug">{view.objective}</h2>
          </>
        )}
      </header>
      <div className="min-h-0 flex-1 overflow-y-auto">
        <p className="px-6 pb-3 text-[13px] text-stone-600">{x.help}</p>
        <ul className="border-t border-stone-200">
          {rows.map((r) => (
            <li key={r.id} className="border-b border-stone-200">
              <button type="button" onClick={() => onOpen(r.id)} className="flex w-full items-center gap-3 px-6 py-4 text-left transition-colors hover:bg-stone-50">
                <span className="min-w-0 flex-1">
                  <span className="block text-[14px] font-semibold">{r.title}</span>
                  <span className="mt-0.5 block truncate text-[13px] text-stone-600">{r.preview}</span>
                </span>
                <Tag tone={r.chip[1]} icon={r.chip[1] === "green" ? "check" : undefined}>
                  {r.chip[0]}
                </Tag>
                <Icon name="next" size={16} className="shrink-0 text-stone-500" />
              </button>
            </li>
          ))}
        </ul>
      </div>
    </div>
  );
}
