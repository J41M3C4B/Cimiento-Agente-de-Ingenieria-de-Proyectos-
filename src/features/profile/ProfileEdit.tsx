import { zodResolver } from "@hookform/resolvers/zod";
import { useForm } from "react-hook-form";
import type { UseFormRegister } from "react-hook-form";
import { Alert, Button, Choice, Modal, RadioCard, Select, TextArea, TextInput } from "../../components/ui";
import { INCOME_KINDS } from "./finance";
import { es } from "../../i18n/es-MX";
import type { FinanceInput, ProfileIssue, ProfileView } from "../../lib/types";
import { formSchema, fromView, type FormValues } from "./profileForm";

const t = es.profile;
const ins = es.institution;
const options = (labels: Record<string, string>): [string, string][] => [["", es.facilities.select], ...Object.entries(labels)];

/** What is being edited: one card of the profile, the approximate expense, or one item of a list (income, expense). */
export type Edit =
  | { kind: "institution" | "contact" | "legal" | "capacity" | "estimate" }
  | { kind: "income" | "expense"; index: number | null };

/** The money is saved in its own module (ADR-032); the rest, in the profile. */
export const isMoney = (e: Edit) => e.kind === "income" || e.kind === "expense" || e.kind === "estimate";

const kindOptions = Object.entries(t.kinds) as [string, string][];

const titleOf = (e: Edit) =>
  e.kind === "income" ? (e.index === null ? t.modal.incomeAdd : t.modal.incomeEdit)
  : e.kind === "expense" ? (e.index === null ? t.modal.expenseAdd : t.modal.expenseEdit)
  : t.modal[e.kind];

/** «Al mes» or «Al año»: Rust turns the amount into a year, so the person writes it the way they know it. */
function PeriodChoice({ name, register }: { name: `income.${number}.period` | `expenses.${number}.period`; register: UseFormRegister<FormValues> }) {
  return (
    <fieldset>
      <legend className="field-label">{t.finance.periodLabel}</legend>
      <div className="flex flex-wrap gap-2">
        <Choice value="monthly" tone="ink" {...register(name)}>
          {t.finance.period.monthly}
        </Choice>
        <Choice value="annual" tone="ink" {...register(name)}>
          {t.finance.period.annual}
        </Choice>
      </div>
    </fieldset>
  );
}

/**
 * One window for each card of the profile: it holds the fields of that card only, starts from what is saved and
 * hands back the whole form; the page saves the profile or the money (`isMoney`) with that change. The page does
 * not keep a form of its own: what is on screen is what is saved.
 */
export function ProfileEdit({
  edit, view, money, issues, busy, onCommit, onClose,
}: {
  edit: Edit;
  view: ProfileView | null;
  money: FinanceInput | null;
  issues: ProfileIssue[];
  busy: boolean;
  onCommit: (values: FormValues, onSaved: () => void) => void;
  onClose: () => void;
}) {
  const start = fromView(view, money);
  // a new item goes at the end of its list, and that is the one the window edits
  if (edit.kind === "income" && edit.index === null) start.income = [...start.income, { label: "", kind: "other", amount_mxn: "", period: "annual" }];
  if (edit.kind === "expense" && edit.index === null) start.expenses = [...start.expenses, { label: "", amount_mxn: "", period: "annual" }];
  const i =
    edit.kind === "income" ? (edit.index ?? start.income.length - 1)
    : edit.kind === "expense" ? (edit.index ?? start.expenses.length - 1)
    : 0;

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
      <form id="profile-edit" onSubmit={handleSubmit((v) => onCommit(v, onClose))} noValidate className="space-y-4">
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
            <div className="grid grid-cols-1 gap-4 sm:grid-cols-2">
              <Select label={ins.state} options={options(ins.states)} {...register("state")} />
              <TextInput label={ins.municipality} {...register("municipality")} />
            </div>
          </>
        )}
        {edit.kind === "legal" && (
          <>
            <p className="text-ui text-ink-2">{t.legalNote}</p>
            <TextInput label={t.fields.rfc} autoFocus {...register("legal_rfc")} />
            <TextInput label={t.fields.legalRep} {...register("legal_rep_name")} />
            <div className="grid grid-cols-1 gap-4 sm:grid-cols-2">
              <Select label={ins.legalForm} options={options(ins.legalForms)} {...register("legal_form")} />
              <TextInput label={ins.foundedYear} inputMode="numeric" error={err(fe.founded_year)} {...register("founded_year")} />
              <Select label={ins.authorizedDonee} options={options(ins.registry)} {...register("authorized_donee")} />
              <Select label={ins.cluni} options={options(ins.registry)} {...register("cluni")} />
            </div>
          </>
        )}
        {edit.kind === "capacity" && (
          <>
            <TextInput label={t.fields.capacity} suffix="personas" inputMode="numeric" autoFocus error={err(fe.capacity_total)} {...register("capacity_total")} />
            <div className="grid grid-cols-1 gap-4 sm:grid-cols-3">
              <TextInput label={ins.servedEstimate} inputMode="numeric" error={err(fe.served_estimate)} {...register("served_estimate")} />
              <TextInput label={ins.staffPaidEstimate} inputMode="numeric" error={err(fe.staff_paid_estimate)} {...register("staff_paid_estimate")} />
              <TextInput label={ins.staffVolunteerEstimate} inputMode="numeric" error={err(fe.staff_volunteer_estimate)} {...register("staff_volunteer_estimate")} />
            </div>
            <TextArea label={t.fields.notes} {...register("notes")} />
          </>
        )}
        {edit.kind === "estimate" && (
          <TextInput label={t.fields.annualBudget} hint={t.finance.expenses.estimateHelp} prefix="$" suffix="al año" autoFocus error={err(fe.annual_budget_mxn)} {...register("annual_budget_mxn")} />
        )}
        {edit.kind === "income" && (
          <>
            <TextInput label={t.fields.incomeLabel} autoFocus placeholder={t.modal.incomePlaceholder} {...register(`income.${i}.label`)} />
            <fieldset className="space-y-2">
              <legend className="field-label">{t.finance.kindLabel}</legend>
              {INCOME_KINDS.map((k) => (
                <RadioCard key={k} value={k} title={t.finance.incomeKinds[k]![0]} note={t.finance.incomeKinds[k]![1]} {...register(`income.${i}.kind`)} />
              ))}
            </fieldset>
            <div className="grid grid-cols-1 gap-4 sm:grid-cols-2">
              <TextInput label={t.finance.amountLabel} prefix="$" error={err(fe.income?.[i]?.amount_mxn)} {...register(`income.${i}.amount_mxn`)} />
              <PeriodChoice name={`income.${i}.period`} register={register} />
            </div>
          </>
        )}
        {edit.kind === "expense" && (
          <>
            <p className="text-ui text-ink-2">{t.finance.expenses.payrollNote}</p>
            <TextInput label={t.finance.expenses.labelField} autoFocus placeholder={t.finance.expenses.placeholder} {...register(`expenses.${i}.label`)} />
            <div className="grid grid-cols-1 gap-4 sm:grid-cols-2">
              <TextInput label={t.finance.amountLabel} prefix="$" error={err(fe.expenses?.[i]?.amount_mxn)} {...register(`expenses.${i}.amount_mxn`)} />
              <PeriodChoice name={`expenses.${i}.period`} register={register} />
            </div>
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
