import { Icon } from "../../components/icons";
import { Button, RowActions, Tag, THead } from "../../components/ui";
import type { TagTone } from "../../components/ui";
import { es } from "../../i18n/es-MX";
import type { Condition, FacilityInput } from "../../lib/types";

const t = es.profile;

const conditionTone: Record<Condition, TagTone> = { good: "green", fair: "amber", poor: "orange", critical: "red" };

/** The spaces of the institution and how they are today: a table, with a button to add one. */
export function FacilitiesTab({
  facilities, busy, onAdd, onEdit, onRemove,
}: {
  facilities: FacilityInput[];
  busy: boolean;
  onAdd: () => void;
  onEdit: (index: number) => void;
  onRemove: (index: number) => void;
}) {
  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between gap-3">
        <p className="text-stone-700">{t.sections.facilitiesHelp}</p>
        <Button size="sm" variant="primary" onClick={onAdd}>
          <Icon name="plus" size={15} strokeWidth={2.4} />
          {t.facilitiesTab.add}
        </Button>
      </div>

      {facilities.length === 0 ? (
        <div className="rounded-xl border border-dashed border-stone-300 px-6 py-12 text-center">
          <p className="mx-auto max-w-sm text-stone-700">{t.empty.facilities}</p>
          <Button className="mt-4" variant="primary" onClick={onAdd}>
            <Icon name="plus" size={16} strokeWidth={2.4} />
            {t.facilitiesTab.add}
          </Button>
        </div>
      ) : (
        <div className="overflow-x-auto">
          <table className="w-full min-w-[640px] border-collapse text-left text-[14px]">
            <THead
              columns={[
                { key: "kind", title: t.facilitiesTab.columns.kind },
                { key: "count", title: t.facilitiesTab.columns.count },
                { key: "condition", title: t.facilitiesTab.columns.condition },
                { key: "accessible", title: t.facilitiesTab.columns.accessible },
                { key: "notes", title: t.facilitiesTab.columns.notes },
                { key: "actions", title: "" },
              ]}
            />
            <tbody>
              {facilities.map((f, i) => (
                <tr key={i} className="group border-t border-stone-200 hover:bg-stone-50">
                  <td className="px-3 py-2.5 font-medium">
                    <button type="button" onClick={() => onEdit(i)} className="text-left hover:text-blue-800">
                      {f.kind}
                    </button>
                  </td>
                  <td className="px-3 py-2.5 tabular-nums">{f.count}</td>
                  <td className="px-3 py-2.5">{f.condition ? <Tag tone={conditionTone[f.condition]}>{t.condition[f.condition]}</Tag> : "—"}</td>
                  <td className="px-3 py-2.5">
                    {f.accessible === null ? "—" : <Tag tone={f.accessible ? "green" : "orange"}>{f.accessible ? es.common.yes : es.common.no}</Tag>}
                  </td>
                  <td className="max-w-[280px] px-3 py-2.5 text-stone-700">
                    <span className="line-clamp-1">{f.notes || "—"}</span>
                  </td>
                  <td className="w-28 px-1 py-1.5 text-right">
                    <RowActions onEdit={() => onEdit(i)} onRemove={() => onRemove(i)} busy={busy} />
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
          <div className="border-t border-stone-200 px-1 py-3 text-[13px] text-stone-700">{t.facilitiesTab.count(facilities.length)}</div>
        </div>
      )}
    </div>
  );
}
