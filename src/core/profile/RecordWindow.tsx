import { useQuery, useQueryClient } from "@tanstack/react-query";
import { open as pickFile } from "@tauri-apps/plugin-dialog";
import { useEffect, useState } from "react";
import { FormRenderer } from "../../components/FormRenderer";
import { Icon } from "../../components/icons";
import { ManualText } from "../../components/ManualHelp";
import { QuarantineDialog } from "../../components/QuarantineDialog";
import { Alert, Button, Inset, Modal } from "../../components/ui";
import { es } from "../../i18n/es-MX";
import { useReadingProgress } from "../../lib/files";
import {
  institutionRecordAttach,
  institutionRecordDetach,
  institutionRecordForm,
  institutionRecordOpen,
  institutionRecordSave,
  institutionRecords,
  manualEntry,
  toAppError,
} from "../../lib/tauri";
import type { AttachOutcome, Decision, Expediente, FormValues, InstitutionRecord, ProfileIssue, QuarantineReport } from "../../lib/types";

const t = es.profile.records;

export const RECORDS_KEY = ["institution", "records"];

export const useRecords = () => useQuery({ queryKey: RECORDS_KEY, queryFn: institutionRecords });

/** A document to open: a type of the catalog (with or without a row), or an `other` by its id (none to add one). */
export type RecordRef = { type: string; id: string | null; custom_name?: string | null };

/** The name of a document: the one of its type, or the one the institution gave it. */
export const recordTitle = (r: Pick<InstitutionRecord, "type" | "custom_name">) =>
  r.type === "other" ? (r.custom_name ?? t.newOther) : (t.types[r.type] ?? r.type);

/** The files a document takes (Rust says again what it can read). */
const EXTENSIONS = ["pdf", "docx", "jpg", "jpeg", "png"];

/**
 * What the person is told once a file is on its document: whether its text was read, what it filled in its own
 * window, what it left as suggestions in «Mi institución», the data of people covered, and a file that looks like
 * another document (ADR-038).
 */
export function attachedNotice(out: Extract<AttachOutcome, { status: "attached" }>): string {
  const label = (field: string) => es.forms.fields[field]?.label ?? field;
  const parts = [
    !out.readable
      ? t.attachedNoText
      : out.doubtful.length > 0
        ? t.attachedDoubtful(es.common.pageList(out.doubtful))
        : out.from_picture
          ? t.attachedFromPicture
          : t.attached,
  ];
  if (out.looks_like) parts.push(t.looksLike(t.types[out.looks_like] ?? out.looks_like));
  if (out.filled.length > 0) parts.push(t.filled(es.common.nameList(out.filled.map(label))));
  if (out.suggested > 0) parts.push(t.suggested(out.suggested));
  if (out.covered > 0) parts.push(t.covered(out.covered));
  return parts.join(" ");
}

/**
 * The window of a document of the record (ADR-033 §5): what the manual says it is, its state, dates and folio
 * (fields from Rust, which checks and keeps them) and its file. A file is chosen in the window of the system, so
 * Rust knows where the original is; it reads its text, passes it through the scanner and keeps only the clean text.
 */
export function RecordWindow({ record, onDone, onClose }: { record: RecordRef; onDone: (message: string) => void; onClose: () => void }) {
  const qc = useQueryClient();
  const [id, setId] = useState(record.id);
  const formKey = ["institution", "record-form", record.type, id];
  const form = useQuery({ queryKey: formKey, queryFn: () => institutionRecordForm(record.type, id), gcTime: 0 });
  const about = useQuery({ queryKey: ["manual", `record.${record.type}`], queryFn: () => manualEntry(`record.${record.type}`), enabled: record.type !== "other", staleTime: Infinity });
  const [values, setValues] = useState<FormValues | null>(null);
  const [issues, setIssues] = useState<ProfileIssue[]>([]);
  const [busy, setBusy] = useState(false);
  const [failure, setFailure] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const [held, setHeld] = useState<{ report: QuarantineReport; retry: (d: Decision) => void } | null>(null);
  // a scan or a photo is read as pictures, a few seconds a page (OCR, ADR-035)
  const [reading, setReading] = useState(false);
  const progress = useReadingProgress(reading);

  // what is saved is the starting point, once
  useEffect(() => {
    if (form.data && values === null) setValues(form.data.values);
  }, [form.data, values]);

  const kept = (e: Expediente) => {
    qc.setQueryData(RECORDS_KEY, e);
    // a file adds its clean text to Documentos
    void qc.invalidateQueries({ queryKey: ["documents"] });
  };

  async function act(action: () => Promise<void>) {
    setBusy(true);
    setFailure(null);
    setNotice(null);
    try {
      await action();
    } catch (e) {
      setFailure(toAppError(e).message);
    } finally {
      setBusy(false);
    }
  }

  const save = (decision?: Decision) =>
    act(async () => {
      if (!values) return;
      setIssues([]);
      const out = await institutionRecordSave(record.type, id, values, decision);
      if (out.status === "quarantine") {
        setHeld({ report: out.report, retry: (d) => void save(d) });
        return;
      }
      setHeld(null);
      if (out.status === "invalid") {
        setIssues(out.issues);
        return;
      }
      kept(out.expediente);
      onDone(es.common.saved);
      onClose();
    });

  // the data of people in its text are covered by the scanner without asking (decision of the institution, 2026-10-13)
  const attach = () =>
    act(async () => {
      const chosen = await pickFile({ multiple: false, directory: false, title: t.pickTitle, filters: [{ name: t.pickFilter, extensions: EXTENSIONS }] });
      if (typeof chosen !== "string") return;
      setReading(true);
      const out = await institutionRecordAttach(record.type, id, chosen).finally(() => setReading(false));
      if (out.status === "unreadable") {
        setFailure(t.unreadable[out.reason] ?? t.unreadable.damaged!);
        return;
      }
      kept(out.expediente);
      // a document with its file is had; what the file filled in its window shows; what the person typed stays
      const fresh = await institutionRecordForm(record.type, out.id);
      qc.setQueryData(["institution", "record-form", record.type, out.id], fresh);
      setId(out.id);
      const filled = Object.fromEntries(out.filled.filter((f) => f in fresh.values).map((f) => [f, fresh.values[f]]));
      setValues((v) => ({ ...(v ?? {}), ...filled, "institution.record.status": "have" }));
      setNotice(attachedNotice(out));
    });

  const open = () =>
    act(async () => {
      if (id && !(await institutionRecordOpen(id))) setFailure(t.openMissing);
    });

  const detach = () =>
    act(async () => {
      if (!id) return;
      kept(await institutionRecordDetach(id));
      void form.refetch();
      setNotice(t.detached);
    });

  const errors = Object.fromEntries(issues.map((i) => [i.field, es.issues[i.code] ?? es.issues.code_unknown]));
  const unplaced = issues.filter((i) => !form.data?.spec.sections.some((s) => s.fields.some((f) => f.id === i.field)));
  const file = form.data?.file ?? null;
  // where each datum read from the file came from, while it still says what the file said
  const readNotes = Object.fromEntries(
    Object.entries(form.data?.read ?? {})
      .filter(([field]) => values?.[field] === form.data?.values[field])
      .map(([field, r]) => [field, r.computed ? t.computedFrom : t.readFrom(r.page)]),
  );
  // an `other` gets its file once it is saved with its name
  const canAttach = record.type !== "other" || id !== null;

  return (
    <>
      <Modal
        title={recordTitle({ type: record.type, custom_name: record.custom_name ?? null })}
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
        {about.data && <ManualText entry={about.data} />}
        {form.isError && <Alert tone="error">{toAppError(form.error).message}</Alert>}
        {canAttach && (
          <Inset className="space-y-3 !p-4">
            <p className="text-small font-semibold text-ink-2">{t.file}</p>
            {file ? (
              <p className="flex items-center gap-2 text-ui">
                <Icon name="file" size={16} />
                <span className="min-w-0 truncate font-semibold">{file.name}</span>
                {!file.readable && <span className="text-small text-ink-3">{t.noText}</span>}
                {file.from_picture && <span className="text-small text-ink-3">{t.fromPicture}</span>}
              </p>
            ) : (
              <p className="text-ui text-ink-3">{t.noFile}</p>
            )}
            {reading && <p className="text-small text-ink-2">{progress ? es.common.readingPictures(progress.done, progress.total) : es.common.loading}</p>}
            <div className="flex flex-wrap gap-2">
              <Button size="sm" variant="secondary" disabled={busy} onClick={() => void attach()}>
                <Icon name="upload" size={16} />
                {file ? t.replace : t.attach}
              </Button>
              {file && id && (
                <>
                  <Button size="sm" variant="plain" disabled={busy} onClick={() => void open()}>
                    {t.open}
                  </Button>
                  <Button size="sm" variant="plain" disabled={busy} onClick={() => void detach()}>
                    {t.detach}
                  </Button>
                </>
              )}
            </div>
          </Inset>
        )}
        {notice && <Alert tone="ok">{notice}</Alert>}
        {form.data && values && <FormRenderer spec={form.data.spec} values={values} onChange={setValues} errors={errors} notes={readNotes} />}
        {(failure || unplaced.length > 0) && <Alert tone="error">{failure ?? unplaced.map((i) => es.issues[i.code] ?? es.issues.code_unknown).join(" ")}</Alert>}
      </Modal>
      {held && <QuarantineDialog report={held.report} busy={busy} onRedact={() => held.retry("redact")} onNotPersonal={() => held.retry("not_personal")} onCancel={() => setHeld(null)} />}
    </>
  );
}
