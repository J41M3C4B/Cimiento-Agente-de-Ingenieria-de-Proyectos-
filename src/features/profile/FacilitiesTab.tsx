import { AddSlot, Inset, RowActions, Tag, THead } from "../../components/ui";
import type { TagTone } from "../../components/ui";
import { es } from "../../i18n/es-MX";
import type { Condition, FacilityInput } from "../../lib/types";

const t = es.profile;

const conditionTone: Record<Condition, TagTone> = { good: "green", fair: "amber", poor: "red", critical: "red" };

/** The spaces of the institution and how they are today: a table, with a slot to add one. */
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
      <p className="text-ui text-ink-3">{t.sections.facilitiesHelp}</p>

      {facilities.length === 0 ? (
        <Inset className="flex flex-col items-center gap-4 !px-6 !py-12 text-center">
          <p className="max-w-sm text-body text-ink-2">{t.empty.facilities}</p>
          <div className="w-full max-w-sm">
            <AddSlot onClick={onAdd}>{t.facilitiesTab.add}</AddSlot>
          </div>
        </Inset>
      ) : (
        <>
          <div className="overflow-x-auto">
            <table className="table min-w-[640px]">
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
                  <tr key={i} className="group">
                    <td>
                      <button type="button" onClick={() => onEdit(i)} className="block text-left">
                        {f.kind}
                      </button>
                    </td>
                    <td className="tabular">{f.count}</td>
                    <td>{f.condition ? <Tag tone={conditionTone[f.condition]}>{t.condition[f.condition]}</Tag> : <span className="text-ink-3">—</span>}</td>
                    <td>{f.accessible === null ? <span className="text-ink-3">—</span> : <Tag tone={f.accessible ? "green" : "amber"}>{f.accessible ? es.common.yes : es.common.no}</Tag>}</td>
                    <td className={`max-w-[280px] ${f.notes ? "" : "text-ink-3"}`}>
                      <span className="line-clamp-1">{f.notes || "—"}</span>
                    </td>
                    <td className="w-24 !pr-0 text-right">
                      <RowActions onEdit={() => onEdit(i)} onRemove={() => onRemove(i)} busy={busy} />
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
          <p className="text-small text-ink-3">{t.facilitiesTab.count(facilities.length)}</p>
          <AddSlot onClick={onAdd}>{t.facilitiesTab.add}</AddSlot>
        </>
      )}
    </div>
  );
}
