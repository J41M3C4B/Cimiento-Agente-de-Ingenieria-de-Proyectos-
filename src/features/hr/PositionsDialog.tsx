import { useState } from "react";
import { Icon } from "../../components/icons";
import { QuarantineDialog } from "../../components/QuarantineDialog";
import { Alert, Button, IconButton, Inset, Modal, Select, Tag, TextArea, TextInput } from "../../components/ui";
import { es } from "../../i18n/es-MX";
import { toAppError } from "../../lib/tauri";
import type { Decision, QuarantineReport } from "../../lib/types";
import { hrPositionSave, hrPositionSetActive } from "./api";
import { modalityName } from "./labels";
import type { HrIssue, ModalityInfo, PositionInput, PositionRow, StaffChange } from "./types";

const h = es.hr;
const p = h.positionDialog;
const opts = (labels: Record<string, string>): [string, string][] => [["", h.none], ...Object.entries(labels)];
const numOrNull = (v: string) => (v.trim() === "" ? null : Number(v.replace(/[,\s$]/g, "")));

const EMPTY: PositionInput = { title: "", area: null, duties: null, default_modality: null, default_schedule: null, reference_pay_mxn: null, authorized_seats: null, reports_to: null };

/**
 * The window to add or edit one position. The title and the duties go through the scanner (they may reach the AI);
 * `onSaved` gets the whole change, so whoever opened it can pick the new position.
 */
export function PositionForm({
  position, positions, modalities, onSaved, onClose,
}: {
  position: PositionRow | null;
  positions: PositionRow[];
  modalities: ModalityInfo[];
  onSaved: (change: StaffChange, title: string) => void;
  onClose: () => void;
}) {
  const [v, setV] = useState<PositionInput>(position ? { ...position } : EMPTY);
  const [seats, setSeats] = useState(position?.authorized_seats?.toString() ?? "");
  const [pay, setPay] = useState(position?.reference_pay_mxn?.toString() ?? "");
  const [issues, setIssues] = useState<HrIssue[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [quarantine, setQuarantine] = useState<QuarantineReport | null>(null);
  const [busy, setBusy] = useState(false);
  const set = (patch: Partial<PositionInput>) => setV((x) => ({ ...x, ...patch }));
  const issue = (field: string) => issues.filter((i) => i.field === field).map((i) => h.issues[i.code] ?? i.code)[0];

  async function save(decision?: Decision) {
    setBusy(true);
    setError(null);
    try {
      const input = { ...v, reference_pay_mxn: numOrNull(pay), authorized_seats: numOrNull(seats) };
      const out = await hrPositionSave(position?.id ?? null, input, decision);
      if (out.status === "saved") onSaved(out.change, input.title.trim());
      else if (out.status === "invalid") setIssues(out.issues);
      else setQuarantine(out.report);
      if (out.status !== "quarantine") setQuarantine(null);
    } catch (e) {
      setError(toAppError(e).message);
    } finally {
      setBusy(false);
    }
  }

  return (
    <>
      <Modal
        title={position ? p.edit : p.add}
        onClose={onClose}
        footer={
          <>
            <Button onClick={onClose}>{es.common.cancel}</Button>
            <Button variant="primary" disabled={busy || v.title.trim() === ""} onClick={() => save()}>
              {busy ? es.common.saving : es.common.save}
            </Button>
          </>
        }
      >
        <TextInput label={p.fields.title} required autoFocus value={v.title} error={issue("title")} onChange={(e) => set({ title: e.target.value })} />
        <div className="grid grid-cols-1 gap-4 sm:grid-cols-2">
          <Select label={p.fields.area} options={opts(h.areas)} value={v.area ?? ""} onChange={(e) => set({ area: e.target.value || null })} />
          <Select
            label={p.fields.reports_to}
            options={[["", h.none], ...positions.filter((x) => x.id !== position?.id && x.active).map((x) => [x.id, x.title] as [string, string])]}
            value={v.reports_to ?? ""}
            onChange={(e) => set({ reports_to: e.target.value || null })}
          />
        </div>
        <TextArea label={p.fields.duties} hint={p.fields.dutiesHint} rows={3} value={v.duties ?? ""} onChange={(e) => set({ duties: e.target.value || null })} />
        <div className="grid grid-cols-1 gap-4 sm:grid-cols-2">
          <Select
            label={p.fields.default_modality}
            options={[["", h.none], ...modalities.map((m) => [m.code, modalityName(m)] as [string, string])]}
            value={v.default_modality ?? ""}
            onChange={(e) => set({ default_modality: e.target.value || null })}
          />
          <Select label={p.fields.default_schedule} options={opts(h.schedules)} value={v.default_schedule ?? ""} onChange={(e) => set({ default_schedule: e.target.value || null })} />
          <TextInput label={p.fields.reference_pay_mxn} prefix="$" inputMode="numeric" value={pay} error={issue("reference_pay_mxn")} onChange={(e) => setPay(e.target.value)} />
          <TextInput label={p.fields.authorized_seats} hint={p.fields.seatsHint} inputMode="numeric" value={seats} error={issue("authorized_seats")} onChange={(e) => setSeats(e.target.value)} />
        </div>
        {error && <Alert tone="error">{error}</Alert>}
      </Modal>
      {quarantine && (
        <QuarantineDialog report={quarantine} busy={busy} onRedact={() => save("redact")} onNotPersonal={() => save("not_personal")} onCancel={() => setQuarantine(null)} />
      )}
    </>
  );
}

/** The catalog of positions: how many people each one has against its seats, to add, edit or archive. */
export function PositionsDialog({
  positions, modalities, onChange, onClose,
}: {
  positions: PositionRow[];
  modalities: ModalityInfo[];
  onChange: (change: StaffChange) => void;
  onClose: () => void;
}) {
  const [editing, setEditing] = useState<PositionRow | "new" | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  async function setActive(id: string, active: boolean) {
    setBusy(true);
    setError(null);
    try {
      onChange(await hrPositionSetActive(id, active));
    } catch (e) {
      setError(toAppError(e).message);
    } finally {
      setBusy(false);
    }
  }

  const row = (x: PositionRow) => (
    <li key={x.id} className="flex items-center gap-3 py-3">
      <div className="min-w-0 flex-1">
        <b className="block truncate font-bold">{x.title}</b>
        <span className="block text-small text-ink-3">
          {[x.area ? h.areas[x.area] : null, x.authorized_seats ? p.seats(x.people, x.authorized_seats) : p.people(x.people)].filter(Boolean).join(" · ")}
        </span>
      </div>
      {x.authorized_seats !== null && x.authorized_seats > x.people && x.active && <Tag tone="amber">{p.vacancies(x.authorized_seats - x.people)}</Tag>}
      {x.active ? (
        <>
          <IconButton icon="pencil" label={p.edit} variant="plain" size="sm" onClick={() => setEditing(x)} />
          {x.people === 0 && (
            <Button size="sm" variant="plain" disabled={busy} onClick={() => setActive(x.id, false)}>
              {p.archive}
            </Button>
          )}
        </>
      ) : (
        <Button size="sm" variant="plain" disabled={busy} onClick={() => setActive(x.id, true)}>
          {p.restore}
        </Button>
      )}
    </li>
  );

  const active = positions.filter((x) => x.active);
  const archived = positions.filter((x) => !x.active);
  return (
    <>
      <Modal
        title={p.title}
        size="lg"
        onClose={onClose}
        footer={
          <>
            <Button onClick={onClose}>{es.common.close}</Button>
            <Button variant="primary" onClick={() => setEditing("new")}>
              <Icon name="plus" size={16} strokeWidth={2.4} />
              {p.add}
            </Button>
          </>
        }
      >
        <p className="text-ink-2">{p.intro}</p>
        {error && <Alert tone="error">{error}</Alert>}
        <Inset className="!px-4 !py-1">
          <ul className="divide-y divide-line">{active.map(row)}</ul>
        </Inset>
        {archived.length > 0 && (
          <div className="space-y-2">
            <h3 className="field-label">{p.archived}</h3>
            <Inset className="!px-4 !py-1">
              <ul className="divide-y divide-line">{archived.map(row)}</ul>
            </Inset>
          </div>
        )}
      </Modal>
      {editing && (
        <PositionForm
          position={editing === "new" ? null : editing}
          positions={positions}
          modalities={modalities}
          onSaved={(change) => {
            onChange(change);
            setEditing(null);
          }}
          onClose={() => setEditing(null)}
        />
      )}
    </>
  );
}
