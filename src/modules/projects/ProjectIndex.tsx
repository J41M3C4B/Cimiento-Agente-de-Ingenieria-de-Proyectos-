import { Icon } from "../../components/icons";
import { Eyebrow, Inset, Tag, Tile } from "../../components/ui";
import type { IconName } from "../../components/icons";
import { es } from "../../i18n/es-MX";
import { formatMxn } from "../../lib/format";
import { projectTone } from "../../lib/palette";
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
  const rows: { id: ProjectPart; icon: IconName; title: string; preview: string; chip: [string, Tone] }[] = view
    ? [
        {
          id: "budget",
          icon: "banknote",
          title: t.budgetTitle,
          preview: view.budget.items.length > 0 ? x.budgetPreview(view.budget.items.length, formatMxn(view.budget.totals.total)) : preparing ? x.preparing : x.noItems,
          chip: view.budget.confirmed ? [x.confirmed, "green"] : view.budget.missing_prices > 0 ? [x.missingCosts, "amber"] : [x.toConfirm, "amber"],
        },
        {
          id: "schedule",
          icon: "calendar",
          title: t.scheduleTitle,
          preview: view.schedule.activities.length > 0 ? x.schedulePreview(view.schedule.activities.length, view.schedule.duration_months) : preparing ? x.preparing : x.noActivities,
          chip: view.schedule.confirmed ? [x.confirmed, "green"] : [x.toConfirm, "amber"],
        },
        {
          id: "texts",
          icon: "pencil",
          title: t.tabs.texts,
          preview: counts && counts.written > 0 ? x.textsPreview(counts.confirmed, counts.total) : x.notWritten,
          chip: counts && counts.pendingRequired === 0 && counts.written > 0 ? [x.confirmed, "green"] : counts && counts.written > 0 ? [x.toConfirm, "amber"] : [x.notWritten, "neutral"],
        },
      ]
    : [];

  const tone = view ? projectTone(view.project.color, view.project.id) : "sky";
  return (
    <div className="flex h-full min-h-0 flex-col">
      <div className="min-h-0 flex-1 space-y-4 overflow-y-auto">
        {view?.objective && (
          <Inset>
            <Eyebrow>{x.objective}</Eyebrow>
            <p className="mt-1 text-ui font-bold">{view.objective}</p>
          </Inset>
        )}
        <p className="text-small text-ink-3">{x.help}</p>
        <ul className="space-y-1">
          {rows.map((r) => (
            <li key={r.id}>
              <button type="button" onClick={() => onOpen(r.id)} className="list-row w-full text-left">
                <Tile icon={r.icon} tone={tone} />
                <span className="min-w-0 flex-1">
                  <span className="block font-bold">{r.title}</span>
                  <span className="block text-small text-ink-3">{r.preview}</span>
                </span>
                <Tag tone={r.chip[1]} variant={r.chip[1] === "neutral" ? "line" : "solid"} icon={r.chip[1] === "green" ? "check" : undefined}>
                  {r.chip[0]}
                </Tag>
                <Icon name="next" size={16} className="shrink-0 text-ink-3" />
              </button>
            </li>
          ))}
        </ul>
      </div>
    </div>
  );
}
