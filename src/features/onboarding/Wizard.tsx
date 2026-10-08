import { useState } from "react";
import { Icon } from "../../components/icons";
import { Alert, Button, Facts, FormSection, RadioCard, Segmented, Select, StepNav, Tag, TextArea, TextButton, TextInput } from "../../components/ui";
import { QuarantineDialog } from "../../components/QuarantineDialog";
import { es } from "../../i18n/es-MX";
import { toAppError } from "../../lib/tauri";
import type { Decision, IncomeKind, InstitutionKind, Period } from "../../lib/types";
import { YesNo } from "../facilities/GroupDialog";
import { INCOME_KINDS } from "../profile/finance";
import { parsePesos } from "../profile/profileForm";
import { onboardingFinish, onboardingSave } from "./api";
import type { OnboardingData, OnboardingIssue, OnboardingStatus } from "./api";
import type { QuarantineReport } from "../../lib/types";
import { StartFrame } from "./Welcome";

const o = es.onboarding;
const ins = es.institution;
const p = es.profile;
const REVIEW = "review";

const num = (v: string) => (v.trim() === "" || !/^\d+$/.test(v.trim()) ? null : Number(v.trim()));
const options = (labels: Record<string, string>): [string, string][] => [["", es.facilities.select], ...Object.entries(labels)];
const txt = (v: string) => (v === "" ? null : v);

/** A short answer (Sí / En trámite / No…) as one plain group. */
function Pills({ label, labels, value, onChange }: { label: string; labels: Record<string, string>; value: string | null; onChange: (v: string) => void; name?: string }) {
  return (
    <fieldset>
      <legend className="field-label">
        {label}
        <span className="req">*</span>
      </legend>
      <Segmented label={label} value={value ?? ""} onChange={onChange} items={Object.entries(labels).map(([id, text]) => ({ id, label: text }))} />
    </fieldset>
  );
}

/**
 * The data of the institution, step by step (ADR-031). Every «Guardar y seguir» saves what there is; it goes on
 * only when Rust says the step is complete, and names what is missing otherwise. The last screen reviews it all and
 * closes it.
 */
export function Wizard({ status, onStatus, onFinished, onBack }: { status: OnboardingStatus; onStatus: (s: OnboardingStatus) => void; onFinished: () => void; onBack?: () => void }) {
  const keys = [...status.steps.map((s) => s.key), REVIEW];
  const firstOpen = status.steps.findIndex((s) => !s.complete);
  const [n, setN] = useState(firstOpen === -1 ? keys.length - 1 : firstOpen);
  const [d, setD] = useState<OnboardingData>(status.data);
  const [issues, setIssues] = useState<OnboardingIssue[]>([]);
  const [shownMissing, setShownMissing] = useState<string[]>([]);
  const [quarantine, setQuarantine] = useState<QuarantineReport | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const key = keys[n]!;

  const set = (patch: Partial<OnboardingData>) => setD((x) => ({ ...x, ...patch }));
  const setInst = (patch: Partial<OnboardingData["institution"]>) => setD((x) => ({ ...x, institution: { ...x.institution, ...patch } }));
  const issue = (field: string) => {
    const i = issues.find((x) => x.field === field);
    return i ? (es.issues[i.code] ?? es.facilities.issues[i.code] ?? es.errors.generic) : undefined;
  };

  async function save(decision?: Decision) {
    setBusy(true);
    setError(null);
    setIssues([]);
    try {
      const out = await onboardingSave(d, decision);
      if (out.status === "quarantine") return setQuarantine(out.report);
      setQuarantine(null);
      if (out.status === "invalid") return setIssues(out.issues);
      onStatus(out.onboarding);
      setD(out.onboarding.data);
      const now = out.onboarding.steps.find((s) => s.key === key);
      if (now && !now.complete) return setShownMissing(now.missing);
      setShownMissing([]);
      setN((x) => Math.min(x + 1, keys.length - 1));
    } catch (e) {
      setError(toAppError(e).message);
    } finally {
      setBusy(false);
    }
  }

  async function finish() {
    setBusy(true);
    setError(null);
    try {
      const s = await onboardingFinish();
      onStatus(s);
      if (s.done) onFinished();
    } catch (e) {
      setError(toAppError(e).message);
    } finally {
      setBusy(false);
    }
  }

  const go = (i: number) => {
    setShownMissing([]);
    setIssues([]);
    setN(i);
  };

  return (
    <StartFrame
      top={
        <>
          <StepNav
            brand
            label={o.title}
            current={n}
            onSelect={go}
            steps={keys.map((k) => {
              const s = status.steps.find((x) => x.key === k);
              const done = k === REVIEW ? status.ready : !!s?.complete;
              return { key: k, label: o.steps[k] ?? k, short: o.stepsShort[k], filled: done ? 1 : 0, total: 1, caption: done ? o.stepDone : o.stepPending };
            })}
          />
          <div className="border-t border-line pt-6">
            <div className="min-w-0 space-y-1.5">
              <h1 className="onb-title text-title font-extrabold leading-tight tracking-tight">{o.steps[key]}</h1>
              <p className="max-w-[64ch] text-ui text-ink-2">{o.help[key]}</p>
              {key !== REVIEW && <p className="pt-1 text-caption text-ink-3">{o.requiredNote}</p>}
            </div>
          </div>
        </>
      }
      footer={
        <div className="flex flex-wrap items-center justify-between gap-3">
          <div className="flex gap-2">
            {n > 0 && <Button onClick={() => go(n - 1)}>{o.back}</Button>}
            {n === 0 && onBack && <TextButton onClick={onBack}>{o.back}</TextButton>}
          </div>
          {key === REVIEW ? (
            <Button variant="primary" disabled={busy || !status.ready} onClick={finish}>
              <Icon name="check" size={16} strokeWidth={2.4} />
              {busy ? o.finishing : o.finish}
            </Button>
          ) : (
            <Button variant="primary" disabled={busy} onClick={() => save()}>
              {busy ? es.common.saving : o.next}
            </Button>
          )}
        </div>
      }
    >
      <div className="space-y-6">
        <div key={key} className="anim-rise min-h-[14rem] space-y-6">

        {key === "institution" && (
          <div className="space-y-4">
            <TextInput label={p.fields.name} required value={d.institution.name} onChange={(e) => setInst({ name: e.target.value })} error={issue("institution.name")} />
            <fieldset className="space-y-2">
              <legend className="field-label">{p.fields.kind}</legend>
              {(Object.entries(p.kinds) as [InstitutionKind, string][]).map(([value, label]) => (
                <RadioCard key={value} name="kind" value={value} title={label} note={p.kindNotes[value]} checked={d.institution.kind === value} onChange={() => setInst({ kind: value })} />
              ))}
            </fieldset>
            <TextArea label={p.fields.mission} required rows={3} value={d.institution.mission ?? ""} onChange={(e) => setInst({ mission: txt(e.target.value) })} />
          </div>
        )}

        {key === "location" && (
          <div className="space-y-6">
            <FormSection title={o.steps.location}>
              <div className="grid grid-cols-1 items-start gap-4 sm:grid-cols-2">
                <Select label={ins.state} required options={options(ins.states)} value={d.institution.state ?? ""} onChange={(e) => setInst({ state: txt(e.target.value) })} error={issue("institution.state")} />
                <TextInput label={ins.municipality} required value={d.institution.municipality ?? ""} onChange={(e) => setInst({ municipality: txt(e.target.value) })} />
                <TextInput label={p.fields.phone} hint={p.privateNote} inputMode="tel" value={d.institution.contact_phone ?? ""} onChange={(e) => setInst({ contact_phone: txt(e.target.value) })} />
                <TextInput label={p.fields.email} inputMode="email" value={d.institution.contact_email ?? ""} onChange={(e) => setInst({ contact_email: txt(e.target.value) })} />
              </div>
            </FormSection>
            <FormSection title={p.cards.legal}>
              <div className="grid grid-cols-1 items-start gap-4 sm:grid-cols-2">
                <Select label={ins.legalForm} required options={options(ins.legalForms)} value={d.institution.legal_form ?? ""} onChange={(e) => setInst({ legal_form: txt(e.target.value) })} />
                <TextInput label={ins.foundedYear} required inputMode="numeric" value={d.institution.founded_year?.toString() ?? ""} onChange={(e) => setInst({ founded_year: num(e.target.value) })} error={issue("institution.founded_year")} />
                <TextInput label={p.fields.rfc} value={d.institution.legal_rfc ?? ""} onChange={(e) => setInst({ legal_rfc: txt(e.target.value) })} />
                <TextInput label={p.fields.legalRep} value={d.institution.legal_rep_name ?? ""} onChange={(e) => setInst({ legal_rep_name: txt(e.target.value) })} />
              </div>
              <Pills name="donee" label={ins.authorizedDonee} labels={ins.registry} value={d.institution.authorized_donee} onChange={(v) => setInst({ authorized_donee: v })} />
              <Pills name="cluni" label={ins.cluni} labels={ins.registry} value={d.institution.cluni} onChange={(v) => setInst({ cluni: v })} />
            </FormSection>
          </div>
        )}

        {key === "people" && (
          <div className="grid grid-cols-1 items-start gap-4 sm:grid-cols-2">
            <TextInput label={p.fields.capacity} required suffix={o.unitPeople} inputMode="numeric" value={d.capacity_total?.toString() ?? ""} onChange={(e) => set({ capacity_total: num(e.target.value) })} error={issue("capacity_total")} />
            {status.records.served > 0 ? (
              <Alert tone="info">{o.records.served(status.records.served)}</Alert>
            ) : (
              <TextInput label={ins.servedEstimate} required suffix={o.unitPeople} inputMode="numeric" value={d.served_estimate?.toString() ?? ""} onChange={(e) => set({ served_estimate: num(e.target.value) })} error={issue("served_estimate")} />
            )}
          </div>
        )}

        {key === "team" &&
          (status.records.staff > 0 ? (
            <Alert tone="info">{o.records.staff(status.records.staff)}</Alert>
          ) : (
            <div className="grid grid-cols-1 items-start gap-4 sm:grid-cols-2">
              <TextInput label={ins.staffPaidEstimate} required suffix={o.unitPeople} inputMode="numeric" value={d.staff_paid_estimate?.toString() ?? ""} onChange={(e) => set({ staff_paid_estimate: num(e.target.value) })} error={issue("staff_paid_estimate")} />
              <TextInput label={ins.staffVolunteerEstimate} required hint={o.zeroHint} suffix={o.unitPeople} inputMode="numeric" value={d.staff_volunteer_estimate?.toString() ?? ""} onChange={(e) => set({ staff_volunteer_estimate: num(e.target.value) })} error={issue("staff_volunteer_estimate")} />
            </div>
          ))}

        {key === "money" && (
          <div className="space-y-6">
            <TextInput
              label={p.fields.annualBudget}
              required
              hint={p.finance.expenses.estimateHelp}
              prefix="$"
              suffix={o.unitYear}
              value={d.annual_budget_mxn?.toString() ?? ""}
              onChange={(e) => set({ annual_budget_mxn: e.target.value.trim() === "" ? null : parsePesos(e.target.value) })}
              error={issue("annual_budget_mxn")}
            />
            <FormSection title={o.income.title}>
              {status.records.fee_payers > 0 && <Alert tone="info">{o.records.fees}</Alert>}
              {d.income.map((x, i) => {
                const change = (patch: Partial<typeof x>) => set({ income: d.income.map((y, j) => (j === i ? { ...y, ...patch } : y)) });
                return (
                  <div key={i} className="grid items-start gap-3 rounded-inset border border-line p-4 sm:grid-cols-3">
                    <div className="flex items-end gap-3 sm:col-span-3">
                      <TextInput className="flex-1" label={o.income.label} placeholder={o.income.placeholder} value={x.label} onChange={(e) => change({ label: e.target.value })} error={issue(`income[${i}].label`)} />
                      <Button variant="plain" onClick={() => set({ income: d.income.filter((_, j) => j !== i) })} aria-label={es.common.remove}>
                        <Icon name="trash" size={16} />
                      </Button>
                    </div>
                    <Select label={o.income.columns.kind} options={INCOME_KINDS.map((k) => [k, p.finance.incomeKinds[k]![0]])} value={x.kind} onChange={(e) => change({ kind: e.target.value as IncomeKind })} />
                    <TextInput label={o.income.amount} prefix="$" value={x.amount_mxn?.toString() ?? ""} onChange={(e) => change({ amount_mxn: e.target.value.trim() === "" ? null : parsePesos(e.target.value) })} />
                    <Select label={o.income.columns.period} options={[["monthly", p.finance.period.monthly], ["annual", p.finance.period.annual]]} value={x.period} onChange={(e) => change({ period: e.target.value as Period })} />
                  </div>
                );
              })}
              <div>
                <Button size="sm" variant="soft" onClick={() => set({ income: [...d.income, { label: "", kind: "occasional_donation", amount_mxn: null, period: "annual" }] })}>
                  <Icon name="plus" size={16} />
                  {o.income.add}
                </Button>
              </div>
            </FormSection>
          </div>
        )}

        {key === "building" && (
          <div className="space-y-4">
            <div className="grid grid-cols-1 items-start gap-4 sm:grid-cols-2">
              <TextInput label={o.floors} hint={o.floorsHint} required inputMode="numeric" value={d.floors?.toString() ?? ""} onChange={(e) => set({ floors: num(e.target.value) })} error={issue("site.floors")} />
              <TextInput label={es.facilities.building.built_m2} suffix="m²" inputMode="numeric" value={d.built_m2?.toString() ?? ""} onChange={(e) => set({ built_m2: num(e.target.value) })} error={issue("site.built_m2")} />
            </div>
            <Pills name="tenure" label={es.facilities.building.tenure} labels={es.facilities.building.tenures} value={d.tenure} onChange={(v) => set({ tenure: v })} />
            {(d.tenure === "loan" || d.tenure === "rent") && (
              <TextInput label={es.facilities.building.tenure_until} inputMode="numeric" value={d.tenure_until?.toString() ?? ""} onChange={(e) => set({ tenure_until: num(e.target.value) })} error={issue("site.tenure_until")} />
            )}
            <YesNo name="tenure_documented" label={es.facilities.building.tenure_documented} value={d.tenure_documented} onChange={(v) => set({ tenure_documented: v })} />
          </div>
        )}

        {key === REVIEW && <Review status={status} onEdit={go} />}
        </div>

        {shownMissing.length > 0 && (
          <Alert tone="warn">
            {o.stillMissing} {shownMissing.map((m) => o.missing[m] ?? m).join(", ")}.
          </Alert>
        )}
        {issues.length > 0 && !issues.some((i) => issue(i.field)) && <Alert tone="error">{issues.map((i) => es.issues[i.code] ?? es.errors.generic).join(" ")}</Alert>}
        {error && <Alert tone="error">{error}</Alert>}

      </div>
      {quarantine && <QuarantineDialog report={quarantine} busy={busy} onRedact={() => save("redact")} onNotPersonal={() => save("not_personal")} onCancel={() => setQuarantine(null)} />}
    </StartFrame>
  );
}

/** Everything the steps say, by step, with what is still missing and a way back to each one. */
function Review({ status, onEdit }: { status: OnboardingStatus; onEdit: (i: number) => void }) {
  const d = status.data;
  const inst = d.institution;
  const none = o.reviewEmpty;
  const n = (v: number | null, unit = "") => (v === null ? none : `${v.toLocaleString("es-MX")}${unit}`);
  const rows: Record<string, [string, string][]> = {
    institution: [
      [p.fields.name, inst.name || none],
      [p.fields.kind, p.kinds[inst.kind]],
      [p.fields.mission, inst.mission ?? none],
    ],
    location: [
      [ins.state, inst.state ? ins.states[inst.state]! : none],
      [ins.municipality, inst.municipality ?? none],
      [p.fields.phone, inst.contact_phone ?? none],
      [p.fields.email, inst.contact_email ?? none],
      [ins.legalForm, inst.legal_form ? ins.legalForms[inst.legal_form]! : none],
      [ins.foundedYear, inst.founded_year?.toString() ?? none],
      [ins.authorizedDonee, inst.authorized_donee ? ins.registry[inst.authorized_donee]! : none],
      [ins.cluni, inst.cluni ? ins.registry[inst.cluni]! : none],
    ],
    people: [
      [p.fields.capacity, n(d.capacity_total, " personas")],
      [ins.servedEstimate, status.records.served > 0 ? o.registered(status.records.served) : n(d.served_estimate, " personas")],
    ],
    team: [
      [ins.staffPaidEstimate, status.records.staff > 0 ? o.registered(status.records.staff) : n(d.staff_paid_estimate)],
      [ins.staffVolunteerEstimate, status.records.staff > 0 ? "—" : n(d.staff_volunteer_estimate)],
    ],
    money: [
      [p.fields.annualBudget, d.annual_budget_mxn === null ? none : `$${d.annual_budget_mxn.toLocaleString("es-MX")}`],
      [o.income.title, d.income.length ? d.income.map((x) => x.label).join(", ") : none],
    ],
    building: [
      [es.facilities.building.floors, n(d.floors)],
      [es.facilities.building.built_m2, n(d.built_m2, " m²")],
      [es.facilities.building.tenure, d.tenure ? es.facilities.building.tenures[d.tenure]! : none],
    ],
  };
  const pending = status.steps.filter((x) => !x.complete).length;
  return (
    <div className="space-y-4">
      <Alert tone={pending === 0 ? "ok" : "warn"}>{pending === 0 ? o.reviewReady : o.reviewPending(pending)}</Alert>
      <div className="gap-3 md:columns-2">
        {status.steps.map((s, i) => (
          <section key={s.key} className="mb-3 flex break-inside-avoid flex-col gap-3 rounded-inset bg-inset p-4">
            <div className="flex items-center justify-between gap-2">
              <h2 className="text-ui font-bold">{o.steps[s.key]}</h2>
              <div className="flex items-center gap-2">
                <Tag tone={s.complete ? "green" : "amber"} variant="soft" icon={s.complete ? "check" : "warn"}>
                  {s.complete ? o.stepDone : o.stepPending}
                </Tag>
                <TextButton onClick={() => onEdit(i)}>{es.facilities.edit}</TextButton>
              </div>
            </div>
            <Facts
              columns={1}
              items={(rows[s.key] ?? []).map(([label, value]): [string, React.ReactNode] => [label, value === none ? <span className="font-medium text-ink-3">{none}</span> : value])}
            />
            {!s.complete && <p className="text-small font-semibold text-amber-ink">{`${o.stillMissing} ${s.missing.map((m) => o.missing[m] ?? m).join(", ")}.`}</p>}
          </section>
        ))}
      </div>
    </div>
  );
}
