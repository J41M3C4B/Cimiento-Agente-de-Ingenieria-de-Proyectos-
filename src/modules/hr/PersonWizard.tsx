import { useState } from "react";
import type { ReactNode } from "react";
import { Icon } from "../../components/icons";
import { Alert, Avatar, Bar, Button, Choice, Eyebrow, FormSection, Inset, MaskedField, Modal, Select, StepNav, Switch, TextButton, TextInput } from "../../components/ui";
import { es } from "../../i18n/es-MX";
import { IssueSummary } from "../../components/IssueSummary";
import { toAppError } from "../../lib/tauri";
import { hrModalityCreate, hrPersonDelete, hrPersonReveal, hrPersonSave } from "./api";
import { modalityName } from "./labels";
import { PositionForm } from "./PositionsDialog";
import type { CustomField, EmergencyContact, HrIssue, ModalityInfo, PersonData, PersonView, PositionRow, SecretField, StaffChange } from "./types";

const h = es.hr;
const STEPS = ["personal", "job", "emergency", "pay"] as const;
type Step = (typeof STEPS)[number];
const NEW = "__new";

export const emptyPerson = (): PersonData => ({
  first_names: "", last_name_1: null, last_name_2: null, birth_date: null, sex: null, curp: "", marital_status: null, nationality: "Mexicana",
  education: null, professional_license: null, address: { street: null, number: null, neighborhood: null, municipality: null, state: null, zip: null },
  phone: null, email: null, position_id: null, modality: "", start_date: null, end_date: null, schedule: null, shift: null, work_days: [],
  weekly_hours: null, status: "active", left_date: null, left_reason: null, emergency_contacts: [], pay_amount_mxn: null, pay_period: "monthly",
  pay_method: null, bank: null, clabe: "", rfc: "", nss: "", tax_regime: null, tax_zip: null, infonavit_credit: null, extra: {},
});

/** Which step a field of an issue belongs to, to take the person there. */
function stepOf(field: string): Step {
  if (field.startsWith("emergency_contacts")) return "emergency";
  if (["position_id", "modality", "start_date", "end_date", "schedule", "shift", "work_days", "weekly_hours", "status", "left_date", "left_reason"].includes(field)) return "job";
  if (["pay_amount_mxn", "pay_period", "pay_method", "bank", "clabe", "rfc", "nss", "tax_regime", "tax_zip"].includes(field)) return "pay";
  return "personal";
}

const opt = (labels: Record<string, string>): [string, string][] => [["", h.select], ...Object.entries(labels)];
const txt = (v: string) => (v.trim() === "" ? null : v);
const num = (v: string) => (v.trim() === "" ? null : Number(v.replace(/[,\s$]/g, "")));

/**
 * A covered identifier: stored, it shows covered with «Mostrar» (which is recorded) and «Cambiar»; new or being
 * changed, it is a plain field. What the record sends: `null` keeps what is stored, text replaces it, "" clears it.
 */
function SecretInput({
  field, label, hint, personId, stored, masked, value, error, onChange,
}: {
  field: SecretField; label: string; hint?: string; personId: string | null; stored: boolean; masked: string | null;
  value: string | null; error?: string; onChange: (v: string | null) => void;
}) {
  const [shown, setShown] = useState<string | null>(null);
  const [failure, setFailure] = useState<string | null>(null);
  if (stored && value === null) {
    return (
      <MaskedField
        label={label}
        value={shown ?? masked ?? h.secret.stored}
        hint={shown !== null ? h.secret.shownNote : hint}
        error={failure ?? undefined}
        actions={
          <>
            {shown === null && personId && (
              <Button
                size="sm"
                variant="plain"
                onClick={async () => {
                  try {
                    setFailure(null);
                    setShown(await hrPersonReveal(personId, field));
                  } catch (e) {
                    setFailure(toAppError(e).message);
                  }
                }}
              >
                {h.secret.show}
              </Button>
            )}
            <Button size="sm" variant="plain" onClick={() => onChange("")}>
              {h.secret.change}
            </Button>
          </>
        }
      />
    );
  }
  return (
    <div className="min-w-0">
      <TextInput label={label} hint={hint} value={value ?? ""} error={error} onChange={(e) => onChange(e.target.value.toUpperCase())} />
      {stored && (
        <TextButton className="mt-1" onClick={() => onChange(null)}>
          {h.secret.keep}
        </TextButton>
      )}
    </div>
  );
}

/** A small window to create an institution's own modality, which behaves as one of the built-in ones. */
function ModalityDialog({ modalities, onCreated, onClose }: { modalities: ModalityInfo[]; onCreated: (all: ModalityInfo[], code: string) => void; onClose: () => void }) {
  const [title, setTitle] = useState("");
  const [base, setBase] = useState("");
  const [error, setError] = useState<string | null>(null);
  const d = h.modalityDialog;
  async function create() {
    try {
      const all = await hrModalityCreate(title, base);
      const added = all.find((m) => !modalities.some((x) => x.code === m.code));
      onCreated(all, added?.code ?? "");
    } catch (e) {
      setError(toAppError(e).message);
    }
  }
  return (
    <Modal
      title={d.title}
      onClose={onClose}
      footer={
        <>
          <Button onClick={onClose}>{es.common.cancel}</Button>
          <Button variant="primary" disabled={!title.trim() || !base} onClick={create}>
            {d.create}
          </Button>
        </>
      }
    >
      <p className="text-ink-2">{d.intro}</p>
      <TextInput label={d.name} placeholder={d.namePlaceholder} autoFocus value={title} onChange={(e) => setTitle(e.target.value)} />
      <Select
        label={d.behavesAs}
        hint={base ? h.modalityNotes[base] : undefined}
        options={[["", h.select], ...modalities.filter((m) => m.builtin).map((m) => [m.code, modalityName(m)] as [string, string])]}
        value={base}
        onChange={(e) => setBase(e.target.value)}
      />
      {error && <Alert tone="error">{error}</Alert>}
    </Modal>
  );
}

/**
 * The record of one person in four steps. It saves at any step: the name, the position and the modality are enough,
 * and the record shows how far it has come so the rest fills in with time.
 */
export function PersonWizard({
  view, positions, modalities, fields, onSaved, onChange, onModalities, onClose,
}: {
  view: PersonView | null;
  positions: PositionRow[];
  modalities: ModalityInfo[];
  fields: CustomField[];
  onSaved: (person: PersonView, change: StaffChange, isNew: boolean) => void;
  onChange: (change: StaffChange) => void;
  onModalities: (all: ModalityInfo[]) => void;
  onClose: () => void;
}) {
  const [current, setCurrent] = useState<PersonView | null>(view);
  const [d, setD] = useState<PersonData>(view ? { ...view.data } : emptyPerson());
  const [step, setStep] = useState(0);
  const [issues, setIssues] = useState<HrIssue[]>(view?.issues ?? []);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [newPosition, setNewPosition] = useState(false);
  const [newModality, setNewModality] = useState(false);
  const [confirmRemove, setConfirmRemove] = useState(false);

  const set = (patch: Partial<PersonData>) => setD((x) => ({ ...x, ...patch }));
  const setAddress = (patch: Partial<PersonData["address"]>) => setD((x) => ({ ...x, address: { ...x.address, ...patch } }));
  const err = (field: string) => issues.filter((i) => i.field === field).map((i) => h.issues[i.code] ?? i.code)[0];
  const modality = modalities.find((m) => m.code === d.modality);
  const pays = modality?.rules.pay !== "none";
  const secretsStored = current?.secrets ?? { curp: false, rfc: false, nss: false, clabe: false };

  async function save(close: boolean) {
    setBusy(true);
    setError(null);
    try {
      const out = await hrPersonSave(current?.id ?? null, d);
      if (out.status === "invalid") {
        setIssues(out.issues);
        const first = out.issues[0];
        if (first) setStep(STEPS.indexOf(stepOf(first.field)));
        return;
      }
      setIssues(out.person.issues);
      onSaved(out.person, out.change, current === null);
      setCurrent(out.person);
      setD({ ...out.person.data });
      if (close) onClose();
    } catch (e) {
      setError(toAppError(e).message);
    } finally {
      setBusy(false);
    }
  }

  async function remove() {
    if (!current) return;
    setBusy(true);
    try {
      onChange(await hrPersonDelete(current.id));
      onClose();
    } catch (e) {
      setError(toAppError(e).message);
      setBusy(false);
    }
  }

  function choosePosition(id: string) {
    if (id === NEW) return setNewPosition(true);
    const p = positions.find((x) => x.id === id);
    set({
      position_id: id || null,
      // what the position usually is, as a suggestion for a new record
      modality: d.modality || p?.default_modality || "",
      schedule: d.schedule ?? p?.default_schedule ?? null,
    });
  }

  const contact = (i: number, patch: Partial<EmergencyContact>) =>
    set({ emergency_contacts: d.emergency_contacts.map((c, n) => (n === i ? { ...c, ...patch } : c)) });
  const progress = current?.progress;
  // each step shows how much of it is filled; one with nothing to fill (the volunteer has no pay) does not apply
  const stepItems = STEPS.map((s) => {
    const p = progress?.[s];
    const na = (s === "pay" && !pays) || (p !== undefined && p.total === 0);
    const warn = issues.filter((i) => stepOf(i.field) === s).length;
    return { key: s, label: h.steps[s], short: h.stepsShort[s], filled: p?.filled, total: p?.total, na, caption: p ? h.stepCaption(p.filled, p.total) : undefined, naLabel: h.stepNotApplicable, warn, warnLabel: warn > 0 ? es.common.review.title(warn) : undefined };
  });

  const grid = (children: ReactNode) => <div className="grid grid-cols-1 gap-4 sm:grid-cols-2">{children}</div>;

  return (
    <>
      <Modal
        title={current ? `${h.editing}: ${[current.data.first_names, current.data.last_name_1].filter(Boolean).join(" ")}` : h.add}
        size="wide"
        fixed
        onClose={onClose}
        footer={
          <>
            {current && (
              <Button variant="plain" className="mr-auto !text-red-ink" disabled={busy} onClick={() => setConfirmRemove(true)}>
                <Icon name="trash" size={16} />
                {h.removePerson}
              </Button>
            )}
            {step > 0 && <Button onClick={() => setStep(step - 1)}>{h.back}</Button>}
            <Button disabled={busy} onClick={() => save(false)}>
              {busy ? es.common.saving : h.save}
            </Button>
            {step < STEPS.length - 1 ? (
              <Button variant="primary" onClick={() => setStep(step + 1)}>
                {h.next}
              </Button>
            ) : (
              <Button variant="primary" disabled={busy} onClick={() => save(true)}>
                {busy ? es.common.saving : h.saveAndClose}
              </Button>
            )}
          </>
        }
      >
        <div className="flex items-center gap-4">
          <Avatar name={[d.first_names, d.last_name_1].filter(Boolean).join(" ")} />
          <div className="min-w-0 flex-1">
            <b className="block truncate font-bold">{[d.first_names, d.last_name_1, d.last_name_2].filter(Boolean).join(" ") || h.newRecord}</b>
            <span className="block truncate text-small text-ink-3">
              {[positions.find((p) => p.id === d.position_id)?.title, modality ? modalityName(modality) : null].filter(Boolean).join(" · ") || h.minimum}
            </span>
          </div>
          {progress && (
            <div className="hidden w-44 shrink-0 sm:block">
              <div className="mb-1 flex justify-between text-caption font-semibold text-ink-2">
                <span>{h.completeness}</span>
                <span className="tabular">{h.progress(progress.percent)}</span>
              </div>
              <Bar percent={progress.percent} label={h.completeness} tone={progress.percent === 100 ? "green" : "ink"} />
            </div>
          )}
        </div>
        <StepNav label={h.stepsLabel} steps={stepItems} current={step} onSelect={setStep} />
        <IssueSummary issues={issues} describe={(code) => h.issues[code] ?? code} stepOf={(field) => STEPS.indexOf(stepOf(field))} stepName={(n) => h.steps[STEPS[n]!]} onGo={setStep} />
        <div key={step} className="anim-rise space-y-6">
          <div className="flex items-baseline gap-3 border-b border-line pb-3">
            <Eyebrow>{h.stepOf(step + 1, STEPS.length)}</Eyebrow>
            <span className="text-ui text-ink-2">{h.stepIntro[STEPS[step]!]}</span>
          </div>
        {STEPS[step] === "personal" && (
          <>
            <FormSection title={h.sections.name} icon="user">
              {grid(
                <>
                  <TextInput label={h.fields.first_names} required autoFocus value={d.first_names} error={err("first_names")} onChange={(e) => set({ first_names: e.target.value })} />
                  <TextInput label={h.fields.last_name_1} value={d.last_name_1 ?? ""} onChange={(e) => set({ last_name_1: txt(e.target.value) })} />
                  <TextInput label={h.fields.last_name_2} value={d.last_name_2 ?? ""} onChange={(e) => set({ last_name_2: txt(e.target.value) })} />
                  <Select label={h.fields.sex} options={opt(h.sexes)} value={d.sex ?? ""} onChange={(e) => set({ sex: txt(e.target.value) })} />
                </>,
              )}
            </FormSection>
            <FormSection title={h.sections.identity} icon="idcard">
              {grid(
                <>
                  <TextInput label={h.fields.birth_date} type="date" value={d.birth_date ?? ""} error={err("birth_date")} onChange={(e) => set({ birth_date: txt(e.target.value) })} />
                  <SecretInput field="curp" label={h.fields.curp} hint={h.hints.curp} personId={current?.id ?? null} stored={secretsStored.curp} masked={current?.masked.curp ?? null} value={d.curp} error={err("curp")} onChange={(v) => set({ curp: v })} />
                  <Select label={h.fields.marital_status} options={opt(h.marital)} value={d.marital_status ?? ""} onChange={(e) => set({ marital_status: txt(e.target.value) })} />
                  <TextInput label={h.fields.nationality} value={d.nationality ?? ""} onChange={(e) => set({ nationality: txt(e.target.value) })} />
                </>,
              )}
            </FormSection>
            <FormSection title={h.sections.schooling} icon="file">
              {grid(
                <>
                  <Select label={h.fields.education} options={opt(h.education)} value={d.education ?? ""} onChange={(e) => set({ education: txt(e.target.value) })} />
                  <TextInput label={h.fields.professional_license} inputMode="numeric" value={d.professional_license ?? ""} error={err("professional_license")} onChange={(e) => set({ professional_license: txt(e.target.value) })} />
                </>,
              )}
            </FormSection>
            <FormSection title={h.sections.address} icon="building">
              {grid(
                <>
                  <TextInput label={h.fields.street} value={d.address.street ?? ""} onChange={(e) => setAddress({ street: txt(e.target.value) })} />
                  <TextInput label={h.fields.number} value={d.address.number ?? ""} onChange={(e) => setAddress({ number: txt(e.target.value) })} />
                  <TextInput label={h.fields.neighborhood} value={d.address.neighborhood ?? ""} onChange={(e) => setAddress({ neighborhood: txt(e.target.value) })} />
                  <TextInput label={h.fields.municipality} value={d.address.municipality ?? ""} onChange={(e) => setAddress({ municipality: txt(e.target.value) })} />
                  <TextInput label={h.fields.state} value={d.address.state ?? ""} onChange={(e) => setAddress({ state: txt(e.target.value) })} />
                  <TextInput label={h.fields.zip} inputMode="numeric" value={d.address.zip ?? ""} error={err("address.zip")} onChange={(e) => setAddress({ zip: txt(e.target.value) })} />
                </>,
              )}
            </FormSection>
            <FormSection title={h.sections.contact} icon="phone">
              {grid(
                <>
                  <TextInput label={h.fields.phone} inputMode="tel" value={d.phone ?? ""} error={err("phone")} onChange={(e) => set({ phone: txt(e.target.value) })} />
                  <TextInput label={h.fields.email} inputMode="email" value={d.email ?? ""} error={err("email")} onChange={(e) => set({ email: txt(e.target.value) })} />
                </>,
              )}
            </FormSection>
          </>
        )}

        {STEPS[step] === "job" && (
          <>
            <FormSection title={h.sections.position} icon="briefcase">
              {grid(
                <>
                  <Select
                    label={h.fields.position_id}
                    required
                    error={err("position_id")}
                    options={[["", h.select], ...positions.filter((p) => p.active || p.id === d.position_id).map((p) => [p.id, p.title] as [string, string]), [NEW, `＋ ${h.newPosition}`]]}
                    value={d.position_id ?? ""}
                    onChange={(e) => choosePosition(e.target.value)}
                  />
                  <Select
                    label={h.fields.modality}
                    required
                    hint={modality ? h.modalityNotes[modality.behaves_as] : h.hints.modality}
                    error={err("modality")}
                    options={[["", h.select], ...modalities.map((m) => [m.code, modalityName(m)] as [string, string]), [NEW, `＋ ${h.newModality}`]]}
                    value={d.modality}
                    onChange={(e) => (e.target.value === NEW ? setNewModality(true) : set({ modality: e.target.value }))}
                  />
                </>,
              )}
            </FormSection>
            <FormSection title={h.sections.dates} icon="calendar">
              {grid(
                <>
                  <TextInput label={h.fields.start_date} type="date" value={d.start_date ?? ""} error={err("start_date")} hint={current?.start_date_approx ? h.approxDate : undefined} onChange={(e) => set({ start_date: txt(e.target.value) })} />
                  {modality?.rules.needs_end_date && (
                    <TextInput label={h.fields.end_date} type="date" value={d.end_date ?? ""} error={err("end_date")} onChange={(e) => set({ end_date: txt(e.target.value) })} />
                  )}
                </>,
              )}
            </FormSection>
            <FormSection title={h.sections.time} icon="clock">
              {grid(
                <>
                  <Select label={h.fields.schedule} options={opt(h.schedules)} value={d.schedule ?? ""} onChange={(e) => set({ schedule: txt(e.target.value) })} />
                  <Select label={h.fields.shift} options={opt(h.shifts)} value={d.shift ?? ""} onChange={(e) => set({ shift: txt(e.target.value) })} />
                  <TextInput label={h.fields.weekly_hours} hint={h.hints.weekly_hours} inputMode="numeric" value={d.weekly_hours?.toString() ?? ""} error={err("weekly_hours")} onChange={(e) => set({ weekly_hours: num(e.target.value) })} />
                </>,
              )}
              <fieldset>
                <legend className="field-label">{h.fields.work_days}</legend>
                <div className="flex flex-wrap gap-2">
                  {Object.entries(h.weekdays).map(([code, label]) => (
                    <Choice
                      key={code}
                      type="checkbox"
                      checked={d.work_days.includes(code)}
                      onChange={(e) => set({ work_days: e.target.checked ? Object.keys(h.weekdays).filter((c) => c === code || d.work_days.includes(c)) : d.work_days.filter((c) => c !== code) })}
                    >
                      {label}
                    </Choice>
                  ))}
                </div>
              </fieldset>
            </FormSection>
            <FormSection title={h.sections.status} icon="info">
              {grid(
                <>
                  <Select label={h.fields.status} options={Object.entries(h.status)} value={d.status} onChange={(e) => set({ status: e.target.value })} />
                  {d.status === "left" && (
                    <>
                      <TextInput label={h.fields.left_date} type="date" value={d.left_date ?? ""} error={err("left_date")} onChange={(e) => set({ left_date: txt(e.target.value) })} />
                      <Select label={h.fields.left_reason} options={opt(h.leftReasons)} value={d.left_reason ?? ""} onChange={(e) => set({ left_reason: txt(e.target.value) })} />
                    </>
                  )}
                </>,
              )}
            </FormSection>
          </>
        )}

        {STEPS[step] === "emergency" && (
          <>
            {d.emergency_contacts.map((c, i) => (
              <FormSection key={i} title={h.sections.contactN(i + 1)} icon="phone">
                {grid(
                  <>
                    <TextInput label={h.fields.contact_name} required value={c.full_name} error={err(`emergency_contacts[${i}].full_name`)} onChange={(e) => contact(i, { full_name: e.target.value })} />
                    <Select label={h.fields.relationship} options={opt(h.relationships)} value={c.relationship ?? ""} onChange={(e) => contact(i, { relationship: txt(e.target.value) })} />
                    <TextInput label={h.fields.contact_phone} inputMode="tel" value={c.phone ?? ""} error={err(`emergency_contacts[${i}].phone`)} onChange={(e) => contact(i, { phone: txt(e.target.value) })} />
                    <TextInput label={h.fields.contact_phone_alt} inputMode="tel" value={c.phone_alt ?? ""} error={err(`emergency_contacts[${i}].phone_alt`)} onChange={(e) => contact(i, { phone_alt: txt(e.target.value) })} />
                  </>,
                )}
                <TextButton className="self-start" onClick={() => set({ emergency_contacts: d.emergency_contacts.filter((_, n) => n !== i) })}>
                  {h.removeContact}
                </TextButton>
              </FormSection>
            ))}
            {d.emergency_contacts.length < 2 && (
              <Button variant="secondary" onClick={() => set({ emergency_contacts: [...d.emergency_contacts, { full_name: "", relationship: null, phone: null, phone_alt: null }] })}>
                <Icon name="plus" size={16} strokeWidth={2.4} />
                {h.addContact}
              </Button>
            )}
          </>
        )}

        {STEPS[step] === "pay" &&
          (!pays ? (
            <Inset className="text-ui text-ink-2">{h.noPayStep}</Inset>
          ) : (
            <>
              <FormSection title={h.sections.payment} icon="banknote">
                {grid(
                  <>
                    <TextInput
                      label={h.fields.pay_amount_mxn}
                      prefix="$"
                      inputMode="numeric"
                      hint={modality?.rules.pay === "support" ? h.hints.payKind.support : modality?.rules.pay === "invoice" ? h.hints.payKind.invoice : h.hints.pay}
                      value={d.pay_amount_mxn?.toString() ?? ""}
                      error={err("pay_amount_mxn")}
                      onChange={(e) => set({ pay_amount_mxn: num(e.target.value) })}
                    />
                    <Select label={h.fields.pay_period} options={Object.entries(h.payPeriods)} value={d.pay_period ?? "monthly"} onChange={(e) => set({ pay_period: e.target.value })} />
                    <Select label={h.fields.pay_method} options={opt(h.payMethods)} value={d.pay_method ?? ""} onChange={(e) => set({ pay_method: txt(e.target.value) })} />
                  </>,
                )}
              </FormSection>
              {d.pay_method === "transfer" && (
                <FormSection title={h.sections.bank} icon="wallet">
                  {grid(
                    <>
                      <SecretInput field="clabe" label={h.fields.clabe} hint={h.hints.clabe} personId={current?.id ?? null} stored={secretsStored.clabe} masked={current?.masked.clabe ?? null} value={d.clabe} error={err("clabe")} onChange={(v) => set({ clabe: v })} />
                      <TextInput label={h.fields.bank} value={d.bank ?? ""} onChange={(e) => set({ bank: txt(e.target.value) })} />
                    </>,
                  )}
                </FormSection>
              )}
              {modality?.rules.tax_data !== false && (
                <FormSection title={h.sections.tax} icon="file">
                  {grid(
                    <>
                      <SecretInput field="rfc" label={h.fields.rfc} hint={h.hints.rfc} personId={current?.id ?? null} stored={secretsStored.rfc} masked={current?.masked.rfc ?? null} value={d.rfc} error={err("rfc")} onChange={(v) => set({ rfc: v })} />
                      {modality?.rules.imss !== false && (
                        <SecretInput field="nss" label={h.fields.nss} hint={h.hints.nss} personId={current?.id ?? null} stored={secretsStored.nss} masked={current?.masked.nss ?? null} value={d.nss} error={err("nss")} onChange={(v) => set({ nss: v })} />
                      )}
                      <Select label={h.fields.tax_regime} options={opt(h.taxRegimes)} value={d.tax_regime ?? ""} onChange={(e) => set({ tax_regime: txt(e.target.value) })} />
                      <TextInput label={h.fields.tax_zip} inputMode="numeric" value={d.tax_zip ?? ""} error={err("tax_zip")} onChange={(e) => set({ tax_zip: txt(e.target.value) })} />
                    </>,
                  )}
                  {modality?.rules.imss !== false && (
                    <Switch label={h.fields.infonavit_credit} checked={d.infonavit_credit === true} onChange={(e) => set({ infonavit_credit: e.target.checked })} />
                  )}
                </FormSection>
              )}
            </>
          ))}

        {STEPS[step] === "pay" && fields.length > 0 && (
          <FormSection title={h.sections.other} icon="sliders">
            {grid(
              fields.map((f) =>
                f.kind === "select" ? (
                  <Select key={f.key} label={f.title} options={[["", h.select], ...f.options.map((o) => [o, o] as [string, string])]} value={d.extra[f.key] ?? ""} onChange={(e) => set({ extra: { ...d.extra, [f.key]: e.target.value } })} />
                ) : (
                  <TextInput key={f.key} label={f.title} inputMode={f.kind === "number" ? "numeric" : undefined} value={d.extra[f.key] ?? ""} onChange={(e) => set({ extra: { ...d.extra, [f.key]: e.target.value } })} />
                ),
              ),
            )}
          </FormSection>
        )}

        </div>

        {error && <Alert tone="error">{error}</Alert>}
      </Modal>

      {newPosition && (
        <PositionForm
          position={null}
          positions={positions}
          modalities={modalities}
          onSaved={(change, title) => {
            onChange(change);
            const created = change.overview.positions.find((p) => p.title.toLowerCase() === title.toLowerCase());
            if (created) set({ position_id: created.id, modality: d.modality || created.default_modality || "" });
            setNewPosition(false);
          }}
          onClose={() => setNewPosition(false)}
        />
      )}
      {newModality && (
        <ModalityDialog
          modalities={modalities}
          onCreated={(all, code) => {
            onModalities(all);
            set({ modality: code });
            setNewModality(false);
          }}
          onClose={() => setNewModality(false)}
        />
      )}
      {confirmRemove && (
        <Modal
          title={h.removePerson}
          onClose={() => setConfirmRemove(false)}
          footer={
            <>
              <Button onClick={() => setConfirmRemove(false)}>{es.common.cancel}</Button>
              <Button variant="danger" disabled={busy} onClick={remove}>
                {h.removeYes}
              </Button>
            </>
          }
        >
          <p className="text-ink-2">{h.removeConfirm}</p>
        </Modal>
      )}
    </>
  );
}
