import { useQuery, useQueryClient } from "@tanstack/react-query";
import { useMemo, useState } from "react";
import { Icon } from "../../components/icons";
import { AddSlot, Alert, Avatar, Bar, Button, Inset, Search, Segmented, StatusDot, Tag, THead, toneOfText } from "../../components/ui";
import type { Tone } from "../../components/ui";
import { es } from "../../i18n/es-MX";
import { toAppError } from "../../lib/tauri";
import type { ProfileView } from "../../lib/types";
import { careOverview, carePersonGet } from "./api";
import { BeneficiaryWizard } from "./BeneficiaryWizard";
import { CareBoard } from "./CareBoard";
import { CareWaitlist } from "./CareWaitlist";
import type { BeneficiaryView, CareChange, CareOverview } from "./types";

const c = es.care;
export const CARE_KEY = ["care"] as const;
type View = "people" | "board" | "waitlist";

/** Active is the usual and gets a green dot; the other situations show as a soft tag. */
const STATUS_TONE: Record<string, Tone> = { hospitalized: "amber", discharged: "neutral", deceased: "neutral" };

/**
 * The people served (ADR-029): their records in five steps, the board that turns small data into arguments for a
 * project, and the waiting list. Everything stays in this computer; the profile and the AI only get counts.
 */
export function CareTab({ onProfile, onNotice }: { onProfile: (p: ProfileView) => void; onNotice?: (text: string) => void }) {
  const qc = useQueryClient();
  const overview = useQuery({ queryKey: CARE_KEY, queryFn: careOverview });
  const [view, setView] = useState<View>("people");
  const [search, setSearch] = useState("");
  const [open, setOpen] = useState<{ person: BeneficiaryView | null } | null>(null);
  const [error, setError] = useState<string | null>(null);
  const data = overview.data;
  const people = data?.people ?? [];

  const shown = useMemo(() => {
    const q = search.trim().toLowerCase();
    return people.filter((p) => !q || p.full_name.toLowerCase().includes(q) || p.group.toLowerCase().includes(q));
  }, [people, search]);

  function apply(change: CareChange) {
    qc.setQueryData<CareOverview>(CARE_KEY, change.overview);
    if (change.profile) onProfile(change.profile);
  }
  async function edit(id: string) {
    setError(null);
    try {
      setOpen({ person: await carePersonGet(id) });
    } catch (e) {
      setError(toAppError(e).message);
    }
  }

  if (!data) return overview.isError ? <Alert tone="error">{toAppError(overview.error).message}</Alert> : null;
  return (
    <div className="space-y-4">
      <div className="flex flex-wrap items-center gap-3">
        <Segmented
          label={c.title}
          value={view}
          onChange={setView}
          items={[
            { id: "people", label: c.tabs.people, count: data.board.indicators.served },
            { id: "board", label: c.tabs.board },
            { id: "waitlist", label: c.tabs.waitlist, count: data.board.waiting },
          ]}
        />
        {view === "people" && (
          <div className="ml-auto flex flex-wrap items-center gap-2">
            <Search label={c.search} placeholder={c.search} value={search} onChange={(e) => setSearch(e.target.value)} className="w-full sm:w-[260px]" />
            <Button variant="primary" onClick={() => setOpen({ person: null })}>
              <Icon name="plus" size={18} />
              {c.add}
            </Button>
          </div>
        )}
      </div>
      {error && <Alert tone="error">{error}</Alert>}

      {view === "board" && <CareBoard board={data.board} flavor={data.flavor} />}
      {view === "waitlist" && <CareWaitlist rows={data.waitlist} freeSeats={data.board.free_seats} onChange={apply} onAdmitted={(p, ch) => { apply(ch); setOpen({ person: p }); }} />}
      {view === "people" &&
        (people.length === 0 ? (
          <Inset className="flex flex-col items-center gap-4 !px-6 !py-12 text-center">
            <p className="max-w-sm text-body text-ink-2">{c.empty}</p>
            <div className="w-full max-w-sm">
              <AddSlot onClick={() => setOpen({ person: null })}>{c.add}</AddSlot>
            </div>
          </Inset>
        ) : (
          <>
            <div className="overflow-x-auto">
              <table className="table min-w-[720px]">
                <THead
                  columns={[
                    { key: "name", title: c.table.name },
                    { key: "group", title: c.table.group },
                    { key: "age", title: c.table.age },
                    { key: "status", title: c.table.status },
                    { key: "progress", title: c.table.progress },
                  ]}
                />
                <tbody>
                  {shown.map((p) => (
                    <tr key={p.id} className={p.status === "discharged" || p.status === "deceased" ? "text-ink-3" : ""}>
                      <td className="min-w-[220px]">
                        <button type="button" onClick={() => edit(p.id)} className="flex min-w-0 items-center gap-3 text-left">
                          <Avatar size="sm" name={p.full_name} />
                          <span className="min-w-0">
                            <span className="block">{p.full_name}</span>
                            {p.heads_up > 0 && (
                              <span className="mt-0.5 flex items-center gap-1 text-small font-semibold text-amber-ink">
                                <Icon name="warn" size={13} />
                                {c.headsUp(p.heads_up)}
                              </span>
                            )}
                          </span>
                        </button>
                      </td>
                      <td>{p.group ? <Tag tone={toneOfText(p.group)}>{p.group}</Tag> : "—"}</td>
                      <td className="tabular">{p.age !== null ? c.years(p.age) : "—"}</td>
                      <td>
                        {p.status === "active" ? (
                          <StatusDot tone="green">{c.statuses.active}</StatusDot>
                        ) : (
                          <Tag tone={STATUS_TONE[p.status] ?? "neutral"} variant="soft">
                            {c.statuses[p.status] ?? p.status}
                          </Tag>
                        )}
                      </td>
                      <td className="min-w-[160px]">
                        <div className="flex items-center gap-3">
                          <div className="min-w-0 flex-1">
                            <Bar percent={p.progress} label={c.progress(p.progress)} tone={p.progress === 100 ? "green" : "ink"} />
                          </div>
                          <span className="tabular w-10 shrink-0 text-right text-small font-bold">{c.progress(p.progress)}</span>
                        </div>
                      </td>
                    </tr>
                  ))}
                  {shown.length === 0 && (
                    <tr>
                      <td colSpan={5} className="text-ink-2">
                        {c.noResults}
                      </td>
                    </tr>
                  )}
                </tbody>
              </table>
            </div>
            <div className="text-small text-ink-3">{c.count(shown.length)}</div>
            <AddSlot onClick={() => setOpen({ person: null })}>{c.add}</AddSlot>
          </>
        ))}

      {open && (
        <BeneficiaryWizard
          view={open.person}
          flavor={data.flavor}
          groups={data.groups}
          fields={data.custom_fields}
          onSaved={(_, change, isNew) => {
            apply(change);
            onNotice?.(isNew ? c.added : es.common.saved);
          }}
          onChange={apply}
          onClose={() => setOpen(null)}
        />
      )}
    </div>
  );
}
