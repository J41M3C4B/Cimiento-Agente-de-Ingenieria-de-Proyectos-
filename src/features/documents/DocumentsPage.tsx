import { useQuery, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import { Alert, Button, Modal, Section, TextArea, TextInput } from "../../components/ui";
import { QuarantineDialog } from "../../components/QuarantineDialog";
import { es } from "../../i18n/es-MX";
import { documentAddText, documentEmergencyDelete, documentsList, toAppError } from "../../lib/tauri";
import type { Decision, DocumentSummary, QuarantineReport } from "../../lib/types";

const t = es.documents;

export function DocumentsPage() {
  const qc = useQueryClient();
  const docs = useQuery({ queryKey: ["documents"], queryFn: documentsList });
  const [name, setName] = useState("");
  const [text, setText] = useState("");
  const [busy, setBusy] = useState(false);
  const [notice, setNotice] = useState<{ tone: "ok" | "error" | "warn"; text: string } | null>(null);
  const [quarantine, setQuarantine] = useState<QuarantineReport | null>(null);
  const [toDelete, setToDelete] = useState<DocumentSummary | null>(null);

  async function add(decision?: Decision) {
    setBusy(true);
    setNotice(null);
    try {
      const out = await documentAddText(name, text, decision);
      if (out.status === "saved") {
        setQuarantine(null);
        setName("");
        setText("");
        setNotice({ tone: "ok", text: es.common.saved });
        await qc.invalidateQueries({ queryKey: ["documents"] });
      } else if (out.status === "quarantine") {
        setQuarantine(out.report);
      } else {
        setQuarantine(null);
        setNotice({ tone: "warn", text: es.quarantine.roster });
      }
    } catch (e) {
      setQuarantine(null);
      setNotice({ tone: "error", text: toAppError(e).message });
    } finally {
      setBusy(false);
    }
  }

  async function remove(doc: DocumentSummary) {
    setBusy(true);
    try {
      await documentEmergencyDelete(doc.id);
      setToDelete(null);
      setNotice({ tone: "ok", text: t.deleted });
      await qc.invalidateQueries({ queryKey: ["documents"] });
    } catch (e) {
      setNotice({ tone: "error", text: toAppError(e).message });
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className="space-y-6">
      <header className="space-y-1.5">
        <h1 className="text-[24px] font-semibold leading-tight tracking-tight">{t.title}</h1>
        <p className="text-stone-700">{t.intro}</p>
      </header>

      <Section title={t.add}>
        <form
          className="space-y-4"
          onSubmit={(e) => {
            e.preventDefault();
            void add();
          }}
        >
          <TextInput label={t.name} value={name} onChange={(e) => setName(e.target.value)} />
          <TextArea label={t.text} rows={10} value={text} onChange={(e) => setText(e.target.value)} />
          {notice && <Alert tone={notice.tone}>{notice.text}</Alert>}
          <Button type="submit" variant="primary" disabled={busy || !name.trim() || !text.trim()}>
            {t.submit}
          </Button>
        </form>
      </Section>

      <Section title={t.list}>
        {docs.data && docs.data.length === 0 && <p>{t.empty}</p>}
        <ul className="space-y-3">
          {docs.data?.map((d) => (
            <li key={d.id} className="flex flex-wrap items-center justify-between gap-3 rounded-lg border border-stone-200 p-4">
              <div>
                <p className="text-[15px] font-semibold">{d.display_name}</p>
                <p className="text-stone-700">
                  {t.fragments(d.chunks)}
                  {d.redactions_count > 0 && ` · ${t.covered(d.redactions_count)}`}
                </p>
              </div>
              <Button variant="danger" onClick={() => setToDelete(d)} disabled={busy}>
                {t.delete}
              </Button>
            </li>
          ))}
        </ul>
      </Section>

      {quarantine && (
        <QuarantineDialog
          report={quarantine}
          busy={busy}
          onRedact={() => add("redact")}
          onNotPersonal={() => add("not_personal")}
          onCancel={() => setQuarantine(null)}
        />
      )}

      {toDelete && (
        <Modal title={t.deleteTitle} onClose={() => setToDelete(null)}>
          <p className="text-[15px]">
            <strong>{toDelete.display_name}</strong>
          </p>
          <p className="text-[15px]">{t.deleteBody}</p>
          <div className="flex flex-wrap gap-3">
            <Button variant="danger" onClick={() => remove(toDelete)} disabled={busy}>
              {t.deleteConfirm}
            </Button>
            <Button onClick={() => setToDelete(null)} disabled={busy}>
              {es.common.cancel}
            </Button>
          </div>
        </Modal>
      )}
    </div>
  );
}
