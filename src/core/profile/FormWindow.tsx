import { useQuery, useQueryClient } from "@tanstack/react-query";
import { useEffect, useState } from "react";
import { FormRenderer } from "../../components/FormRenderer";
import { QuarantineDialog } from "../../components/QuarantineDialog";
import { Alert, Button, Modal } from "../../components/ui";
import { es } from "../../i18n/es-MX";
import { formGet, formSave, toAppError } from "../../lib/tauri";
import type { Decision, FormValues, ProfileIssue, ProfileView, QuarantineReport } from "../../lib/types";
import { ONBOARDING_KEY } from "../onboarding/api";

export const formKey = (id: string) => ["form", id];

/**
 * A window of «Mi institución» whose fields come from Rust (ADR-033): it asks for the form, draws it and hands the
 * values back to Rust, which checks them, passes them through the scanner and saves them. Nothing here knows which
 * field goes where in the profile.
 */
export function FormWindow({ id, title, onSaved, onClose }: { id: string; title: string; onSaved: (profile: ProfileView) => void; onClose: () => void }) {
  const qc = useQueryClient();
  const form = useQuery({ queryKey: formKey(id), queryFn: () => formGet(id) });
  const [values, setValues] = useState<FormValues | null>(null);
  const [issues, setIssues] = useState<ProfileIssue[]>([]);
  const [quarantine, setQuarantine] = useState<QuarantineReport | null>(null);
  const [busy, setBusy] = useState(false);
  const [failure, setFailure] = useState<string | null>(null);

  // what is saved is the starting point, once
  useEffect(() => {
    if (form.data && values === null) setValues(form.data.values);
  }, [form.data, values]);

  async function save(decision?: Decision) {
    if (!values) return;
    setBusy(true);
    setFailure(null);
    setIssues([]);
    try {
      const out = await formSave(id, values, decision);
      if (out.status === "saved") {
        setQuarantine(null);
        qc.setQueryData(["profile"], out.profile);
        void qc.invalidateQueries({ queryKey: ONBOARDING_KEY });
        void qc.invalidateQueries({ queryKey: formKey(id) });
        onSaved(out.profile);
        onClose();
      } else if (out.status === "invalid") {
        setQuarantine(null);
        setIssues(out.issues);
      } else {
        setQuarantine(out.report);
      }
    } catch (e) {
      setFailure(toAppError(e).message);
    } finally {
      setBusy(false);
    }
  }

  const errors = Object.fromEntries(issues.map((i) => [i.field, es.issues[i.code] ?? es.issues.code_unknown]));
  const unplaced = issues.filter((i) => !form.data?.spec.sections.some((s) => s.fields.some((f) => f.id === i.field)));

  return (
    <>
      <Modal
        title={title}
        onClose={onClose}
        footer={
          <>
            <Button onClick={onClose}>{es.common.cancel}</Button>
            <Button variant="primary" disabled={busy || !values} onClick={() => void save()}>
              {busy ? es.common.saving : es.common.save}
            </Button>
          </>
        }
      >
        {form.isError && <Alert tone="error">{toAppError(form.error).message}</Alert>}
        {form.data && values && <FormRenderer spec={form.data.spec} values={values} onChange={setValues} errors={errors} />}
        {(failure || unplaced.length > 0) && (
          <div className="mt-4">
            <Alert tone="error">{failure ?? unplaced.map((i) => es.issues[i.code] ?? es.issues.code_unknown).join(" ")}</Alert>
          </div>
        )}
      </Modal>
      {quarantine && (
        <QuarantineDialog
          report={quarantine}
          busy={busy}
          onRedact={() => void save("redact")}
          onNotPersonal={() => void save("not_personal")}
          onCancel={() => setQuarantine(null)}
        />
      )}
    </>
  );
}
