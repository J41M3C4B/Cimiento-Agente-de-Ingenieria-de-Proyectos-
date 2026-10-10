import { zodResolver } from "@hookform/resolvers/zod";
import { useForm } from "react-hook-form";
import { Alert, Button, FormSection, Modal, Select, TextArea, TextInput } from "../../components/ui";
import { es } from "../../i18n/es-MX";
import type { ProfileIssue, ProfileView } from "../../lib/types";
import { formSchema, fromView, type FormValues } from "./profileForm";

const t = es.profile;
const ins = es.institution;
const options = (labels: Record<string, string>): [string, string][] => [["", es.facilities.select], ...Object.entries(labels)];

/**
 * What is being edited: one card of the profile. The money is edited in its module (ADR-032), and «institution» is a
 * form described in Rust (`FormWindow`, ADR-033); this window holds the others until they move there too.
 */
export type Edit = { kind: "institution" | "contact" | "legal" | "capacity" };

const titleOf = (e: Edit) => t.modal[e.kind];

/**
 * One window for each card of the profile: it holds the fields of that card only, starts from what is saved and
 * hands back the whole form; the page saves the profile with that change. The page does not keep a form of its own:
 * what is on screen is what is saved.
 */
export function ProfileEdit({
  edit, view, issues, busy, onCommit, onClose,
}: {
  edit: Edit;
  view: ProfileView | null;
  issues: ProfileIssue[];
  busy: boolean;
  onCommit: (values: FormValues, onSaved: () => void) => void;
  onClose: () => void;
}) {
  const start = fromView(view);

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
            <FormSection title={t.modal.capacityPeople}>
              <div className="grid grid-cols-1 gap-4 sm:grid-cols-2">
                <TextInput label={t.fields.capacity} suffix="personas" inputMode="numeric" autoFocus error={err(fe.capacity_total)} {...register("capacity_total")} />
                <TextInput label={ins.servedEstimate} suffix="personas" inputMode="numeric" error={err(fe.served_estimate)} {...register("served_estimate")} />
              </div>
            </FormSection>
            <FormSection title={t.modal.capacityStaff}>
              <div className="grid grid-cols-1 gap-4 sm:grid-cols-2">
                <TextInput label={ins.staffPaidEstimate} inputMode="numeric" error={err(fe.staff_paid_estimate)} {...register("staff_paid_estimate")} />
                <TextInput label={ins.staffVolunteerEstimate} inputMode="numeric" error={err(fe.staff_volunteer_estimate)} {...register("staff_volunteer_estimate")} />
              </div>
            </FormSection>
            <TextArea label={t.fields.notes} {...register("notes")} />
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
