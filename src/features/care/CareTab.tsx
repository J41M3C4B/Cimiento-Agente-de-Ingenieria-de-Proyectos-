import { useQuery, useQueryClient } from "@tanstack/react-query";
import { useMemo, useState } from "react";
import { Icon } from "../../components/icons";
import { AddSlot, Alert, Avatar, Bar, Button, Figures, Inset, Modal, Search, Segmented, Select, StatusDot, Tag, TextInput, THead } from "../../components/ui";
import { es } from "../../i18n/es-MX";
import { toAppError } from "../../lib/tauri";
import type { ProfileView } from "../../lib/types";
import { useSession } from "../access/session";
import { careOverview, carePersonGet, careWaitlistAdmit, careWaitlistDelete, careWaitlistSave } from "./api";
import { BeneficiaryWizard } from "./BeneficiaryWizard";
import type { Board, BeneficiaryView, CareChange, CareOverview, Count, WaitlistInput, WaitlistRow } from "./types";

const c = es.care;
export const CARE_KEY = ["care"] as const;
type View = "people" | "board" | "waitlist";
const peso = (n: number) => `$${n.toLocaleString("es-MX")}`;
const day = (iso: string) => new Date(`${iso}T12:00:00`).toLocaleDateString("es-MX", { dateStyle: "medium" });

/** Counts as short bars: «Silla de ruedas — 7». */
function Bars({ title, list, labels, total }: { title: string; list: Count[]; labels: Record<string, string>; total: number }) {
  if (list.length === 0) return null;
  return (
    <Inset className="space-y-3">
      <h3 className="text-ui font-bold">{title}</h3>
      {list.map((x) => (
        <Bar key={x.code} percent={total > 0 ? (x.count / total) * 100 : 0} label={`${labels[x.code] ?? x.code} — ${x.count}`} />
      ))}
    </Inset>
  );
}

function BoardView({ board, flavor }: { board: Board; flavor: CareOverview["flavor"] }) {
  const b = c.board;
  const i = board.indicators;
  const figures = [
    { label: b.served, value: String(i.served), sub: board.occupancy_percent !== null ? b.occupancy(board.occupancy_percent) : undefined, fill: board.occupancy_percent ?? undefined, tone: "violet" as const },
    { label: c.tabs.waitlist, value: String(board.waiting), sub: board.free_seats !== null ? b.free(board.free_seats) : undefined, tone: "amber" as const },
    { label: b.cost, value: board.cost_per_person_monthly !== null ? peso(board.cost_per_person_monthly) : "—", sub: i.average_fee !== null ? `${b.averageFee}: ${peso(i.average_fee)}` : undefined },
    { label: b.averageAge(Math.round(i.average_age ?? 0)), value: i.average_years !== null ? b.years(Math.round(i.average_years)) : "—", sub: b.thisYear(i.admitted_this_year, i.discharged_this_year, i.deceased_this_year) },
  ];
  return (
    <div className="space-y-4">
      <Figures items={figures} />
      <Inset className="space-y-3">
        <div>
          <h3 className="text-heading font-bold">{b.findings}</h3>
          <p className="text-small text-ink-2">{b.findingsHelp}</p>
        </div>
        {board.insights.length === 0 ? (
          <p className="text-ui text-ink-3">{b.noFindings}</p>
        ) : (
          <ul className="space-y-2">
            {board.insights.map((x) => (
              <li key={x.code} className="flex flex-wrap items-start gap-3">
                <Icon name={x.for_ai ? "sparkles" : "info"} size={18} className="mt-0.5 shrink-0 text-ink-2" />
                <span className="min-w-0 flex-1 text-ui">{c.insights[x.code]?.(x.values, x.items) ?? x.code}</span>
                <Tag tone={x.for_ai ? "violet" : "neutral"}>{x.for_ai ? b.forAi : b.internal}</Tag>
              </li>
            ))}
          </ul>
        )}
      </Inset>
      <div className="grid gap-4 md:grid-cols-2">
        <Bars title={b.pyramid} list={i.pyramid.map(([band, sex, n]) => ({ code: `${band}|${sex}`, count: n }))} labels={Object.fromEntries(i.pyramid.map(([band, sex]) => [`${band}|${sex}`, `${c.sexes[sex] ?? sex}, ${c.bands[band] ?? band}`]))} total={i.served} />
        <Bars title={b.sections.dependency} list={i.dependency} labels={c.dependency} total={i.served} />
        <Bars title={b.sections.mobility} list={i.mobility} labels={c.mobility} total={i.served} />
        <Bars title={b.sections.disabilities} list={i.disabilities} labels={c.disabilities} total={i.served} />
        {flavor !== "children_home" && <Bars title={b.sections.chronic} list={i.chronic} labels={c.chronic} total={i.served} />}
        <Bars title={b.sections.admission_reasons} list={i.admission_reasons} labels={c.admissionReasons} total={i.served} />
        <Bars title={b.sections.programs} list={i.programs} labels={c.programs} total={i.served} />
        {flavor === "children_home" && <Bars title={b.sections.legal} list={i.legal} labels={c.legal} total={i.served} />}
      </div>
      {i.incomplete > 0 && <Alert tone="info">{b.incomplete(i.incomplete)}</Alert>}
    </div>
  );
}

function WaitlistView({ rows, onChange, onAdmitted }: { rows: WaitlistRow[]; onChange: (ch: CareChange) => void; onAdmitted: (p: BeneficiaryView, ch: CareChange) => void }) {
  const w = c.waitlist;
  const access = useSession();
  const empty: WaitlistInput = { requested_on: new Date().toISOString().slice(0, 10), name: null, phone: null, sex: null, approx_age: null, dependency: null, reason: null, status: "waiting" };
  const [editing, setEditing] = useState<{ id: string | null; v: WaitlistInput } | null>(null);
  const [error, setError] = useState<string | null>(null);
  const waiting = rows.filter((r) => r.status === "waiting");

  async function run(job: () => Promise<void>) {
    setError(null);
    try {
      await job();
    } catch (e) {
      setError(toAppError(e).message);
    }
  }
  const save = () =>
    run(async () => {
      if (!editing) return;
      const out = await careWaitlistSave(editing.id, editing.v);
      if (out.status === "invalid") return setError(c.issues[out.issues[0]?.code ?? ""] ?? es.errors.generic);
      onChange(out.change);
      setEditing(null);
    });
  const set = (patch: Partial<WaitlistInput>) => setEditing((e) => e && { ...e, v: { ...e.v, ...patch } });

  return (
    <div className="space-y-4">
      <div className="flex flex-wrap items-center justify-between gap-3">
        <p className="max-w-[70ch] text-ui text-ink-2">{w.help}</p>
        <Button variant="primary" onClick={() => setEditing({ id: null, v: empty })}>
          <Icon name="plus" size={16} strokeWidth={2.4} />
          {w.add}
        </Button>
      </div>
      {error && <Alert tone="error">{error}</Alert>}
      {waiting.length === 0 ? (
        <Inset className="text-ui text-ink-3">{w.empty}</Inset>
      ) : (
        <ul className="divide-y divide-line">
          {waiting.map((r) => (
            <li key={r.id} className="flex flex-wrap items-center gap-3 py-3">
              <div className="min-w-[220px] flex-1">
                <b className="block font-bold">{r.name ?? w.noName}</b>
                <span className="block text-small text-ink-3">
                  {[w.since(day(r.requested_on)), r.sex ? c.sexes[r.sex] : null, r.approx_age !== null ? c.years(r.approx_age) : null, r.dependency ? c.dependency[r.dependency] : null, r.reason ? c.admissionReasons[r.reason] : null]
                    .filter(Boolean)
                    .join(" · ")}
                </span>
              </div>
              <Button size="sm" onClick={() => setEditing({ id: r.id, v: { ...r } })}>
                {w.edit}
              </Button>
              <Button size="sm" variant="primary" onClick={() => run(async () => { const out = await careWaitlistAdmit(r.id); if (out.status === "saved") onAdmitted(out.person, out.change); })}>
                {w.admit}
              </Button>
              {access?.can("delete") && (
                <Button size="sm" variant="plain" onClick={() => run(async () => onChange(await careWaitlistDelete(r.id)))}>
                  {w.remove}
                </Button>
              )}
            </li>
          ))}
        </ul>
      )}
      {editing && (
        <Modal
          title={editing.id ? w.edit : w.add}
          onClose={() => setEditing(null)}
          footer={
            <>
              <Button onClick={() => setEditing(null)}>{es.common.cancel}</Button>
              <Button variant="primary" onClick={save}>
                {es.common.save}
              </Button>
            </>
          }
        >
          <div className="grid grid-cols-1 gap-4 sm:grid-cols-2">
            <TextInput label={w.requested_on} type="date" value={editing.v.requested_on} onChange={(e) => set({ requested_on: e.target.value })} />
            <TextInput label={w.name} value={editing.v.name ?? ""} onChange={(e) => set({ name: e.target.value || null })} />
            <TextInput label={w.phone} inputMode="tel" value={editing.v.phone ?? ""} onChange={(e) => set({ phone: e.target.value || null })} />
            <Select label={c.fields.sex} options={[["", c.select], ...Object.entries(c.sexes)]} value={editing.v.sex ?? ""} onChange={(e) => set({ sex: e.target.value || null })} />
            <TextInput label={c.fields.approx_age} inputMode="numeric" value={editing.v.approx_age?.toString() ?? ""} onChange={(e) => set({ approx_age: e.target.value ? Number(e.target.value) : null })} />
            <Select label={c.fields.dependency} options={[["", c.select], ...Object.entries(c.dependency)]} value={editing.v.dependency ?? ""} onChange={(e) => set({ dependency: e.target.value || null })} />
            <Select label={w.reason} options={[["", c.select], ...Object.entries(c.admissionReasons)]} value={editing.v.reason ?? ""} onChange={(e) => set({ reason: e.target.value || null })} />
            {editing.id && (
              <Select label={c.fields.status} options={Object.entries(w.statuses)} value={editing.v.status} onChange={(e) => set({ status: e.target.value })} />
            )}
          </div>
        </Modal>
      )}
    </div>
  );
}

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

      {view === "board" && <BoardView board={data.board} flavor={data.flavor} />}
      {view === "waitlist" && <WaitlistView rows={data.waitlist} onChange={apply} onAdmitted={(p, ch) => { apply(ch); setOpen({ person: p }); }} />}
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
                            {p.heads_up > 0 && <span className="block text-small text-ink-3">{c.headsUp(p.heads_up)}</span>}
                          </span>
                        </button>
                      </td>
                      <td>{p.group}</td>
                      <td className="tabular">{p.age !== null ? c.years(p.age) : "—"}</td>
                      <td>
                        <StatusDot tone={p.status === "active" ? "green" : p.status === "hospitalized" ? "amber" : "neutral"}>{c.statuses[p.status] ?? p.status}</StatusDot>
                      </td>
                      <td className="min-w-[140px]">
                        <Bar percent={p.progress} label={`${p.progress} %`} tone={p.progress === 100 ? "green" : "ink"} />
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
