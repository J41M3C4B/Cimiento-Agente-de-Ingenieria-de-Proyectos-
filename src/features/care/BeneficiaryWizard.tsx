import { useState } from "react";
import type { ReactNode } from "react";
import { Icon } from "../../components/icons";
import { Alert, Button, Choice, FormSection, Inset, Modal, Select, Steps, Switch, TextButton, TextInput } from "../../components/ui";
import { es } from "../../i18n/es-MX";
import { toAppError } from "../../lib/tauri";
import { carePersonDelete, carePersonReveal, carePersonSave } from "./api";
import type { BeneficiaryData, BeneficiaryView, CareChange, CareField, CareFlavor, CareGroup, CareIssue, ResponsibleContact } from "./types";

const c = es.care;
const STEPS = ["identification", "stay", "care", "family", "contribution"] as const;
type Step = (typeof STEPS)[number];

export const emptyBeneficiary = (): BeneficiaryData => ({
  first_names: "", last_name_1: null, last_name_2: null, birth_date: null, birth_date_approx: false, approx_age: null, sex: null, curp: "",
  origin_municipality: null, origin_state: null, indigenous_language: null, education: null, literate: null, attends_school: null,
  school_grade: null, school_lag: null, group_id: null, entry_date: null, entry_date_approx: false, stay_mode: null, referred_by: null,
  admission_reasons: [], status: "active", status_date: null, discharge_reason: null, dependency: null, mobility: null, disabilities: [],
  chronic_conditions: [], continence: null, orientation: null, psych_care: null, vaccines_up_to_date: null, contacts: [], visits: null,
  legal_status: null, monthly_fee_mxn: null, fee_payer: null, programs: [], consent_date: null, consent_signer: null, extra: {},
});

function stepOf(field: string): Step {
  if (["entry_date", "stay_mode", "referred_by", "admission_reasons", "status", "status_date", "discharge_reason"].includes(field)) return "stay";
  if (["dependency", "mobility", "disabilities", "chronic_conditions", "continence", "orientation"].includes(field)) return "care";
  if (field.startsWith("contacts") || ["visits", "legal_status"].includes(field)) return "family";
  if (["monthly_fee_mxn", "fee_payer", "programs", "consent_date", "consent_signer"].includes(field)) return "contribution";
  return "identification";
}

const opt = (labels: Record<string, string>): [string, string][] => [["", c.select], ...Object.entries(labels)];
const txt = (v: string) => (v.trim() === "" ? null : v);
const num = (v: string) => (v.trim() === "" ? null : Number(v.replace(/[,\s$]/g, "")));
const yesNo = (v: boolean | null) => (v === null ? "" : v ? "yes" : "no");
const fromYesNo = (v: string) => (v === "" ? null : v === "yes");

/** Several choices from a list, as pills. */
function Many({ label, labels, value, onChange }: { label: string; labels: Record<string, string>; value: string[]; onChange: (v: string[]) => void }) {
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

/**
 * The record of a person served in five steps (ADR-029). It adapts to the kind of institution: the elderly home and
 * the children's home show their own data. It saves at any step.
 */
export function BeneficiaryWizard({
  view, flavor, groups, fields, onSaved, onChange, onClose,
}: {
  view: BeneficiaryView | null;
  flavor: CareFlavor;
  groups: CareGroup[];
  fields: CareField[];
  onSaved: (person: BeneficiaryView, change: CareChange, isNew: boolean) => void;
  onChange: (change: CareChange) => void;
  onClose: () => void;
}) {
  const [current, setCurrent] = useState<BeneficiaryView | null>(view);
  const [d, setD] = useState<BeneficiaryData>(view ? { ...view.data } : emptyBeneficiary());
  const [step, setStep] = useState(0);
  const [issues, setIssues] = useState<CareIssue[]>(view?.issues ?? []);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [noBirthDate, setNoBirthDate] = useState(!view);
  const [shownCurp, setShownCurp] = useState<string | null>(null);
  const [confirmRemove, setConfirmRemove] = useState(false);
  const elderly = flavor === "elderly_home";
  const children = flavor === "children_home";

  const set = (patch: Partial<BeneficiaryData>) => setD((x) => ({ ...x, ...patch }));
  const err = (field: string) => issues.filter((i) => i.field === field).map((i) => c.issues[i.code] ?? i.code)[0];
  const grid = (children: ReactNode) => <div className="grid grid-cols-1 gap-4 sm:grid-cols-2">{children}</div>;
  const contact = (i: number, patch: Partial<ResponsibleContact>) => set({ contacts: d.contacts.map((x, n) => (n === i ? { ...x, ...patch } : x)) });

  async function save(close: boolean) {
    setBusy(true);
    setError(null);
    try {
      const out = await carePersonSave(current?.id ?? null, { ...d, birth_date: noBirthDate ? null : d.birth_date, approx_age: noBirthDate ? d.approx_age : null });
      if (out.status === "invalid") {
        setIssues(out.issues);
        if (out.issues[0]) setStep(STEPS.indexOf(stepOf(out.issues[0].field)));
        return;
      }
      setIssues(out.person.issues);
      onSaved(out.person, out.change, current === null);
      setCurrent(out.person);
      setD({ ...out.person.data });
      setNoBirthDate(false);
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
      onChange(await carePersonDelete(current.id));
      onClose();
    } catch (e) {
      setError(toAppError(e).message);
      setBusy(false);
    }
  }

  const progress = current?.progress;
  const stepLabel = (s: Step) => {
    const p = progress?.[s];
    return p && p.total > 0 ? `${c.steps[s]} · ${p.filled}/${p.total}` : c.steps[s];
  };
  const blocking = issues.filter((i) => i.blocking);
  const headsUp = issues.filter((i) => !i.blocking);
  const curpStored = current?.curp_stored ?? false;

  return (
    <>
      <Modal
        title={current ? `${c.editing}: ${[current.data.first_names, current.data.last_name_1].filter(Boolean).join(" ")}` : c.add}
        size="lg"
        onClose={onClose}
        footer={
          <>
            {current && (
              <Button variant="plain" className="mr-auto" disabled={busy} onClick={() => setConfirmRemove(true)}>
                <Icon name="trash" size={16} />
                {c.removePerson}
              </Button>
            )}
            {step > 0 && <Button onClick={() => setStep(step - 1)}>{c.back}</Button>}
            {step < STEPS.length - 1 ? (
              <>
                <Button disabled={busy} onClick={() => save(false)}>
                  {busy ? es.common.saving : c.save}
                </Button>
                <Button variant="primary" onClick={() => setStep(step + 1)}>
                  {c.next}
                </Button>
              </>
            ) : (
              <Button variant="primary" disabled={busy} onClick={() => save(true)}>
                {busy ? es.common.saving : c.saveAndClose}
              </Button>
            )}
          </>
        }
      >
        <Steps steps={STEPS.map((s) => ({ key: s, label: stepLabel(s) }))} current={step} />
        {!current && step === 0 && <p className="text-ui text-ink-2">{c.minimum}</p>}
        {blocking.length > 0 && (
          <Alert tone="error">
            <ul className="list-disc pl-4">
              {blocking.map((i, n) => (
                <li key={n}>{c.issues[i.code] ?? i.code}</li>
              ))}
            </ul>
          </Alert>
        )}

        {STEPS[step] === "identification" && (
          <>
            <FormSection title={c.sections.name}>
              {grid(
                <>
                  <TextInput label={c.fields.first_names} required autoFocus value={d.first_names} error={err("first_names")} onChange={(e) => set({ first_names: e.target.value })} />
                  <TextInput label={c.fields.last_name_1} value={d.last_name_1 ?? ""} onChange={(e) => set({ last_name_1: txt(e.target.value) })} />
                  <TextInput label={c.fields.last_name_2} value={d.last_name_2 ?? ""} onChange={(e) => set({ last_name_2: txt(e.target.value) })} />
                  <Select label={c.fields.sex} options={opt(c.sexes)} value={d.sex ?? ""} onChange={(e) => set({ sex: txt(e.target.value) })} />
                </>,
              )}
            </FormSection>
            <FormSection title={c.sections.identity}>
              {grid(
                <>
                  {noBirthDate ? (
                    <TextInput label={c.fields.approx_age} required inputMode="numeric" value={d.approx_age?.toString() ?? ""} error={err("approx_age") ?? err("birth_date")} onChange={(e) => set({ approx_age: num(e.target.value) })} />
                  ) : (
                    <TextInput label={c.fields.birth_date} required type="date" value={d.birth_date ?? ""} hint={d.birth_date_approx ? c.approxBirth : undefined} error={err("birth_date")} onChange={(e) => set({ birth_date: txt(e.target.value) })} />
                  )}
                  <Switch label={c.unknownDate} className="self-end" checked={noBirthDate} onChange={(e) => setNoBirthDate(e.target.checked)} />
                  {curpStored && d.curp === null ? (
                    <div className="min-w-0">
                      <span className="field-label">{c.fields.curp}</span>
                      <div className="flex flex-wrap items-center gap-3">
                        <span className="tabular text-ui font-bold">{shownCurp ?? current?.curp_masked ?? c.secret.stored}</span>
                        {shownCurp === null && current && (
                          <TextButton onClick={async () => setShownCurp(await carePersonReveal(current.id))}>{c.secret.show}</TextButton>
                        )}
                        <TextButton onClick={() => set({ curp: "" })}>{c.secret.change}</TextButton>
                      </div>
                      {shownCurp !== null && <span className="field-hint">{c.secret.shownNote}</span>}
                    </div>
                  ) : (
                    <TextInput label={c.fields.curp} hint={c.hints.curp} value={d.curp ?? ""} error={err("curp")} onChange={(e) => set({ curp: e.target.value.toUpperCase() })} />
                  )}
                </>,
              )}
            </FormSection>
            <FormSection title={c.sections.origin}>
              {grid(
                <>
                  <TextInput label={c.fields.origin_municipality} value={d.origin_municipality ?? ""} onChange={(e) => set({ origin_municipality: txt(e.target.value) })} />
                  <TextInput label={c.fields.origin_state} value={d.origin_state ?? ""} onChange={(e) => set({ origin_state: txt(e.target.value) })} />
                  <TextInput label={c.fields.indigenous_language} value={d.indigenous_language ?? ""} onChange={(e) => set({ indigenous_language: txt(e.target.value) })} />
                </>,
              )}
            </FormSection>
            {elderly && (
              <FormSection title={c.sections.schooling}>
                {grid(
                  <>
                    <Select label={c.fields.education} options={opt(c.education)} value={d.education ?? ""} onChange={(e) => set({ education: txt(e.target.value) })} />
                    <Select label={c.fields.literate} options={[["", c.select], ["yes", c.yes], ["no", c.no]]} value={yesNo(d.literate)} onChange={(e) => set({ literate: fromYesNo(e.target.value) })} />
                  </>,
                )}
              </FormSection>
            )}
            {children && (
              <FormSection title={c.sections.school}>
                {grid(
                  <>
                    <Select label={c.fields.attends_school} options={[["", c.select], ["yes", c.yes], ["no", c.no]]} value={yesNo(d.attends_school)} onChange={(e) => set({ attends_school: fromYesNo(e.target.value) })} />
                    <Select label={c.fields.school_grade} options={opt(c.schoolGrades)} value={d.school_grade ?? ""} onChange={(e) => set({ school_grade: txt(e.target.value) })} />
                    <Select label={c.fields.school_lag} options={[["", c.select], ["yes", c.yes], ["no", c.no]]} value={yesNo(d.school_lag)} onChange={(e) => set({ school_lag: fromYesNo(e.target.value) })} />
                  </>,
                )}
              </FormSection>
            )}
            {groups.length > 0 && (
              <FormSection title={c.sections.group}>
                <Select
                  label={c.fields.group_id}
                  options={[["", c.fields.groupAuto(current?.group ?? "—")], ...groups.filter((g) => g.active || g.id === d.group_id).map((g) => [g.id, g.title] as [string, string])]}
                  value={d.group_id ?? ""}
                  onChange={(e) => set({ group_id: txt(e.target.value) })}
                />
              </FormSection>
            )}
          </>
        )}

        {STEPS[step] === "stay" && (
          <>
            <FormSection title={c.sections.entry}>
              {grid(
                <>
                  <TextInput label={c.fields.entry_date} type="date" value={d.entry_date ?? ""} hint={d.entry_date_approx ? c.approxEntry : undefined} error={err("entry_date")} onChange={(e) => set({ entry_date: txt(e.target.value) })} />
                  <Select label={c.fields.stay_mode} options={opt(c.stayModes)} value={d.stay_mode ?? ""} onChange={(e) => set({ stay_mode: txt(e.target.value) })} />
                  <Select label={c.fields.referred_by} options={opt(c.referredBy)} value={d.referred_by ?? ""} onChange={(e) => set({ referred_by: txt(e.target.value) })} />
                </>,
              )}
            </FormSection>
            <FormSection title={c.sections.reasons}>
              <Many label={c.fields.admission_reasons} labels={c.admissionReasons} value={d.admission_reasons} onChange={(v) => set({ admission_reasons: v })} />
            </FormSection>
            <FormSection title={c.sections.status}>
              {grid(
                <>
                  <Select label={c.fields.status} options={Object.entries(c.statuses)} value={d.status} onChange={(e) => set({ status: e.target.value })} />
                  {(d.status === "discharged" || d.status === "deceased") && (
                    <TextInput label={c.fields.status_date} type="date" value={d.status_date ?? ""} error={err("status_date")} onChange={(e) => set({ status_date: txt(e.target.value) })} />
                  )}
                  {d.status === "discharged" && (
                    <Select label={c.fields.discharge_reason} options={opt(c.dischargeReasons)} value={d.discharge_reason ?? ""} onChange={(e) => set({ discharge_reason: txt(e.target.value) })} />
                  )}
                </>,
              )}
            </FormSection>
          </>
        )}

        {STEPS[step] === "care" && (
          <>
            <Inset className="text-small text-ink-2">{c.healthNote}</Inset>
            <FormSection title={c.sections.support}>
              {grid(
                <>
                  <Select label={c.fields.dependency} options={opt(c.dependency)} value={d.dependency ?? ""} onChange={(e) => set({ dependency: txt(e.target.value) })} />
                  <Select label={c.fields.mobility} options={opt(c.mobility)} value={d.mobility ?? ""} onChange={(e) => set({ mobility: txt(e.target.value) })} />
                  {elderly && (
                    <>
                      <Select label={c.fields.continence} options={opt(c.continence)} value={d.continence ?? ""} onChange={(e) => set({ continence: txt(e.target.value) })} />
                      <Select label={c.fields.orientation} options={opt(c.orientation)} value={d.orientation ?? ""} onChange={(e) => set({ orientation: txt(e.target.value) })} />
                    </>
                  )}
                  {children && (
                    <>
                      <Select label={c.fields.vaccines_up_to_date} options={[["", c.select], ["yes", c.yes], ["no", c.no]]} value={yesNo(d.vaccines_up_to_date)} onChange={(e) => set({ vaccines_up_to_date: fromYesNo(e.target.value) })} />
                      <Select label={c.fields.psych_care} options={[["", c.select], ["yes", c.yes], ["no", c.no]]} value={yesNo(d.psych_care)} onChange={(e) => set({ psych_care: fromYesNo(e.target.value) })} />
                    </>
                  )}
                </>,
              )}
            </FormSection>
            <FormSection title={c.sections.health}>
              <Many label={c.fields.disabilities} labels={c.disabilities} value={d.disabilities} onChange={(v) => set({ disabilities: v })} />
              {!children && <Many label={c.fields.chronic_conditions} labels={c.chronic} value={d.chronic_conditions} onChange={(v) => set({ chronic_conditions: v })} />}
            </FormSection>
          </>
        )}

        {STEPS[step] === "family" && (
          <>
            {d.contacts.map((x, i) => (
              <FormSection key={i} title={c.sections.contactN(i + 1)}>
                {grid(
                  <>
                    <TextInput label={c.fields.contact_name} required value={x.full_name} error={err(`contacts[${i}].full_name`)} onChange={(e) => contact(i, { full_name: e.target.value })} />
                    <Select label={c.fields.relationship} options={opt(c.relationships)} value={x.relationship ?? ""} onChange={(e) => contact(i, { relationship: txt(e.target.value) })} />
                    <TextInput label={c.fields.phone} inputMode="tel" value={x.phone ?? ""} error={err(`contacts[${i}].phone`)} onChange={(e) => contact(i, { phone: txt(e.target.value) })} />
                    <TextInput label={c.fields.phone_alt} inputMode="tel" value={x.phone_alt ?? ""} error={err(`contacts[${i}].phone_alt`)} onChange={(e) => contact(i, { phone_alt: txt(e.target.value) })} />
                  </>,
                )}
                <Switch label={c.fields.legal_guardian} checked={x.legal_guardian} onChange={(e) => contact(i, { legal_guardian: e.target.checked })} />
                <TextButton onClick={() => set({ contacts: d.contacts.filter((_, n) => n !== i) })}>{c.removeContact}</TextButton>
              </FormSection>
            ))}
            {d.contacts.length < 2 && (
              <Button variant="secondary" onClick={() => set({ contacts: [...d.contacts, { full_name: "", relationship: null, phone: null, phone_alt: null, legal_guardian: false }] })}>
                <Icon name="plus" size={16} strokeWidth={2.4} />
                {c.addContact}
              </Button>
            )}
            <FormSection title={c.sections.visits}>
              <Select label={c.fields.visits} options={opt(c.visits)} value={d.visits ?? ""} onChange={(e) => set({ visits: txt(e.target.value) })} />
            </FormSection>
            {children && (
              <FormSection title={c.sections.legal}>
                <Select label={c.fields.legal_status} hint={c.hints.legal} options={opt(c.legal)} value={d.legal_status ?? ""} onChange={(e) => set({ legal_status: txt(e.target.value) })} />
              </FormSection>
            )}
          </>
        )}

        {STEPS[step] === "contribution" && (
          <>
            <FormSection title={c.sections.fee}>
              {grid(
                <>
                  <TextInput label={c.fields.monthly_fee_mxn} prefix="$" inputMode="numeric" hint={c.hints.fee} value={d.monthly_fee_mxn?.toString() ?? ""} error={err("monthly_fee_mxn")} onChange={(e) => set({ monthly_fee_mxn: num(e.target.value) })} />
                  <Select label={c.fields.fee_payer} options={opt(c.feePayers)} value={d.fee_payer ?? ""} onChange={(e) => set({ fee_payer: txt(e.target.value) })} />
                </>,
              )}
            </FormSection>
            <FormSection title={c.sections.programs}>
              <Many label={c.fields.programs} labels={c.programs} value={d.programs} onChange={(v) => set({ programs: v })} />
            </FormSection>
            <FormSection title={c.sections.consent}>
              <p className="text-small text-ink-2">{c.hints.consent}</p>
              {grid(
                <>
                  <TextInput label={c.fields.consent_date} type="date" value={d.consent_date ?? ""} error={err("consent_date")} onChange={(e) => set({ consent_date: txt(e.target.value) })} />
                  <Select label={c.fields.consent_signer} options={opt(c.consentSigners)} value={d.consent_signer ?? ""} onChange={(e) => set({ consent_signer: txt(e.target.value) })} />
                </>,
              )}
            </FormSection>
            {fields.length > 0 && (
              <FormSection title={c.sections.other}>
                {grid(
                  fields.map((f) =>
                    f.kind === "select" ? (
                      <Select key={f.key} label={f.title} options={[["", c.select], ...f.options.map((o) => [o, o] as [string, string])]} value={d.extra[f.key] ?? ""} onChange={(e) => set({ extra: { ...d.extra, [f.key]: e.target.value } })} />
                    ) : (
                      <TextInput key={f.key} label={f.title} inputMode={f.kind === "number" ? "numeric" : undefined} value={d.extra[f.key] ?? ""} onChange={(e) => set({ extra: { ...d.extra, [f.key]: e.target.value } })} />
                    ),
                  ),
                )}
              </FormSection>
            )}
          </>
        )}

        {headsUp.length > 0 && (
          <Alert tone="warn">
            <ul className="list-disc pl-4">
              {headsUp.map((i, n) => (
                <li key={n}>{c.issues[i.code] ?? i.code}</li>
              ))}
            </ul>
          </Alert>
        )}
        {error && <Alert tone="error">{error}</Alert>}
      </Modal>

      {confirmRemove && (
        <Modal
          title={c.removePerson}
          onClose={() => setConfirmRemove(false)}
          footer={
            <>
              <Button onClick={() => setConfirmRemove(false)}>{es.common.cancel}</Button>
              <Button variant="danger" disabled={busy} onClick={remove}>
                {c.removeYes}
              </Button>
            </>
          }
        >
          <p className="text-ink-2">{c.removeConfirm}</p>
        </Modal>
      )}
    </>
  );
}
