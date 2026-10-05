import { zodResolver } from "@hookform/resolvers/zod";
import { useForm } from "react-hook-form";
import { Alert, Button, Modal, Select, TextArea, TextInput } from "../../components/ui";
import { es } from "../../i18n/es-MX";
import type { ProfileInput, ProfileIssue, ProfileView } from "../../lib/types";
import { emptyForm, formSchema, fromView, toInput, type FormValues } from "./profileForm";

const t = es.profile;

/** What is being edited: one card of the profile, or one item of a list (income, space). */
export type Edit =
  | { kind: "institution" | "contact" | "legal" | "capacity" }
  | { kind: "income" | "facility"; index: number | null };

const kindOptions = Object.entries(t.kinds) as [string, string][];
const withBlank = (o: Record<string, string>): [string, string][] => [["", t.fields.optionNone], ...Object.entries(o)];

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
            <TextInput label={t.fields.name} autoFocus {...register("name")} />
            <fieldset>
              <legend className="mb-1.5 text-[13px] font-medium text-stone-700">{t.fields.kind}</legend>
              <div className="space-y-2">
                {kindOptions.map(([value, label]) => (
                  <label
                    key={value}
                    className="flex cursor-pointer items-center gap-3 rounded-lg border border-stone-300 px-3.5 py-2.5 text-[14px] transition-colors hover:border-stone-400 has-[:checked]:border-blue-800 has-[:checked]:bg-blue-50 has-[:focus-visible]:ring-[3px] has-[:focus-visible]:ring-blue-100"
                  >
                    <input type="radio" value={value} className="h-4 w-4 shrink-0 accent-blue-800" {...register("kind")} />
                    {label}
                  </label>
                ))}
              </div>
            </fieldset>
          </>
        )}
        {edit.kind === "contact" && (
          <>
            <p className="text-stone-700">{t.privateNote}</p>
            <TextInput label={t.fields.phone} inputMode="tel" autoFocus {...register("contact_phone")} />
            <TextInput label={t.fields.email} inputMode="email" {...register("contact_email")} />
          </>
        )}
        {edit.kind === "legal" && (
          <>
            <p className="text-stone-700">{t.legalNote}</p>
            <TextInput label={t.fields.rfc} autoFocus {...register("legal_rfc")} />
            <TextInput label={t.fields.legalRep} {...register("legal_rep_name")} />
          </>
        )}
        {edit.kind === "capacity" && (
          <>
            <div className="grid gap-4 sm:grid-cols-2">
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
            <div className="grid gap-4 sm:grid-cols-[1fr_120px]">
              <TextInput label={t.fields.facilityKind} autoFocus placeholder={t.modal.facilityPlaceholder} {...register(`facilities.${i}.kind`)} />
              <TextInput label={t.fields.count} inputMode="numeric" error={err(fe.facilities?.[i]?.count)} {...register(`facilities.${i}.count`)} />
            </div>
            <div className="grid gap-4 sm:grid-cols-2">
              <Select label={t.fields.condition} options={withBlank(t.condition)} {...register(`facilities.${i}.condition`)} />
              <Select
                label={t.fields.accessible}
                options={[["", t.fields.optionNone], ["yes", es.common.yes], ["no", es.common.no]]}
                {...register(`facilities.${i}.accessible`)}
              />
            </div>
            <TextArea label={t.fields.facilityNotes} rows={3} {...register(`facilities.${i}.notes`)} />
          </>
        )}
        {issues.length > 0 && (
          <Alert tone="error">
            <ul className="list-disc pl-5">
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
