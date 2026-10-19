import { useQuery, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import { Icon } from "../../components/icons";
import { AddSlot, Alert, Button, Inset, RowActions, Tag, THead } from "../../components/ui";
import { QuarantineDialog } from "../../components/QuarantineDialog";
import { es } from "../../i18n/es-MX";
import { toAppError } from "../../lib/tauri";
import type { Decision, QuarantineReport } from "../../lib/types";
import { facilitiesEquipmentDelete, facilitiesEquipmentSave, facilitiesOverview, facilitiesSiteSave, facilitiesSpaceDelete, facilitiesSpaceSave } from "./api";
import { BuildingPanel } from "./BuildingPanel";
import { FacilitiesBoard } from "./FacilitiesBoard";
import { EquipmentDialog, SpaceDialog, emptyEquipment, emptySpace } from "./GroupDialog";
import { StateBar } from "./StateBar";
import type { EquipmentData, EquipmentRow, FacilitiesOutcome, FacilitiesOverview, FacilityIssue, SpaceData, SpaceRow, States } from "./types";

const f = es.facilities;
export const FACILITIES_KEY = ["facilities"] as const;
export type FacilitiesView = "spaces" | "building" | "equipment" | "board";

/** The whole of something at a glance: how many there are and how they are, in one bar. */
function Summary({ title, total, s, count }: { title: string; total: string; s: States; count: number }) {
  if (count === 0) return null;
  return (
    <Inset className="flex flex-col gap-3">
      <div className="flex flex-wrap items-baseline justify-between gap-x-4">
        <h3 className="text-ui font-bold">{title}</h3>
        <span className="text-small text-ink-3">{total}</span>
      </div>
      <StateBar s={s} count={count} thick />
    </Inset>
  );
}

/** What fails, as soft tags: the first two and how many more. */
function Problems({ list }: { list: string[] }) {
  if (list.length === 0) return <span className="text-ink-3">—</span>;
  return (
    <span className="flex flex-wrap gap-1.5">
      {list.slice(0, 2).map((p) => (
        <Tag key={p} tone="amber" variant="soft">
          {f.problems[p] ?? p}
        </Tag>
      ))}
      {list.length > 2 && <Tag variant="line">{`+${list.length - 2}`}</Tag>}
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
export function FacilitiesTab({ view, onView, onNotice }: { view: FacilitiesView; onView: (v: FacilitiesView) => void; onNotice?: (text: string) => void }) {
  const qc = useQueryClient();
  const overview = useQuery({ queryKey: FACILITIES_KEY, queryFn: facilitiesOverview });
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
      {(view === "spaces" || view === "equipment") && (
        <div className="flex flex-wrap items-center gap-3">
          <p className="max-w-[80ch] flex-1 text-ui text-ink-2">{f.help}</p>
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
      )}
      {view === "building" && <p className="max-w-[80ch] text-ui text-ink-2">{f.help}</p>}
      {error && <Alert tone="error">{error}</Alert>}

      {view === "spaces" && <Summary title={f.board.spacesState} total={f.spaces.total(total)} s={data.indicators.spaces_states} count={total} />}
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
                      <td className="tabular font-bold">{r.count}</td>
                      <td className="min-w-[220px]">
                        <StateBar s={r} count={r.count} />
                      </td>
                      <td className="max-w-[280px]">
                        <Problems list={r.problems} />
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

      {view === "equipment" && <Summary title={f.board.equipmentState} total={f.equipment.total(data.indicators.equipment)} s={data.indicators.equipment_states} count={data.indicators.equipment} />}
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
                    <td className="tabular font-bold">{r.count}</td>
                    <td className="min-w-[220px]">
                      <StateBar s={r} count={r.count} />
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

      {view === "board" && <FacilitiesBoard board={data.board} onGo={onView} />}

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
