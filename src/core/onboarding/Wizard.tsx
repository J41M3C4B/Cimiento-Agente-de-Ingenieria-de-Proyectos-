import { useState } from "react";
import { Icon } from "../../components/icons";
import { Alert, Button, Facts, StepNav, Status, TextButton } from "../../components/ui";
import { FormRenderer, applies, shownValue } from "../../components/FormRenderer";
import { QuarantineDialog } from "../../components/QuarantineDialog";
import { es } from "../../i18n/es-MX";
import { toAppError } from "../../lib/tauri";
import type { Decision, FormValues, QuarantineReport } from "../../lib/types";
import { onboardingFinish, onboardingSave } from "./api";
import type { OnboardingIssue, OnboardingStatus, StepStatus } from "./api";
import { StartFrame, asideOf } from "./Welcome";

const o = es.onboarding;
const REVIEW = "review";
/** What a missing field is called, in the words of its window. */
const fieldName = (id: string) => es.forms.fields[id]?.label ?? id;
/** «Para seguir falta: …» with the names of the fields, closed with a period unless the last one is a question. */
const stillMissing = (ids: string[]) => {
  const list = ids.map(fieldName).join(", ");
  return `${o.stillMissing} ${list}${/[?.]$/.test(list) ? "" : "."}`;
};

/**
 * The indispensable data of the institution, step by step (ADR-031, ADR-042). Rust says the steps and draws each one
 * from the catalog of forms, so a field reads and checks the same as in its window. Every «Guardar y seguir» saves
 * what there is through that window; it goes on only when Rust says the step is complete, and names what is missing
 * otherwise. The last screen reviews it all and closes it.
 */
export function Wizard({ status, onStatus, onFinished, onBack }: { status: OnboardingStatus; onStatus: (s: OnboardingStatus) => void; onFinished: () => void; onBack?: () => void }) {
  const screens = [...status.steps.map((s) => s.key), REVIEW];
  const firstOpen = status.steps.findIndex((s) => !s.complete);
  const [n, setN] = useState(firstOpen === -1 ? screens.length - 1 : firstOpen);
  const [values, setValues] = useState<FormValues>(status.values);
  const [issues, setIssues] = useState<OnboardingIssue[]>([]);
  const [shownMissing, setShownMissing] = useState<string[]>([]);
  const [quarantine, setQuarantine] = useState<QuarantineReport | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const key = screens[n]!;
  const step = status.steps.find((s) => s.key === key);

  async function save(decision?: Decision) {
    if (!step) return;
    setBusy(true);
    setError(null);
    setIssues([]);
    try {
      const mine = Object.fromEntries(Object.entries(values).filter(([id]) => step.form.sections.some((s) => s.fields.some((f) => f.id === id))));
      const out = await onboardingSave(step.key, mine, decision);
      if (out.status === "quarantine") return setQuarantine(out.report);
      setQuarantine(null);
      if (out.status === "invalid") return setIssues(out.issues);
      onStatus(out.onboarding);
      // the last indispensable datum finishes the first start (ADR-042): the app opens
      if (out.onboarding.done) return onFinished();
      setValues(out.onboarding.values);
      const now = out.onboarding.steps.find((s) => s.key === step.key)!;
      if (!now.complete) return setShownMissing(now.missing);
      setShownMissing([]);
      setN((x) => Math.min(x + 1, screens.length - 1));
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

  const errors = Object.fromEntries(issues.map((i) => [i.field, es.issues[i.code] ?? es.issues.code_unknown]));
  const unplaced = issues.filter((i) => !step?.form.sections.some((s) => s.fields.some((f) => f.id === i.field)));
  return (
    <StartFrame
      aside={asideOf(key)}
      top={
        <StepNav
          brand
          label={o.title}
          current={n}
          onSelect={go}
          steps={screens.map((id) => {
            const ok = id === REVIEW ? status.ready : (status.steps.find((s) => s.key === id)?.complete ?? false);
            return { key: id, label: o.steps[id] ?? id, short: o.stepsShort[id], filled: ok ? 1 : 0, total: 1, caption: ok ? o.stepDone : o.stepPending };
          })}
        />
      }
      footer={
        <div className="flex flex-wrap items-center justify-between gap-3">
          {n > 0 ? <Button onClick={() => go(n - 1)}>{o.back}</Button> : onBack ? <Button onClick={onBack}>{o.back}</Button> : <span />}
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
        <header className="space-y-2">
          <h1 className="onb-title text-title font-bold leading-tight tracking-tight">{o.steps[key]}</h1>
          <p className="onb-lead onb-narrow-only">{o.about[key]?.text}</p>
        </header>

        <div key={key} className="anim-rise space-y-6">
          {step && (
            <>
              {key === "figures" && status.records.served > 0 && <Alert tone="info">{o.records.served(status.records.served)}</Alert>}
              {key === "figures" && status.records.staff > 0 && <Alert tone="info">{o.records.staff(status.records.staff)}</Alert>}
              <FormRenderer bare spec={step.form} values={values} onChange={setValues} errors={errors} />
            </>
          )}
          {key === REVIEW && <Review status={status} onEdit={go} />}
        </div>

        {shownMissing.length > 0 && (
          <Alert tone="pending">
            {stillMissing(shownMissing)}
          </Alert>
        )}
        {unplaced.length > 0 && <Alert tone="error">{unplaced.map((i) => es.issues[i.code] ?? es.errors.generic).join(" ")}</Alert>}
        {error && <Alert tone="error">{error}</Alert>}
      </div>
      {quarantine && <QuarantineDialog report={quarantine} busy={busy} onRedact={() => save("redact")} onNotPersonal={() => save("not_personal")} onCancel={() => setQuarantine(null)} />}
    </StartFrame>
  );
}

/** Everything the steps say, with what is still missing and a way back to each one. */
function Review({ status, onEdit }: { status: OnboardingStatus; onEdit: (i: number) => void }) {
  const none = o.reviewEmpty;
  const pending = status.steps.filter((s) => !s.complete).length;
  // what does not apply with the answers is not asked, so it is not reviewed either (the capacity, where nobody stays)
  const rows = (s: StepStatus): [string, React.ReactNode][] =>
    s.form.sections
      .flatMap((sec) => sec.fields)
      .filter((f) => applies(f, status.values))
      .map((f) => {
        const v = status.values[f.id];
        const shown = v === undefined || (Array.isArray(v) && v.length === 0) ? <span className="font-medium text-ink-3">{none}</span> : shownValue(f, v);
        return [fieldName(f.id), shown];
      });
  return (
    <div className="space-y-4">
      <Alert tone={pending === 0 ? "ok" : "pending"}>{pending === 0 ? o.reviewReady : o.reviewPending(pending)}</Alert>
      <div className="gap-3 md:columns-2">
        {status.steps.map((s, i) => (
          <section key={s.key} className="mb-3 flex break-inside-avoid flex-col gap-3 rounded-inset bg-inset p-4">
            <div className="flex items-center justify-between gap-2">
              <h2 className="text-ui font-bold">{o.steps[s.key]}</h2>
              <TextButton onClick={() => onEdit(i)}>{es.facilities.edit}</TextButton>
            </div>
            <Status kind={s.complete ? "ok" : "pending"}>{s.complete ? o.stepDone : o.stepPending}</Status>
            <Facts columns={1} items={rows(s)} />
            {!s.complete && <p className="text-small font-semibold text-amber-ink">{stillMissing(s.missing)}</p>}
          </section>
        ))}
      </div>
    </div>
  );
}
