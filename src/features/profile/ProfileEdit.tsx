import { zodResolver } from "@hookform/resolvers/zod";
import { useForm } from "react-hook-form";
import { Alert, Button, Choice, Modal, RadioCard, TextArea, TextInput } from "../../components/ui";
import type { Tone } from "../../components/ui";
import { es } from "../../i18n/es-MX";
import type { ProfileInput, ProfileIssue, ProfileView } from "../../lib/types";
import { FormSection } from "./FormSection";
import { emptyForm, formSchema, fromView, toInput, type FormValues } from "./profileForm";

const t = es.profile;

/** What is being edited: one card of the profile, or one item of a list (income, space). */
export type Edit =
  | { kind: "institution" | "contact" | "legal" | "capacity" }
  | { kind: "income" | "facility"; index: number | null };

const conditionTone: Record<string, Tone> = { good: "green", fair: "amber", poor: "red", critical: "red" };
const kindOptions = Object.entries(t.kinds) as [string, string][];

const titleOf = (e: Edit) =>
  e.kind === "income" ? (e.index === null ? t.modal.incomeAdd : t.modal.incomeEdit)
  : e.kind === "facility" ? (e.index === null ? t.modal.facilityAdd : t.modal.facilityEdit)
  : t.modal[e.kind];

/**
 * One window for each card of the profile: it holds the fields of that card only, starts from what is saved and
 * saves the whole profile with that change. The page does not keep a form of its own: what is on screen is what
 * is saved.
 */
export function ProfileEdit({
  edit, view, issues, busy, onCommit, onClose,
}: {
  edit: Edit;
  view: ProfileView | null;
  issues: ProfileIssue[];
  busy: boolean;
  onCommit: (input: ProfileInput, onSaved: () => void) => void;
  onClose: () => void;
}) {
  const start = view ? fromView(view) : emptyForm();
  // a new item goes at the end of its list, and that is the one the window edits
  if (edit.kind === "income" && edit.index === null) start.income = [...start.income, { label: "", annual_amount_mxn: "" }];
  if (edit.kind === "facility" && edit.index === null) start.facilities = [...start.facilities, { kind: "", count: "1", condition: "", accessible: "", notes: "" }];
  const i = edit.kind === "income" ? (edit.index ?? start.income.length - 1) : edit.kind === "facility" ? (edit.index ?? start.facilities.length - 1) : 0;

  const { register, handleSubmit, formState } = useForm<FormValues>({ resolver: zodResolver(formSchema), defaultValues: start });
  const fe = formState.errors;
  const err = (e?: { message?: string }) => (e?.message ? (es.issues[e.message] ?? e.message) : undefined);

  return (
    <Modal
      title={titleOf(edit)}
      onClose={onClose}
      footer={
        <>
          <Button onClick={onClose}>{es.common.cancel}</Button>
          <Button type="submit" form="profile-edit" variant="primary" disabled={busy}>
            {busy ? es.common.saving : es.common.save}
          </Button>
        </>
      }
    >
      <form id="profile-edit" onSubmit={handleSubmit((v) => onCommit(toInput(v), onClose))} noValidate className="space-y-4">
        {edit.kind === "institution" && (
          <>
            <TextInput label={t.fields.name} required autoFocus {...register("name")} />
            <fieldset className="space-y-2">
              <legend className="field-label">{t.fields.kind}</legend>
              {kindOptions.map(([value, label]) => (
                <RadioCard key={value} value={value} title={label} note={t.kindNotes[value]} {...register("kind")} />
              ))}
            </fieldset>
            <TextArea label={t.fields.mission} {...register("mission")} />
          </>
        )}
        {edit.kind === "contact" && (
          <>
            <p className="text-ui text-ink-2">{t.privateNote}</p>
            <TextInput label={t.fields.phone} inputMode="tel" autoFocus {...register("contact_phone")} />
            <TextInput label={t.fields.email} inputMode="email" {...register("contact_email")} />
          </>
        )}
        {edit.kind === "legal" && (
          <>
            <p className="text-ui text-ink-2">{t.legalNote}</p>
            <TextInput label={t.fields.rfc} autoFocus {...register("legal_rfc")} />
            <TextInput label={t.fields.legalRep} {...register("legal_rep_name")} />
          </>
        )}
        {edit.kind === "capacity" && (
          <>
            <div className="grid grid-cols-1 gap-4 sm:grid-cols-2">
              <TextInput label={t.fields.capacity} suffix="personas" inputMode="numeric" autoFocus error={err(fe.capacity_total)} {...register("capacity_total")} />
              <TextInput label={t.fields.annualBudget} prefix="$" suffix="al año" inputMode="numeric" error={err(fe.annual_budget_mxn)} {...register("annual_budget_mxn")} />
            </div>
            <TextArea label={t.fields.notes} {...register("notes")} />
          </>
        )}
        {edit.kind === "income" && (
          <>
            <TextInput label={t.fields.incomeLabel} autoFocus placeholder={t.modal.incomePlaceholder} {...register(`income.${i}.label`)} />
            <TextInput label={t.fields.amount} prefix="$" suffix="al año" inputMode="numeric" error={err(fe.income?.[i]?.annual_amount_mxn)} {...register(`income.${i}.annual_amount_mxn`)} />
          </>
        )}
        {edit.kind === "facility" && (
          <>
            <FormSection title={t.modal.facilitySpace}>
              <div className="grid grid-cols-1 gap-4 sm:grid-cols-[1fr_160px]">
                <TextInput label={t.fields.facilityKind} required autoFocus placeholder={t.modal.facilityPlaceholder} {...register(`facilities.${i}.kind`)} />
                <TextInput label={t.fields.count} inputMode="numeric" error={err(fe.facilities?.[i]?.count)} {...register(`facilities.${i}.count`)} />
              </div>
            </FormSection>
            <FormSection title={t.modal.facilityToday}>
              <fieldset>
                <legend className="field-label">{t.fields.condition}</legend>
                <div className="flex flex-wrap gap-2">
                  {Object.entries(t.condition).map(([value, label]) => (
                    <Choice key={value} value={value} tone={conditionTone[value]} {...register(`facilities.${i}.condition`)}>
                      {label}
                    </Choice>
                  ))}
                  <Choice value="" tone="neutral" {...register(`facilities.${i}.condition`)}>
                    {t.fields.optionNone}
                  </Choice>
                </div>
              </fieldset>
              <fieldset>
                <legend className="field-label">{t.fields.accessible}</legend>
                <div className="flex flex-wrap gap-2">
                  <Choice value="yes" tone="green" {...register(`facilities.${i}.accessible`)}>
                    {es.common.yes}
                  </Choice>
                  <Choice value="no" tone="amber" {...register(`facilities.${i}.accessible`)}>
                    {es.common.no}
                  </Choice>
                  <Choice value="" tone="neutral" {...register(`facilities.${i}.accessible`)}>
                    {t.fields.optionNone}
                  </Choice>
                </div>
              </fieldset>
              <TextArea label={t.fields.facilityNotes} rows={3} {...register(`facilities.${i}.notes`)} />
            </FormSection>
          </>
        )}
        {issues.length > 0 && (
          <Alert tone="error">
            <ul className="list-disc pl-4">
              {issues.map((x, n) => (
                <li key={n}>{es.issues[x.code] ?? es.issues.label_missing}</li>
              ))}
            </ul>
          </Alert>
        )}
      </form>
    </Modal>
  );
}
