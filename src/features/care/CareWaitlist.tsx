import { useState } from "react";
import { Icon } from "../../components/icons";
import { Alert, Avatar, Button, IconButton, Inset, Modal, Select, Tag, TextInput, THead, Tile } from "../../components/ui";
import { es } from "../../i18n/es-MX";
import { toAppError } from "../../lib/tauri";
import { useSession } from "../access/session";
import { careWaitlistAdmit, careWaitlistDelete, careWaitlistSave } from "./api";
import type { BeneficiaryView, CareChange, WaitlistInput, WaitlistRow } from "./types";

const c = es.care;
const w = c.waitlist;
const day = (iso: string) => new Date(`${iso}T12:00:00`).toLocaleDateString("es-MX", { dateStyle: "medium" });
const daysSince = (iso: string) => Math.floor((Date.now() - new Date(`${iso}T12:00:00`).getTime()) / 86_400_000);

/** The requests to come in: since when, who and what they need. Taking one in opens its record; removing one is only for the administrator. */
export function CareWaitlist({ rows, freeSeats, onChange, onAdmitted }: { rows: WaitlistRow[]; freeSeats: number | null; onChange: (ch: CareChange) => void; onAdmitted: (p: BeneficiaryView, ch: CareChange) => void }) {
  const access = useSession();
  const empty: WaitlistInput = { requested_on: new Date().toISOString().slice(0, 10), name: null, phone: null, sex: null, approx_age: null, dependency: null, reason: null, status: "waiting" };
  const [editing, setEditing] = useState<{ id: string | null; v: WaitlistInput } | null>(null);
  const [asking, setAsking] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const waiting = rows.filter((r) => r.status === "waiting").sort((a, b) => a.requested_on.localeCompare(b.requested_on));

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
        <div className="min-w-[260px] flex-1 space-y-2">
          <p className="max-w-[70ch] text-ui text-ink-2">{w.help}</p>
          {waiting.length > 0 && (
            <div className="flex flex-wrap items-center gap-2">
              <Tag tone="amber" icon="clock">
                {w.count(waiting.length)}
              </Tag>
              {freeSeats !== null && (
                <Tag tone={freeSeats > 0 ? "green" : "neutral"} variant="soft">
                  {c.board.free(Math.max(0, freeSeats))}
                </Tag>
              )}
            </div>
          )}
        </div>
        <Button variant="primary" onClick={() => setEditing({ id: null, v: empty })}>
          <Icon name="plus" size={16} strokeWidth={2.4} />
          {w.add}
        </Button>
      </div>
      {error && <Alert tone="error">{error}</Alert>}
      {waiting.length === 0 ? (
        <Inset className="flex flex-col items-center gap-2 !px-6 !py-12 text-center text-body text-ink-2">{w.empty}</Inset>
      ) : (
        <div className="overflow-x-auto">
          <table className="table min-w-[760px]">
            <THead
              columns={[
                { key: "who", title: w.table.who },
                { key: "since", title: w.table.since },
                { key: "need", title: w.table.need },
                { key: "reason", title: w.table.reason },
                { key: "actions", title: "" },
              ]}
            />
            <tbody>
              {waiting.map((r) => {
                const days = daysSince(r.requested_on);
                return (
                  <tr key={r.id}>
                    <td className="min-w-[200px]">
                      <div className="flex min-w-0 items-center gap-3">
                        {r.name ? <Avatar size="sm" name={r.name} /> : <Tile small icon="user" tone="neutral" />}
                        <span className="min-w-0">
                          <span className={`block ${r.name ? "" : "text-ink-3"}`}>{r.name ?? w.noName}</span>
                          {r.phone && <span className="tabular block text-small text-ink-3">{r.phone}</span>}
                        </span>
                      </div>
                    </td>
                    <td className="min-w-[150px]">
                      <span className="block">{day(r.requested_on)}</span>
                      <span className={`block text-small ${days >= 90 ? "font-semibold text-amber-ink" : "text-ink-3"}`}>{w.waitingFor(days)}</span>
                    </td>
                    <td className="min-w-[160px]">
                      <span className="block">{[r.sex ? c.sexes[r.sex] : null, r.approx_age !== null ? c.years(r.approx_age) : null].filter(Boolean).join(" · ") || "—"}</span>
                      {r.dependency && <span className="block text-small text-ink-3">{c.dependency[r.dependency]}</span>}
                    </td>
                    <td>{r.reason ? <Tag variant="line">{c.admissionReasons[r.reason] ?? r.reason}</Tag> : "—"}</td>
                    <td>
                      <div className="flex items-center justify-end gap-1">
                        {asking === r.id ? (
                          <>
                            <span className="mr-1 text-small font-bold text-red-ink">{w.removeAsk}</span>
                            <Button size="sm" variant="danger" onClick={() => run(async () => { onChange(await careWaitlistDelete(r.id)); setAsking(null); })}>
                              {es.common.yes}
                            </Button>
                            <Button size="sm" variant="plain" onClick={() => setAsking(null)}>
                              {es.common.no}
                            </Button>
                          </>
                        ) : (
                          <>
                            <Button size="sm" variant="primary" onClick={() => run(async () => { const out = await careWaitlistAdmit(r.id); if (out.status === "saved") onAdmitted(out.person, out.change); })}>
                              {w.admit}
                            </Button>
                            <IconButton icon="pencil" label={w.edit} variant="plain" size="sm" onClick={() => setEditing({ id: r.id, v: { ...r } })} />
                            {access?.can("administer") && <IconButton icon="trash" label={w.remove} variant="plain" size="sm" onClick={() => setAsking(r.id)} />}
                          </>
                        )}
                      </div>
                    </td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        </div>
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
            {editing.id && <Select label={c.fields.status} options={Object.entries(w.statuses)} value={editing.v.status} onChange={(e) => set({ status: e.target.value })} />}
          </div>
        </Modal>
      )}
    </div>
  );
}
