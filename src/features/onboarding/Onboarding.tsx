import { zodResolver } from "@hookform/resolvers/zod";
import { useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import { useForm } from "react-hook-form";
import { Icon } from "../../components/icons";
import { SociaiLogo } from "../../components/Logo";
import { QuarantineDialog } from "../../components/QuarantineDialog";
import { Alert, Button, Eyebrow, RadioCard, TextArea, TextInput } from "../../components/ui";
import { es } from "../../i18n/es-MX";
import { devLoadFixture, profileSave, toAppError } from "../../lib/tauri";
import type { Decision, ProfileInput, ProfileIssue, QuarantineReport } from "../../lib/types";
import { emptyForm, formSchema, toInput, type FormValues } from "../profile/profileForm";
import { BrandArt } from "./BrandArt";

const t = es.onboarding;
const p = es.profile;
const LAST = t.steps.length - 1;
const kindOptions = Object.entries(p.kinds) as [string, string][];

/**
 * The first screen, shown until the institution is registered: split in two. On the left the blue of the brand with
 * the logo and a picture; on the right the forms, one short step at a time. Only the name is needed to go on.
 * It saves the same profile «Mi institución» shows, so when it ends the person lands in the program.
 */
export function Onboarding() {
  const qc = useQueryClient();
  const [step, setStep] = useState(0);
  const [busy, setBusy] = useState(false);
  const [message, setMessage] = useState<string | null>(null);
  const [issues, setIssues] = useState<ProfileIssue[]>([]);
  const [quarantine, setQuarantine] = useState<{ input: ProfileInput; report: QuarantineReport } | null>(null);

  const { register, handleSubmit, getValues, setError, setFocus, formState } = useForm<FormValues>({ resolver: zodResolver(formSchema), defaultValues: emptyForm() });
  const fe = formState.errors;
  const err = (e?: { message?: string }) => (e?.message ? (es.issues[e.message] ?? e.message) : undefined);
  const copy = t.steps[step]!;

  async function save(input: ProfileInput, decision?: Decision) {
    setBusy(true);
    setIssues([]);
    setMessage(null);
    try {
      const out = await profileSave(input, decision);
      if (out.status === "saved") {
        setQuarantine(null);
        // the app shows the program as soon as the profile exists
        qc.setQueryData(["profile"], out.profile);
      } else if (out.status === "invalid") setIssues(out.issues);
      else setQuarantine({ input, report: out.report });
    } catch (e) {
      setMessage(toAppError(e).message);
    } finally {
      setBusy(false);
    }
  }

  function next() {
    if (step === 0 && getValues("name").trim() === "") {
      setError("name", { message: "name_missing" });
      return setFocus("name");
    }
    setStep((s) => Math.min(s + 1, LAST));
  }

  async function loadExample(name: "asilo" | "casa-hogar") {
    setBusy(true);
    try {
      qc.setQueryData(["profile"], await devLoadFixture(name));
    } catch (e) {
      setMessage(toAppError(e).message);
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className="onb text-body text-ink">
      <aside className="onb-brand">
        <SociaiLogo size="lg" />
        <BrandArt className="onb-art" />
        <div className="onb-foot space-y-6">
          <div className="onb-copy space-y-4">
            <h1 className="text-hero font-bold tracking-tight">{t.headline}</h1>
            <p className="text-body font-medium opacity-90">{t.lead}</p>
          </div>
          <p className="onb-dots">
            {t.points.map((x, i) => (
              <span key={x} className="contents">
                {i > 0 && <i aria-hidden="true" />}
                {x}
              </span>
            ))}
          </p>
        </div>
      </aside>

      <main className="onb-form">
        <form
          className="onb-panel space-y-6"
          noValidate
          onSubmit={(e) => {
            if (step < LAST) {
              e.preventDefault();
              return next();
            }
            void handleSubmit((v) => save(toInput(v)))(e);
          }}
        >
          <div className="space-y-4">
            <Eyebrow>{t.stepOf(step + 1, t.steps.length)}</Eyebrow>
            <div className="onb-progress" role="img" aria-label={t.stepOf(step + 1, t.steps.length)}>
              {t.steps.map((s, i) => (
                <i key={s.title} className={i <= step ? "on" : ""} />
              ))}
            </div>
          </div>

          <div className="space-y-1.5">
            <h2 className="text-subtitle font-bold leading-tight tracking-tight">{copy.title}</h2>
            <p className="text-ui text-ink-2">{copy.help}</p>
          </div>

          <div className="space-y-5">
            {step === 0 && (
              <>
                <TextInput label={p.fields.name} required autoFocus autoComplete="off" error={err(fe.name)} {...register("name")} />
                <fieldset className="space-y-2">
                  <legend className="field-label">{p.fields.kind}</legend>
                  {kindOptions.map(([value, label]) => (
                    <RadioCard key={value} value={value} title={label} note={p.kindNotes[value]} {...register("kind")} />
                  ))}
                </fieldset>
                <TextArea label={p.fields.mission} {...register("mission")} />
              </>
            )}
            {step === 1 && (
              <>
                <TextInput label={p.fields.phone} icon="phone" inputMode="tel" autoFocus {...register("contact_phone")} />
                <TextInput label={p.fields.email} icon="mail" inputMode="email" {...register("contact_email")} />
              </>
            )}
            {step === 2 && (
              <>
                <TextInput label={p.fields.capacity} suffix="personas" inputMode="numeric" autoFocus error={err(fe.capacity_total)} {...register("capacity_total")} />
                <TextInput label={p.fields.annualBudget} hint={p.finance.expenses.estimateHelp} prefix="$" suffix="al año" error={err(fe.annual_budget_mxn)} {...register("annual_budget_mxn")} />
              </>
            )}
            {message && <Alert tone="error">{message}</Alert>}
            {issues.length > 0 && (
              <Alert tone="error">
                <ul className="list-disc pl-4">
                  {issues.map((x, n) => (
                    <li key={n}>{es.issues[x.code] ?? es.issues.label_missing}</li>
                  ))}
                </ul>
              </Alert>
            )}
          </div>

          <div className="flex flex-wrap items-center gap-3">
            {step > 0 && (
              <Button onClick={() => setStep((s) => s - 1)} disabled={busy}>
                <Icon name="back" size={16} />
                {t.back}
              </Button>
            )}
            <Button type="submit" variant="primary" className="ml-auto" disabled={busy}>
              {step < LAST ? t.next : busy ? es.common.saving : t.finish}
              <Icon name={step < LAST ? "next" : "check"} size={16} strokeWidth={2.4} />
            </Button>
          </div>
          {step > 0 && (
            <p className="text-center">
              <Button size="sm" variant="plain" onClick={() => (step < LAST ? next() : void handleSubmit((v) => save(toInput(v)))())} disabled={busy}>
                {t.skip}
              </Button>
            </p>
          )}

          {import.meta.env.DEV && (
            <p className="flex flex-wrap items-center justify-center gap-3 text-small text-ink-3">
              {p.banner.loadExample}
              <Button size="sm" variant="plain" onClick={() => void loadExample("asilo")} disabled={busy}>
                {p.devAsilo}
              </Button>
              <Button size="sm" variant="plain" onClick={() => void loadExample("casa-hogar")} disabled={busy}>
                {p.devCasaHogar}
              </Button>
            </p>
          )}
        </form>
      </main>

      {quarantine && (
        <QuarantineDialog
          report={quarantine.report}
          busy={busy}
          onRedact={() => void save(quarantine.input, "redact")}
          onNotPersonal={() => void save(quarantine.input, "not_personal")}
          onCancel={() => setQuarantine(null)}
        />
      )}
    </div>
  );
}
