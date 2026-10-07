import { useQuery, useQueryClient } from "@tanstack/react-query";
import { useMemo, useState } from "react";
import { Icon } from "../../components/icons";
import { AddSlot, Alert, Avatar, Bar, Button, Inset, Search, Select, Tag, THead, toneOfText } from "../../components/ui";
import { es } from "../../i18n/es-MX";
import { toAppError } from "../../lib/tauri";
import type { ProfileView } from "../../lib/types";
import { hrOverview, hrPersonGet } from "./api";
import { modalityOf } from "./labels";
import { PersonWizard } from "./PersonWizard";
import { PositionsDialog } from "./PositionsDialog";
import type { ModalityInfo, PersonView, StaffChange, StaffOverview } from "./types";

const h = es.hr;
export const HR_KEY = ["hr"] as const;

/**
 * The staff of the institution (ADR-027): a record per person, filled in four steps, and the catalog of positions.
 * Everything stays in this computer; the profile and the AI only get how many people there are per position.
 */
export function StaffTab({ onProfile, onNotice }: { onProfile: (p: ProfileView) => void; onNotice?: (text: string) => void }) {
  const qc = useQueryClient();
  const overview = useQuery({ queryKey: HR_KEY, queryFn: hrOverview });
  const [search, setSearch] = useState("");
  const [position, setPosition] = useState("");
  const [open, setOpen] = useState<{ view: PersonView | null } | null>(null);
  const [positionsOpen, setPositionsOpen] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const data = overview.data;
  const people = data?.people ?? [];
  const positions = data?.positions ?? [];
  const titleOf = (id: string | null) => positions.find((p) => p.id === id)?.title ?? "—";

  const shown = useMemo(() => {
    const q = search.trim().toLowerCase();
    return people.filter((p) => (!position || p.position_id === position) && (!q || p.full_name.toLowerCase().includes(q) || titleOf(p.position_id).toLowerCase().includes(q)));
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [people, positions, search, position]);

  function apply(change: StaffChange) {
    qc.setQueryData<StaffOverview>(HR_KEY, change.overview);
    if (change.profile) onProfile(change.profile);
  }
  const setModalities = (modalities: ModalityInfo[]) => qc.setQueryData<StaffOverview>(HR_KEY, (old) => old && { ...old, modalities });

  async function edit(id: string) {
    setError(null);
    try {
      setOpen({ view: await hrPersonGet(id) });
    } catch (e) {
      setError(toAppError(e).message);
    }
  }

  return (
    <div className="space-y-4">
      <div className="flex flex-wrap items-center gap-3">
        <Search label={h.search} placeholder={h.search} value={search} onChange={(e) => setSearch(e.target.value)} className="w-full sm:max-w-[280px] sm:flex-1" />
        <Select
          label={h.fields.position_id}
          hideLabel
          value={position}
          onChange={(e) => setPosition(e.target.value)}
          options={[["", h.allPositions], ...positions.filter((p) => p.people > 0).map((p) => [p.id, p.title] as [string, string])]}
          className="w-full sm:w-auto sm:min-w-[220px]"
        />
        <div className="ml-auto flex flex-wrap items-center gap-2">
          <Button size="sm" variant="plain" onClick={() => setPositionsOpen(true)}>
            <Icon name="briefcase" size={16} />
            {h.positions}
          </Button>
          <Button variant="primary" onClick={() => setOpen({ view: null })}>
            <Icon name="plus" size={18} />
            {h.add}
          </Button>
        </div>
      </div>

      {!overview.isSuccess ? null : people.length === 0 ? (
        <Inset className="flex flex-col items-center gap-4 !px-6 !py-12 text-center">
          <p className="max-w-sm text-body text-ink-2">{h.empty}</p>
          <div className="w-full max-w-sm">
            <AddSlot onClick={() => setOpen({ view: null })}>{h.add}</AddSlot>
          </div>
        </Inset>
      ) : (
        <>
          <div className="overflow-x-auto">
            <table className="table min-w-[720px]">
              <THead
                columns={[
                  { key: "name", title: h.table.name },
                  { key: "position", title: h.table.position },
                  { key: "modality", title: h.table.modality },
                  { key: "status", title: h.table.status },
                  { key: "progress", title: h.table.progress },
                ]}
              />
              <tbody>
                {shown.map((p) => (
                  <tr key={p.id} className={p.status === "left" ? "text-ink-3" : ""}>
                    <td className="min-w-[220px]">
                      <button type="button" onClick={() => edit(p.id)} className="flex min-w-0 items-center gap-3 text-left">
                        <Avatar size="sm" name={p.full_name} />
                        <span className="min-w-0">
                          <span className="block">{p.full_name}</span>
                          {p.heads_up > 0 && <span className="block text-small text-ink-3">{h.headsUp(p.heads_up)}</span>}
                        </span>
                      </button>
                    </td>
                    <td>{p.position_id ? <Tag tone={toneOfText(titleOf(p.position_id))}>{titleOf(p.position_id)}</Tag> : "—"}</td>
                    <td className="max-w-[220px]">
                      <span className="line-clamp-1">{modalityOf(p.modality, data?.modalities ?? [])}</span>
                    </td>
                    <td>{h.status[p.status as keyof typeof h.status] ?? p.status}</td>
                    <td className="min-w-[140px]">
                      <Bar percent={p.progress} label={h.progress(p.progress)} tone={p.progress === 100 ? "green" : "ink"} />
                    </td>
                  </tr>
                ))}
                {shown.length === 0 && (
                  <tr>
                    <td colSpan={5} className="text-ink-2">
                      {h.noResults}
                    </td>
                  </tr>
                )}
              </tbody>
            </table>
          </div>
          <div className="text-small text-ink-3">{h.count(shown.length)}</div>
          <AddSlot onClick={() => setOpen({ view: null })}>{h.add}</AddSlot>
        </>
      )}
      {error && <Alert tone="error">{error}</Alert>}

      {open && data && (
        <PersonWizard
          view={open.view}
          positions={positions}
          modalities={data.modalities}
          fields={data.custom_fields}
          onSaved={(_, change, isNew) => {
            apply(change);
            onNotice?.(isNew ? h.added : es.common.saved);
          }}
          onChange={apply}
          onModalities={setModalities}
          onClose={() => setOpen(null)}
        />
      )}
      {positionsOpen && data && <PositionsDialog positions={positions} modalities={data.modalities} onChange={apply} onClose={() => setPositionsOpen(false)} />}
    </div>
  );
}
