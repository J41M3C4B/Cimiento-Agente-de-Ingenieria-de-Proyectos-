import { useEffect, useState } from "react";
import type { ReactNode } from "react";
import { Icon } from "../../components/icons";
import { Alert, Button, DropZone, Eyebrow, Inset, Modal, RadioCard, Select, TextArea, TextInput } from "../../components/ui";
import { es } from "../../i18n/es-MX";
import { baseName, extOf, fileSize, isPlainText } from "./documentsModel";
import type { DocCol, Draft, UploadTarget } from "./documentsModel";

const t = es.documents;
const f = t.form;

function FormSection({ children }: { children: ReactNode }) {
  return (
    <div className="flex items-center gap-3">
      <Eyebrow>{children}</Eyebrow>
      <span className="h-px flex-1 bg-line" aria-hidden="true" />
    </div>
  );
}

/**
 * The window to upload a document. From a subfolder the destination is known (it shows as a route) and the form only
 * asks for what is missing; from the main button it also asks whose the document is.
 * The text of the document reaches the backend as text: plain files are read here, other kinds ask to paste it.
 */
export function UploadModal({
  target,
  initial,
  scope: startScope,
  file,
  donors,
  busy,
  notice,
  onSubmit,
  onClose,
}: {
  target: UploadTarget | null;
  initial?: Draft | null;
  /** whose the document is when no subfolder says it */
  scope?: DocCol;
  /** a file that was dropped on a subfolder */
  file?: File | null;
  donors: string[];
  busy: boolean;
  notice: { tone: "ok" | "error" | "warn"; text: string } | null;
  onSubmit: (draft: Draft) => void;
  onClose: () => void;
}) {
  const [scope, setScope] = useState<DocCol>(initial?.scope ?? target?.col ?? startScope ?? "inst");
  const [name, setName] = useState(initial?.name ?? "");
  const [text, setText] = useState(initial?.text ?? "");
  const [donor, setDonor] = useState(initial?.donor ?? target?.donor ?? "");
  const [kind, setKind] = useState(initial?.kind ?? target?.type ?? "");
  const [picked, setPicked] = useState<Draft["file"]>(initial?.file ?? null);
  const [unreadable, setUnreadable] = useState(false);

  function take(file: File) {
    setPicked({ name: file.name, size: fileSize(file.size), ext: extOf(file.name) || "txt" });
    setName((n) => (n.trim() ? n : baseName(file.name)));
    if (!isPlainText(file)) {
      // TODO(logica): PDF, Word and Excel need a reader in the backend (or a library here); until then the text is pasted
      setUnreadable(true);
      return;
    }
    setUnreadable(false);
    const reader = new FileReader();
    reader.onload = () => setText(String(reader.result ?? ""));
    reader.onerror = () => setUnreadable(true);
    reader.readAsText(file);
  }

  useEffect(() => {
    if (file) take(file);
    // only once: the file that was dropped to open this window
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const col = target?.col ?? scope;
  const askDonor = col === "donor" && !target?.donor;
  const askKind = !target?.type;
  const kinds = col === "donor" ? f.kindsDonor : f.kindsInst;
  const ready = name.trim() !== "" && text.trim() !== "" && (!askDonor || donor.trim() !== "");

  return (
    <Modal
      title={target ? t.uploadTo(target.label) : t.upload}
      onClose={onClose}
      footer={
        <>
          <Button onClick={onClose} disabled={busy}>
            {es.common.cancel}
          </Button>
          <Button type="submit" form="document-form" variant="primary" disabled={busy || !ready}>
            {busy ? es.common.saving : es.common.save}
          </Button>
        </>
      }
    >
      <form
        id="document-form"
        className="space-y-6"
        onSubmit={(e) => {
          e.preventDefault();
          if (ready) onSubmit({ scope: col, name, text, donor, kind, file: picked, target });
        }}
      >
        {target ? (
          <Inset className="flex flex-wrap items-center gap-2 !py-3 text-ui font-semibold text-ink-2">
            <span className="sr-only">{f.where}: </span>
            <Icon name="folder" size={18} className="text-ink-3" />
            <span>{es.documents.cols[target.col].title}</span>
            <Icon name="next" size={14} className="text-ink-3" />
            <b className="font-extrabold text-ink">{target.label}</b>
          </Inset>
        ) : (
          <fieldset className="space-y-2">
            <legend className="field-label">{f.scope}</legend>
            <RadioCard name="scope" title={f.scopeInst[0]!} note={f.scopeInst[1]} checked={scope === "inst"} onChange={() => setScope("inst")} />
            <RadioCard name="scope" title={f.scopeDonor[0]!} note={f.scopeDonor[1]} checked={scope === "donor"} onChange={() => setScope("donor")} />
          </fieldset>
        )}

        <div className="space-y-4">
          <FormSection>{f.fileSection}</FormSection>
          <div>
            <span className="field-label">{f.fileLabel}</span>
            <DropZone label={f.drop} hint={f.dropHint} file={picked} changeHint={f.change} onFile={take} accept=".pdf,.doc,.docx,.xls,.xlsx,.csv,.ppt,.pptx,.txt,.md" />
          </div>
          {unreadable && <Alert tone="warn">{f.unreadable}</Alert>}
          {askDonor && <TextInput label={f.donor} required list="document-donors" placeholder={f.donorPh} value={donor} onChange={(e) => setDonor(e.target.value)} />}
          {askKind && (
            <Select label={f.kind} value={kind} onChange={(e) => setKind(e.target.value)} options={[["", f.kindPick], ...kinds.map((k): [string, string] => [k, k])]} />
          )}
          <datalist id="document-donors">
            {donors.map((d) => (
              <option key={d} value={d} />
            ))}
          </datalist>
        </div>

        <div className="space-y-4">
          <FormSection>{f.pasteSection}</FormSection>
          <TextInput label={t.name} required value={name} onChange={(e) => setName(e.target.value)} />
          <TextArea label={t.text} hint={f.pasteHint} required rows={8} value={text} onChange={(e) => setText(e.target.value)} />
        </div>

        {notice && notice.tone !== "ok" && <Alert tone={notice.tone}>{notice.text}</Alert>}
      </form>
    </Modal>
  );
}
