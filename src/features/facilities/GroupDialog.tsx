import { useState } from "react";
import { Alert, Button, Choice, FormSection, Modal, Select, TextArea, TextInput } from "../../components/ui";
import type { Tone } from "../../components/ui";
import { es } from "../../i18n/es-MX";
import type { EquipmentData, FacilityIssue, SpaceData, States } from "./types";

const f = es.facilities;
export const STATE_KEYS = ["good", "fair", "poor", "unusable"] as const;
export const STATE_TONE: Record<string, Tone> = { good: "green", fair: "amber", poor: "red", unusable: "red" };

const num = (v: string) => (v.trim() === "" ? null : Number(v.replace(/[,\s]/g, "")));
const yesNo = (v: boolean | null) => (v === null ? "" : v ? "yes" : "no");
const fromYesNo = (v: string) => (v === "" ? null : v === "yes");
const checked = (s: States) => s.good + s.fair + s.poor + s.unusable;

export const emptySpace = (kind = "bedroom"): SpaceData => ({
  kind, label: null, floor: 0, count: 1, good: 0, fair: 0, poor: 0, unusable: 0, problems: [], accessible: null, beds: null, hospital_beds: null,
  grab_bars: null, accessible_shower: null, notes: null,
});
export const emptyEquipment = (kind = "washer"): EquipmentData => ({ kind, label: null, count: 1, good: 0, fair: 0, poor: 0, unusable: 0, notes: null });

/** A yes / no / I do not know question, as three pills. */
export function YesNo({ label, value, onChange, name }: { label: string; value: boolean | null; onChange: (v: boolean | null) => void; name: string }) {
  return (
    <fieldset>
      <legend className="field-label">{label}</legend>
      <div className="flex flex-wrap gap-2">
        {[["yes", es.common.yes, "green"], ["no", es.common.no, "amber"], ["", f.unknown, "neutral"]].map(([v, text, tone]) => (
          <Choice key={v} name={name} tone={tone as Tone} checked={yesNo(value) === v} onChange={() => onChange(fromYesNo(v))}>
            {text}
          </Choice>
        ))}
      </div>
    </fieldset>
  );
}

/** Several choices from a list, as pills. */
export function Many({ label, labels, value, onChange }: { label: string; labels: Record<string, string>; value: string[]; onChange: (v: string[]) => void }) {
  return (
    <fieldset>
      <legend className="field-label">{label}</legend>
      <div className="flex flex-wrap gap-2">
        {Object.entries(labels).map(([code, text]) => (
          <Choice key={code} type="checkbox" checked={value.includes(code)} onChange={(e) => onChange(e.target.checked ? Object.keys(labels).filter((x) => x === code || value.includes(x)) : value.filter((x) => x !== code))}>
            {text}
          </Choice>
        ))}
      </div>
    </fieldset>
  );
}

/** How many are in each state, and how many are left without checking. */
function StatesField({ count, value, onChange, error }: { count: number; value: States; onChange: (s: States) => void; error?: string }) {
  const rest = count - checked(value);
  return (
    <FormSection title={f.spaces.stateTitle}>
      <p className="text-small text-ink-2">{f.spaces.stateHelp}</p>
      <div className="grid grid-cols-2 gap-3 sm:grid-cols-4">
        {STATE_KEYS.map((k) => (
          <TextInput
            key={k}
            label={f.states[k]}
            hint={f.statesHelp[k]}
            inputMode="numeric"
            value={value[k] === 0 ? "" : String(value[k])}
            placeholder="0"
            onChange={(e) => onChange({ ...value, [k]: num(e.target.value) ?? 0 })}
          />
        ))}
      </div>
      <div className="flex flex-wrap items-center gap-3">
        <Button size="sm" variant="plain" onClick={() => onChange({ good: count, fair: 0, poor: 0, unusable: 0 })}>
          {f.allGood}
        </Button>
        {rest > 0 && <span className="text-small text-ink-3">{f.unchecked(rest)}</span>}
      </div>
      {error && <Alert tone="error">{error}</Alert>}
    </FormSection>
  );
}

function issueText(issues: FacilityIssue[], field: string) {
  const i = issues.find((x) => x.field === field);
  return i ? (f.issues[i.code] ?? es.errors.generic) : undefined;
}

/** Adds or edits a group of spaces: kind, floor, how many and how they are, what fails and the data of its kind. */
export function SpaceDialog({
  start, isNew, kinds, floors, onSave, onClose,
}: {
  start: SpaceData;
  isNew: boolean;
  kinds: string[];
  floors: number | null;
  onSave: (d: SpaceData) => Promise<FacilityIssue[] | null>;
  onClose: () => void;
}) {
  const [d, setD] = useState<SpaceData>(start);
  const [issues, setIssues] = useState<FacilityIssue[]>([]);
  const [busy, setBusy] = useState(false);
  const set = (patch: Partial<SpaceData>) => setD((x) => ({ ...x, ...patch }));
  const top = Math.max((floors ?? 4) - 1, d.floor, 0);
  const floorOptions: [string, string][] = Array.from({ length: top + 2 }, (_, n) => n - 1).map((n) => [String(n), f.floors(n)]);
  const notGood = d.fair + d.poor + d.unusable > 0;

  async function save() {
    setBusy(true);
    const out = await onSave(d);
    setBusy(false);
    if (out) setIssues(out);
  }

  return (
    <Modal
      title={isNew ? f.spaces.add : f.spaces.edit}
      size="lg"
      onClose={onClose}
      footer={
        <>
          <Button onClick={onClose}>{es.common.cancel}</Button>
          <Button variant="primary" onClick={save} disabled={busy}>
            {busy ? es.common.saving : es.common.save}
          </Button>
        </>
      }
    >
      <FormSection title={f.spaces.kind}>
        <div className="grid grid-cols-1 gap-4 sm:grid-cols-2">
          <Select label={f.spaces.kind} hideLabel options={kinds.map((k) => [k, f.spaceKinds[k] ?? k])} value={d.kind} onChange={(e) => set({ kind: e.target.value })} error={issueText(issues, "kind")} />
          <TextInput
            label={d.kind === "other" ? f.spaces.labelRequired : f.spaces.label}
            hint={d.kind === "other" ? undefined : f.spaces.labelHint}
            required={d.kind === "other"}
            value={d.label ?? ""}
            onChange={(e) => set({ label: e.target.value || null })}
            error={issueText(issues, "label")}
          />
          <Select label={f.spaces.floor} options={floorOptions} value={String(d.floor)} onChange={(e) => set({ floor: Number(e.target.value) })} error={issueText(issues, "floor")} />
          <TextInput label={f.spaces.count} inputMode="numeric" value={String(d.count || "")} onChange={(e) => set({ count: num(e.target.value) ?? 0 })} error={issueText(issues, "count")} />
        </div>
      </FormSection>

      <StatesField count={d.count} value={d} onChange={(s) => set(s)} error={issueText(issues, "states")} />

      {(notGood || d.problems.length > 0) && <Many label={f.spaces.problemsTitle} labels={f.problems} value={d.problems} onChange={(problems) => set({ problems })} />}

      <FormSection title={f.spaces.accessible}>
        <YesNo name="accessible" label={f.spaces.accessible} value={d.accessible} onChange={(accessible) => set({ accessible })} />
        {(d.kind === "bedroom" || d.kind === "infirmary") && (
          <div className="grid grid-cols-1 gap-4 sm:grid-cols-2">
            <TextInput label={f.spaces.beds} inputMode="numeric" value={d.beds?.toString() ?? ""} onChange={(e) => set({ beds: num(e.target.value) })} error={issueText(issues, "beds")} />
            <TextInput label={f.spaces.hospitalBeds} inputMode="numeric" value={d.hospital_beds?.toString() ?? ""} onChange={(e) => set({ hospital_beds: num(e.target.value) })} error={issueText(issues, "hospital_beds")} />
          </div>
        )}
        {d.kind === "bathroom" && (
          <>
            <YesNo name="grab_bars" label={f.spaces.grabBars} value={d.grab_bars} onChange={(grab_bars) => set({ grab_bars })} />
            <YesNo name="accessible_shower" label={f.spaces.accessibleShower} value={d.accessible_shower} onChange={(accessible_shower) => set({ accessible_shower })} />
          </>
        )}
      </FormSection>

      <TextArea label={f.spaces.notes} hint={f.spaces.notesHint} rows={3} value={d.notes ?? ""} onChange={(e) => set({ notes: e.target.value || null })} />
      {issues.some((i) => !["kind", "label", "floor", "count", "states", "beds", "hospital_beds"].includes(i.field)) && (
        <Alert tone="error">{issues.map((i) => f.issues[i.code] ?? es.errors.generic).join(" ")}</Alert>
      )}
    </Modal>
  );
}

/** Adds or edits a group of equipment: kind, how many and how they are. */
export function EquipmentDialog({
  start, isNew, kinds, onSave, onClose,
}: {
  start: EquipmentData;
  isNew: boolean;
  kinds: string[];
  onSave: (d: EquipmentData) => Promise<FacilityIssue[] | null>;
  onClose: () => void;
}) {
  const [d, setD] = useState<EquipmentData>(start);
  const [issues, setIssues] = useState<FacilityIssue[]>([]);
  const [busy, setBusy] = useState(false);
  const set = (patch: Partial<EquipmentData>) => setD((x) => ({ ...x, ...patch }));

  async function save() {
    setBusy(true);
    const out = await onSave(d);
    setBusy(false);
    if (out) setIssues(out);
  }

  return (
    <Modal
      title={isNew ? f.equipment.add : f.equipment.edit}
      size="lg"
      onClose={onClose}
      footer={
        <>
          <Button onClick={onClose}>{es.common.cancel}</Button>
          <Button variant="primary" onClick={save} disabled={busy}>
            {busy ? es.common.saving : es.common.save}
          </Button>
        </>
      }
    >
      <div className="grid grid-cols-1 gap-4 sm:grid-cols-2">
        <Select label={f.equipment.kind} options={kinds.map((k) => [k, f.equipmentKinds[k] ?? k])} value={d.kind} onChange={(e) => set({ kind: e.target.value })} error={issueText(issues, "kind")} />
        <TextInput
          label={d.kind === "other" ? f.spaces.labelRequired : f.equipment.label}
          hint={d.kind === "other" ? undefined : f.equipment.labelHint}
          required={d.kind === "other"}
          value={d.label ?? ""}
          onChange={(e) => set({ label: e.target.value || null })}
          error={issueText(issues, "label")}
        />
        <TextInput label={f.equipment.count} inputMode="numeric" value={String(d.count || "")} onChange={(e) => set({ count: num(e.target.value) ?? 0 })} error={issueText(issues, "count")} />
      </div>
      <StatesField count={d.count} value={d} onChange={(s) => set(s)} error={issueText(issues, "states")} />
      <TextArea label={f.spaces.notes} hint={f.spaces.notesHint} rows={3} value={d.notes ?? ""} onChange={(e) => set({ notes: e.target.value || null })} />
    </Modal>
  );
}
