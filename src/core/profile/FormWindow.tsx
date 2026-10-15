import { useQuery, useQueryClient } from "@tanstack/react-query";
import { useEffect, useState } from "react";
import { FormRenderer } from "../../components/FormRenderer";
import { QuarantineDialog } from "../../components/QuarantineDialog";
import { Icon } from "../../components/icons";
import { Alert, Button, Modal } from "../../components/ui";
import { es } from "../../i18n/es-MX";
import { formGet, formSave, institutionFillFromDocuments, proposalsReject, toAppError } from "../../lib/tauri";
import type { Decision, FormValues, ProfileIssue, ProfileView, Proposal, QuarantineReport } from "../../lib/types";
import { useAssistant } from "../assistant/Assistant";
import { ONBOARDING_KEY } from "../onboarding/api";
import { fillNotice } from "./fromDocuments";
import type { FillNotice } from "./fromDocuments";
import { OVERVIEW_KEY } from "./gaps";

export const formKey = (id: string) => ["form", id];

/**
 * A window of «Mi institución» whose fields come from Rust (ADR-033): it asks for the form, draws it and hands the
 * values back to Rust, which checks them, passes them through the scanner and saves them. Nothing here knows which
 * field goes where in the profile.
 *
 * The suggestions of the AI (IA4) show under their fields: «Usar» puts one in its field, where the person may correct
 * it, and «Guardar» saves it like anything typed; «Descartar» drops it. None is saved without that click.
 */
export function FormWindow({ id, title, onSaved, onClose }: { id: string; title: string; onSaved: (profile: ProfileView) => void; onClose: () => void }) {
  const qc = useQueryClient();
  const form = useQuery({ queryKey: formKey(id), queryFn: () => formGet(id) });
  const [values, setValues] = useState<FormValues | null>(null);
  const [issues, setIssues] = useState<ProfileIssue[]>([]);
  const [quarantine, setQuarantine] = useState<QuarantineReport | null>(null);
  const [busy, setBusy] = useState(false);
  const [failure, setFailure] = useState<string | null>(null);
  // the suggestions put in their fields, saved with «Guardar»
  const [used, setUsed] = useState<string[]>([]);
  const assistant = useAssistant();
  // closing the window (saved or not) ends «Llenarlo por mí» for it in the Assistant
  const close = () => {
    assistant?.windowClosed(id);
    onClose();
  };
  // «Llenar con mis documentos» (C2): the Reader looks in the Expediente for the empty fields of this window
  const [searching, setSearching] = useState(false);
  const [fromDocs, setFromDocs] = useState<FillNotice | null>(null);
  async function fillFromDocuments() {
    setSearching(true);
    setFromDocs(null);
    setFailure(null);
    try {
      const out = await institutionFillFromDocuments(id);
      setFromDocs(fillNotice(out));
      void qc.invalidateQueries({ queryKey: formKey(id) });
      void qc.invalidateQueries({ queryKey: OVERVIEW_KEY });
    } catch (e) {
      setFailure(toAppError(e).message);
    } finally {
      setSearching(false);
    }
  }
  const waiting = (form.data?.proposals ?? []).filter((p) => !used.includes(p.id));

  const use = (p: Proposal) => {
    setValues((v) => ({ ...(v ?? {}), [p.field_id]: p.value }));
    setUsed((u) => [...u, p.id]);
  };
  async function discard(ps: Proposal[]) {
    setFailure(null);
    try {
      await proposalsReject(ps.map((p) => p.id));
      void qc.invalidateQueries({ queryKey: formKey(id) });
      void qc.invalidateQueries({ queryKey: OVERVIEW_KEY });
    } catch (e) {
      setFailure(toAppError(e).message);
    }
  }

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
      const out = await formSave(id, values, decision, used);
      if (out.status === "saved") {
        setQuarantine(null);
        qc.setQueryData(["profile"], out.profile);
        void qc.invalidateQueries({ queryKey: ONBOARDING_KEY });
        void qc.invalidateQueries({ queryKey: OVERVIEW_KEY });
        void qc.invalidateQueries({ queryKey: formKey(id) });
        onSaved(out.profile);
        close();
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
        onClose={close}
        footer={
          <>
            <Button onClick={close}>{es.common.cancel}</Button>
            <Button variant="primary" disabled={busy || !values} onClick={() => void save()}>
              {busy ? es.common.saving : es.common.save}
            </Button>
          </>
        }
      >
        {form.isError && <Alert tone="error">{toAppError(form.error).message}</Alert>}
        {form.data?.fill && (
          <div className="flex flex-wrap gap-2">
            {assistant && (
              <Button size="sm" variant="secondary" onClick={() => assistant.fill(form.data.fill!)}>
                <Icon name="sparkles" size={16} />
                {es.assistant.fillMe}
              </Button>
            )}
            <Button size="sm" variant="secondary" disabled={searching} onClick={() => void fillFromDocuments()}>
              <Icon name="file" size={16} />
              {searching ? es.suggestions.searchingDocuments : es.suggestions.fillFromDocuments}
            </Button>
          </div>
        )}
        {fromDocs && <Alert tone={fromDocs.tone}>{fromDocs.text}</Alert>}
        {waiting.length > 0 && (
          <Alert tone="info">
            <p>{es.suggestions.waiting(waiting.length)}</p>
            <Button size="sm" variant="plain" className="mt-2" onClick={() => void discard(waiting)}>
              {es.suggestions.discardAll}
            </Button>
          </Alert>
        )}
        {form.data && values && (
          <FormRenderer spec={form.data.spec} values={values} onChange={setValues} errors={errors} suggestions={waiting} onUse={use} onDiscard={(p) => void discard([p])} />
        )}
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
