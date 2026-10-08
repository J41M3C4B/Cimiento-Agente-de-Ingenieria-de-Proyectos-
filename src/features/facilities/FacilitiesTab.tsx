import { useQuery, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import { Icon } from "../../components/icons";
import { AddSlot, Alert, Button, Inset, RowActions, Segmented, Tag, THead } from "../../components/ui";
import { QuarantineDialog } from "../../components/QuarantineDialog";
import { es } from "../../i18n/es-MX";
import { toAppError } from "../../lib/tauri";
import type { Decision, QuarantineReport } from "../../lib/types";
import { facilitiesEquipmentDelete, facilitiesEquipmentSave, facilitiesOverview, facilitiesSiteSave, facilitiesSpaceDelete, facilitiesSpaceSave } from "./api";
import { BuildingPanel } from "./BuildingPanel";
import { FacilitiesBoard } from "./FacilitiesBoard";
import { EquipmentDialog, SpaceDialog, STATE_KEYS, STATE_TONE, emptyEquipment, emptySpace } from "./GroupDialog";
import type { EquipmentData, EquipmentRow, FacilitiesOutcome, FacilitiesOverview, FacilityIssue, SpaceData, SpaceRow, States } from "./types";

const f = es.facilities;
export const FACILITIES_KEY = ["facilities"] as const;
type View = "spaces" | "building" | "equipment" | "board";

/** «3 bien · 1 mal · 2 sin revisar», as tags with the color of each state. */
function StateTags({ s, count }: { s: States; count: number }) {
  const rest = count - (s.good + s.fair + s.poor + s.unusable);
  return (
    <span className="flex flex-wrap gap-1.5">
      {STATE_KEYS.filter((k) => s[k] > 0).map((k) => (
        <Tag key={k} tone={STATE_TONE[k]} variant="soft">{`${s[k]} ${f.states[k].toLowerCase()}`}</Tag>
      ))}
      {rest > 0 && <Tag variant="line">{f.unchecked(rest)}</Tag>}
    </span>
  );
}

const spaceName = (r: SpaceRow) => (r.kind === "other" ? (r.label ?? f.spaceKinds.other) : f.spaceKinds[r.kind] ?? r.kind);
const equipmentName = (r: EquipmentRow) => (r.kind === "other" ? (r.label ?? f.equipmentKinds.other) : f.equipmentKinds[r.kind] ?? r.kind);

/**
 * The facilities (ADR-030): the spaces in groups that count how many are in each state, the building with its
 * services and safety, the equipment, and the board that turns them into arguments for a project. Names and notes go
 * through the scanner, because they reach the automatic help.
 */
export function FacilitiesTab({ onNotice }: { onNotice?: (text: string) => void }) {
  const qc = useQueryClient();
  const overview = useQuery({ queryKey: FACILITIES_KEY, queryFn: facilitiesOverview });
  const [view, setView] = useState<View>("spaces");
  const [space, setSpace] = useState<{ id: string | null; start: SpaceData } | null>(null);
  const [item, setItem] = useState<{ id: string | null; start: EquipmentData } | null>(null);
  const [quarantine, setQuarantine] = useState<{ report: QuarantineReport; retry: (d: Decision) => void; cancel: () => void } | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const data = overview.data;

  const apply = (o: FacilitiesOverview) => qc.setQueryData(FACILITIES_KEY, o);

  /**
   * Runs a save. Resolves `null` when it was saved (the window closes), or the problems to show. A quarantine asks
   * the person first and runs it again with their decision.
   */
  function submit(run: (decision?: Decision) => Promise<FacilitiesOutcome>): Promise<FacilityIssue[] | null> {
    return new Promise((resolve) => {
      const go = async (decision?: Decision) => {
        setError(null);
        setBusy(true);
        try {
          const out = await run(decision);
          if (out.status === "saved") {
            setQuarantine(null);
            apply(out.overview);
            onNotice?.(es.common.saved);
            resolve(null);
          } else if (out.status === "invalid") {
            setQuarantine(null);
            resolve(out.issues);
          } else {
            setQuarantine({ report: out.report, retry: (d) => void go(d), cancel: () => { setQuarantine(null); resolve([]); } });
          }
        } catch (e) {
          setQuarantine(null);
          setError(toAppError(e).message);
          resolve([]);
        } finally {
          setBusy(false);
        }
      };
      void go();
    });
  }

  async function remove(job: () => Promise<FacilitiesOverview>) {
    setError(null);
    setBusy(true);
    try {
      apply(await job());
    } catch (e) {
      setError(toAppError(e).message);
    } finally {
      setBusy(false);
    }
  }

  if (!data) return overview.isError ? <Alert tone="error">{toAppError(overview.error).message}</Alert> : null;
  const spaces = data.spaces;
  const equipment = data.equipment;
  const total = data.indicators.spaces;

  return (
    <div className="space-y-4">
      <div className="flex flex-wrap items-center gap-3">
        <Segmented
          label={f.title}
          value={view}
          onChange={setView}
          items={[
            { id: "spaces", label: f.tabs.spaces, count: total },
            { id: "building", label: f.tabs.building },
            { id: "equipment", label: f.tabs.equipment, count: data.indicators.equipment },
            { id: "board", label: f.tabs.board, count: data.board.insights.length },
          ]}
        />
        <span className="flex-1" />
        {view === "spaces" && (
          <Button variant="primary" onClick={() => setSpace({ id: null, start: emptySpace(data.space_kinds[0]) })}>
            <Icon name="plus" size={16} strokeWidth={2.4} />
            {f.spaces.add}
          </Button>
        )}
        {view === "equipment" && (
          <Button variant="primary" onClick={() => setItem({ id: null, start: emptyEquipment(data.equipment_kinds[0]) })}>
            <Icon name="plus" size={16} strokeWidth={2.4} />
            {f.equipment.add}
          </Button>
        )}
      </div>
      {view !== "board" && <p className="max-w-[80ch] text-ui text-ink-2">{f.help}</p>}
      {error && <Alert tone="error">{error}</Alert>}

      {view === "spaces" &&
        (spaces.length === 0 ? (
          <Inset className="flex flex-col items-center gap-4 !px-6 !py-12 text-center">
            <p className="max-w-sm text-body text-ink-2">{f.spaces.empty}</p>
            <div className="w-full max-w-sm">
              <AddSlot onClick={() => setSpace({ id: null, start: emptySpace(data.space_kinds[0]) })}>{f.spaces.add}</AddSlot>
            </div>
          </Inset>
        ) : (
          <>
            <div className="overflow-x-auto">
              <table className="table min-w-[760px]">
                <THead
                  columns={[
                    { key: "space", title: f.spaces.columns.space },
                    { key: "count", title: f.spaces.columns.count },
                    { key: "state", title: f.spaces.columns.state },
                    { key: "problems", title: f.spaces.columns.problems },
                    { key: "actions", title: "" },
                  ]}
                />
                <tbody>
                  {spaces.map((r) => (
                    <tr key={r.id} className="group">
                      <td className="min-w-[200px]">
                        <button type="button" onClick={() => setSpace({ id: r.id, start: { ...r } })} className="block text-left">
                          <span className="block font-semibold">{spaceName(r)}</span>
                          <span className="block text-small text-ink-3">
                            {[f.floors(r.floor), r.kind !== "other" ? r.label : null, r.beds !== null ? f.spaces.beds_short(r.beds) : null].filter(Boolean).join(" · ")}
                          </span>
                        </button>
                      </td>
                      <td className="tabular">{r.count}</td>
                      <td className="min-w-[180px]">
                        <StateTags s={r} count={r.count} />
                      </td>
                      <td className="max-w-[260px]">
                        {r.problems.length > 0 ? <span className="line-clamp-2 text-small">{r.problems.map((p) => f.problems[p] ?? p).join(", ")}</span> : <span className="text-ink-3">—</span>}
                      </td>
                      <td className="w-24 !pr-0 text-right">
                        <RowActions onEdit={() => setSpace({ id: r.id, start: { ...r } })} onRemove={() => remove(() => facilitiesSpaceDelete(r.id))} busy={busy} />
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
            <p className="text-small text-ink-3">{`${f.spaces.groups(spaces.length)} · ${f.spaces.total(total)}`}</p>
            <AddSlot onClick={() => setSpace({ id: null, start: emptySpace(data.space_kinds[0]) })}>{f.spaces.add}</AddSlot>
          </>
        ))}

      {view === "building" && <BuildingPanel site={data.site.data} issues={data.site.issues} onSave={(d) => submit((decision) => facilitiesSiteSave(d, decision))} />}

      {view === "equipment" &&
        (equipment.length === 0 ? (
          <Inset className="flex flex-col items-center gap-4 !px-6 !py-12 text-center">
            <p className="max-w-sm text-body text-ink-2">{f.equipment.empty}</p>
            <div className="w-full max-w-sm">
              <AddSlot onClick={() => setItem({ id: null, start: emptyEquipment(data.equipment_kinds[0]) })}>{f.equipment.add}</AddSlot>
            </div>
          </Inset>
        ) : (
          <div className="overflow-x-auto">
            <table className="table min-w-[640px]">
              <THead
                columns={[
                  { key: "item", title: f.equipment.columns.item },
                  { key: "count", title: f.equipment.columns.count },
                  { key: "state", title: f.equipment.columns.state },
                  { key: "actions", title: "" },
                ]}
              />
              <tbody>
                {equipment.map((r) => (
                  <tr key={r.id} className="group">
                    <td className="min-w-[200px]">
                      <button type="button" onClick={() => setItem({ id: r.id, start: { ...r } })} className="block text-left">
                        <span className="block font-semibold">{equipmentName(r)}</span>
                        {r.kind !== "other" && r.label && <span className="block text-small text-ink-3">{r.label}</span>}
                      </button>
                    </td>
                    <td className="tabular">{r.count}</td>
                    <td>
                      <StateTags s={r} count={r.count} />
                    </td>
                    <td className="w-24 !pr-0 text-right">
                      <RowActions onEdit={() => setItem({ id: r.id, start: { ...r } })} onRemove={() => remove(() => facilitiesEquipmentDelete(r.id))} busy={busy} />
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        ))}

      {view === "board" && <FacilitiesBoard board={data.board} />}

      {space && (
        <SpaceDialog
          start={space.start}
          isNew={space.id === null}
          kinds={data.space_kinds}
          floors={data.site.data.floors}
          onClose={() => setSpace(null)}
          onSave={async (d) => {
            const out = await submit((decision) => facilitiesSpaceSave(space.id, d, decision));
            if (out === null) setSpace(null);
            return out;
          }}
        />
      )}
      {item && (
        <EquipmentDialog
          start={item.start}
          isNew={item.id === null}
          kinds={data.equipment_kinds}
          onClose={() => setItem(null)}
          onSave={async (d) => {
            const out = await submit((decision) => facilitiesEquipmentSave(item.id, d, decision));
            if (out === null) setItem(null);
            return out;
          }}
        />
      )}
      {quarantine && (
        <QuarantineDialog
          report={quarantine.report}
          busy={busy}
          onRedact={() => quarantine.retry("redact")}
          onNotPersonal={() => quarantine.retry("not_personal")}
          onCancel={quarantine.cancel}
        />
      )}
    </div>
  );
}
