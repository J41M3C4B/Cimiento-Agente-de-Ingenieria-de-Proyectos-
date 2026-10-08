import { zodResolver } from "@hookform/resolvers/zod";
import { useForm } from "react-hook-form";
import type { UseFormRegister } from "react-hook-form";
import { Alert, Button, Choice, Inset, Modal, RadioCard, Tag, TextInput } from "../../components/ui";
import { es } from "../../i18n/es-MX";
import type { FinanceInput, ProfileIssue } from "../../lib/types";
import { INCOME_KINDS } from "./finance";
import { financeSchema, fromFinance, type FinanceValues } from "./financeForm";

const t = es.profile;

/** What is being edited: the approximate expense, or one line of income or expenses. */
export type MoneyEdit = { kind: "estimate" } | { kind: "income" | "expense"; index: number | null };

const titleOf = (e: MoneyEdit) =>
  e.kind === "income" ? (e.index === null ? t.modal.incomeAdd : t.modal.incomeEdit)
  : e.kind === "expense" ? (e.index === null ? t.modal.expenseAdd : t.modal.expenseEdit)
  : t.modal.estimate;

/** «Al mes» or «Al año»: Rust turns the amount into a year, so the person writes it the way they know it. */
function PeriodChoice({ name, register }: { name: `income.${number}.period` | `expenses.${number}.period`; register: UseFormRegister<FinanceValues> }) {
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
 * One window for the approximate expense or for one line of money. It starts from what is saved and hands back the
 * whole money with that change; the page saves it in the finance module (ADR-032).
 */
export function FinanceEdit({
  edit, money, issues, busy, onCommit, onClose,
}: {
  edit: MoneyEdit;
  money: FinanceInput | null;
  issues: ProfileIssue[];
  busy: boolean;
  onCommit: (values: FinanceValues, onSaved: () => void) => void;
  onClose: () => void;
}) {
  const start = fromFinance(money);
  // a new line goes at the end of its list, and that is the one the window edits
  if (edit.kind === "income" && edit.index === null) start.income = [...start.income, { label: "", kind: "other", amount_mxn: "", period: "annual" }];
  if (edit.kind === "expense" && edit.index === null) start.expenses = [...start.expenses, { label: "", amount_mxn: "", period: "annual" }];
  const i = edit.kind === "income" ? (edit.index ?? start.income.length - 1) : edit.kind === "expense" ? (edit.index ?? start.expenses.length - 1) : 0;

  const { register, handleSubmit, formState } = useForm<FinanceValues>({ resolver: zodResolver(financeSchema), defaultValues: start });
  const fe = formState.errors;
  const err = (e?: { message?: string }) => (e?.message ? (es.issues[e.message] ?? e.message) : undefined);

  return (
    <Modal
      title={titleOf(edit)}
      onClose={onClose}
      footer={
        <>
          <Button onClick={onClose}>{es.common.cancel}</Button>
          <Button type="submit" form="finance-edit" variant="primary" disabled={busy}>
            {busy ? es.common.saving : es.common.save}
          </Button>
        </>
      }
    >
      <form id="finance-edit" onSubmit={handleSubmit((v) => onCommit(v, onClose))} noValidate className="space-y-4">
        {edit.kind === "estimate" && (
          <>
            <p className="text-ui text-ink-2">{t.modal.estimateIntro}</p>
            <Inset className="space-y-3 !p-4">
              <div className="text-small font-bold text-ink-2">{t.modal.estimateIncludes}</div>
              <ul className="flex flex-wrap gap-2">
                {t.modal.estimateItems.map((x) => (
                  <li key={x}>
                    <Tag tone="amber" variant="soft" icon="check">
                      {x}
                    </Tag>
                  </li>
                ))}
              </ul>
            </Inset>
            <TextInput label={t.fields.annualBudget} hint={t.finance.expenses.estimateHelp} prefix="$" suffix="al año" autoFocus error={err(fe.annual_budget_mxn)} {...register("annual_budget_mxn")} />
          </>
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
