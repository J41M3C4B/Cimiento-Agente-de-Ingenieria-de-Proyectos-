import { useState } from "react";
import type { ReactNode } from "react";
import { Icon } from "../../components/icons";
import { Alert, Avatar, Bar, Button, Choice, Eyebrow, FormSection, Inset, MaskedField, Modal, Select, StepNav, Switch, TextButton, TextInput } from "../../components/ui";
import { es } from "../../i18n/es-MX";
import { IssueSummary } from "../../components/IssueSummary";
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
  const [revealError, setRevealError] = useState<string | null>(null);
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
  const stepItems = STEPS.map((s) => {
    const p = progress?.[s];
    const warn = issues.filter((i) => stepOf(i.field) === s).length;
    return { key: s, label: c.steps[s], short: c.stepsShort[s], filled: p?.filled, total: p?.total, na: p !== undefined && p.total === 0, caption: p ? c.stepCaption(p.filled, p.total) : undefined, naLabel: c.stepNotApplicable, warn, warnLabel: warn > 0 ? es.common.review.title(warn) : undefined };
  });
  const fullName = [d.first_names, d.last_name_1, d.last_name_2].filter(Boolean).join(" ");
  const curpStored = current?.curp_stored ?? false;

  return (
    <>
      <Modal
        title={current ? `${c.editing}: ${[current.data.first_names, current.data.last_name_1].filter(Boolean).join(" ")}` : c.add}
        size="wide"
        fixed
        onClose={onClose}
        footer={
          <>
            {current && (
              <Button variant="plain" className="mr-auto !text-red-ink" disabled={busy} onClick={() => setConfirmRemove(true)}>
                <Icon name="trash" size={16} />
                {c.removePerson}
              </Button>
            )}
            {step > 0 && <Button onClick={() => setStep(step - 1)}>{c.back}</Button>}
            <Button disabled={busy} onClick={() => save(false)}>
              {busy ? es.common.saving : c.save}
            </Button>
            {step < STEPS.length - 1 ? (
              <Button variant="primary" onClick={() => setStep(step + 1)}>
                {c.next}
              </Button>
            ) : (
              <Button variant="primary" disabled={busy} onClick={() => save(true)}>
                {busy ? es.common.saving : c.saveAndClose}
              </Button>
            )}
          </>
        }
      >
        <div className="flex items-center gap-4">
          <Avatar name={fullName} />
          <div className="min-w-0 flex-1">
            <b className="block truncate font-bold">{fullName || c.newRecord}</b>
            <span className="block truncate text-small text-ink-3">{[current?.group, current?.age != null ? c.years(current.age) : null].filter(Boolean).join(" · ") || c.minimum}</span>
          </div>
          {progress && (
            <div className="hidden w-44 shrink-0 sm:block">
              <div className="mb-1 flex justify-between text-caption font-semibold text-ink-2">
                <span>{c.completeness}</span>
                <span className="tabular">{c.progress(progress.percent)}</span>
              </div>
              <Bar percent={progress.percent} label={c.completeness} tone={progress.percent === 100 ? "green" : "ink"} />
            </div>
          )}
        </div>
        <StepNav label={c.stepsLabel} steps={stepItems} current={step} onSelect={setStep} />
        <IssueSummary issues={issues} describe={(code) => c.issues[code] ?? code} stepOf={(field) => STEPS.indexOf(stepOf(field))} stepName={(n) => c.steps[STEPS[n]!]} onGo={setStep} />
        <div key={step} className="anim-rise space-y-6">
        <div className="flex items-baseline gap-3 border-b border-line pb-3">
          <Eyebrow>{c.stepOf(step + 1, STEPS.length)}</Eyebrow>
          <span className="text-ui text-ink-2">{c.stepIntro[STEPS[step]!]}</span>
        </div>
        {STEPS[step] === "identification" && (
          <>
            <FormSection title={c.sections.name} icon="user">
              {grid(
                <>
                  <TextInput label={c.fields.first_names} required autoFocus value={d.first_names} error={err("first_names")} onChange={(e) => set({ first_names: e.target.value })} />
                  <TextInput label={c.fields.last_name_1} value={d.last_name_1 ?? ""} onChange={(e) => set({ last_name_1: txt(e.target.value) })} />
                  <TextInput label={c.fields.last_name_2} value={d.last_name_2 ?? ""} onChange={(e) => set({ last_name_2: txt(e.target.value) })} />
                  <Select label={c.fields.sex} options={opt(c.sexes)} value={d.sex ?? ""} onChange={(e) => set({ sex: txt(e.target.value) })} />
                </>,
              )}
            </FormSection>
            <FormSection title={c.sections.identity} icon="idcard">
              {grid(
                <>
                  {noBirthDate ? (
                    <TextInput label={c.fields.approx_age} required inputMode="numeric" value={d.approx_age?.toString() ?? ""} error={err("approx_age") ?? err("birth_date")} onChange={(e) => set({ approx_age: num(e.target.value) })} />
                  ) : (
                    <TextInput label={c.fields.birth_date} required type="date" value={d.birth_date ?? ""} hint={d.birth_date_approx ? c.approxBirth : undefined} error={err("birth_date")} onChange={(e) => set({ birth_date: txt(e.target.value) })} />
                  )}
                  <Switch label={c.unknownDate} className="self-end" checked={noBirthDate} onChange={(e) => setNoBirthDate(e.target.checked)} />
                  {curpStored && d.curp === null ? (
                    <MaskedField
                      label={c.fields.curp}
                      value={shownCurp ?? current?.curp_masked ?? c.secret.stored}
                      hint={shownCurp !== null ? c.secret.shownNote : c.hints.curp}
                      error={revealError ?? undefined}
                      actions={
                        <>
                          {shownCurp === null && current && (
                            <Button
                              size="sm"
                              variant="plain"
                              onClick={async () => {
                                try {
                                  setRevealError(null);
                                  setShownCurp(await carePersonReveal(current.id));
                                } catch (e) {
                                  setRevealError(toAppError(e).message);
                                }
                              }}
                            >
                              {c.secret.show}
                            </Button>
                          )}
                          <Button size="sm" variant="plain" onClick={() => set({ curp: "" })}>
                            {c.secret.change}
                          </Button>
                        </>
                      }
                    />
                  ) : (
                    <TextInput label={c.fields.curp} hint={c.hints.curp} value={d.curp ?? ""} error={err("curp")} onChange={(e) => set({ curp: e.target.value.toUpperCase() })} />
                  )}
                </>,
              )}
            </FormSection>
            <FormSection title={c.sections.origin} icon="home">
              {grid(
                <>
                  <TextInput label={c.fields.origin_municipality} value={d.origin_municipality ?? ""} onChange={(e) => set({ origin_municipality: txt(e.target.value) })} />
                  <TextInput label={c.fields.origin_state} value={d.origin_state ?? ""} onChange={(e) => set({ origin_state: txt(e.target.value) })} />
                  <TextInput label={c.fields.indigenous_language} value={d.indigenous_language ?? ""} onChange={(e) => set({ indigenous_language: txt(e.target.value) })} />
                </>,
              )}
            </FormSection>
            {elderly && (
              <FormSection title={c.sections.schooling} icon="file">
                {grid(
                  <>
                    <Select label={c.fields.education} options={opt(c.education)} value={d.education ?? ""} onChange={(e) => set({ education: txt(e.target.value) })} />
                    <Select label={c.fields.literate} options={[["", c.select], ["yes", c.yes], ["no", c.no]]} value={yesNo(d.literate)} onChange={(e) => set({ literate: fromYesNo(e.target.value) })} />
                  </>,
                )}
              </FormSection>
            )}
            {children && (
              <FormSection title={c.sections.school} icon="file">
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
              <FormSection title={c.sections.group} icon="users">
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
            <FormSection title={c.sections.entry} icon="calendar">
              {grid(
                <>
                  <TextInput label={c.fields.entry_date} type="date" value={d.entry_date ?? ""} hint={d.entry_date_approx ? c.approxEntry : undefined} error={err("entry_date")} onChange={(e) => set({ entry_date: txt(e.target.value) })} />
                  <Select label={c.fields.stay_mode} options={opt(c.stayModes)} value={d.stay_mode ?? ""} onChange={(e) => set({ stay_mode: txt(e.target.value) })} />
                  <Select label={c.fields.referred_by} options={opt(c.referredBy)} value={d.referred_by ?? ""} onChange={(e) => set({ referred_by: txt(e.target.value) })} />
                </>,
              )}
            </FormSection>
            <FormSection title={c.sections.reasons} icon="info">
              <Many label={c.fields.admission_reasons} labels={c.admissionReasons} value={d.admission_reasons} onChange={(v) => set({ admission_reasons: v })} />
            </FormSection>
            <FormSection title={c.sections.status} icon="clock">
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
            <FormSection title={c.sections.support} icon="heart">
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
            <FormSection title={c.sections.health} icon="shield">
              <Many label={c.fields.disabilities} labels={c.disabilities} value={d.disabilities} onChange={(v) => set({ disabilities: v })} />
              {!children && <Many label={c.fields.chronic_conditions} labels={c.chronic} value={d.chronic_conditions} onChange={(v) => set({ chronic_conditions: v })} />}
            </FormSection>
          </>
        )}

        {STEPS[step] === "family" && (
          <>
            {d.contacts.map((x, i) => (
              <FormSection key={i} title={c.sections.contactN(i + 1)} icon="phone">
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
            <FormSection title={c.sections.visits} icon="calendar">
              <Select label={c.fields.visits} options={opt(c.visits)} value={d.visits ?? ""} onChange={(e) => set({ visits: txt(e.target.value) })} />
            </FormSection>
            {children && (
              <FormSection title={c.sections.legal} icon="shield">
                <Select label={c.fields.legal_status} hint={c.hints.legal} options={opt(c.legal)} value={d.legal_status ?? ""} onChange={(e) => set({ legal_status: txt(e.target.value) })} />
              </FormSection>
            )}
          </>
        )}

        {STEPS[step] === "contribution" && (
          <>
            <FormSection title={c.sections.fee} icon="wallet">
              {grid(
                <>
                  <TextInput label={c.fields.monthly_fee_mxn} prefix="$" inputMode="numeric" hint={c.hints.fee} value={d.monthly_fee_mxn?.toString() ?? ""} error={err("monthly_fee_mxn")} onChange={(e) => set({ monthly_fee_mxn: num(e.target.value) })} />
                  <Select label={c.fields.fee_payer} options={opt(c.feePayers)} value={d.fee_payer ?? ""} onChange={(e) => set({ fee_payer: txt(e.target.value) })} />
                </>,
              )}
            </FormSection>
            <FormSection title={c.sections.programs} icon="building">
              <Many label={c.fields.programs} labels={c.programs} value={d.programs} onChange={(v) => set({ programs: v })} />
            </FormSection>
            <FormSection title={c.sections.consent} icon="lock">
              <p className="text-small text-ink-2">{c.hints.consent}</p>
              {grid(
                <>
                  <TextInput label={c.fields.consent_date} type="date" value={d.consent_date ?? ""} error={err("consent_date")} onChange={(e) => set({ consent_date: txt(e.target.value) })} />
                  <Select label={c.fields.consent_signer} options={opt(c.consentSigners)} value={d.consent_signer ?? ""} onChange={(e) => set({ consent_signer: txt(e.target.value) })} />
                </>,
              )}
            </FormSection>
            {fields.length > 0 && (
              <FormSection title={c.sections.other} icon="file">
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

        {error && <Alert tone="error">{error}</Alert>}
        </div>
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
