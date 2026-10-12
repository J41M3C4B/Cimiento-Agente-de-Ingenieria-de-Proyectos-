import { useQuery, useQueryClient } from "@tanstack/react-query";
import { useEffect, useMemo, useRef, useState } from "react";
import { Icon } from "../../components/icons";
import { Alert, Button, Toast, PageHeader } from "../../components/ui";
import { QuarantineDialog } from "../../components/QuarantineDialog";
import { es } from "../../i18n/es-MX";
import { toBase64, useReadingProgress } from "../../lib/files";
import { documentAddFile, documentAddText, documentEmergencyDelete, documentsList, documentsSearch, toAppError } from "../../lib/tauri";
import type { Decision, QuarantineReport } from "../../lib/types";
import { DocColumn } from "./DocColumn";
import type { Inside } from "./DocColumn";
import { addDonorDocument, defaultMode, donorItems, institutionItems } from "./documentsModel";
import type { DocCol, DocItem, Draft, GroupMode, UploadTarget } from "./documentsModel";
import { UploadModal } from "./UploadModal";

const t = es.documents;

type Notice = { tone: "ok" | "error" | "warn"; text: string };
/** The upload window: where it goes (if known), a file dropped on it, and what was typed if it has to come back. */
type Form = { target: UploadTarget | null; file: File | null; initial: Draft | null; scope?: DocCol };

export function DocumentsPage() {
  const qc = useQueryClient();
  const docs = useQuery({ queryKey: ["documents"], queryFn: documentsList });
  const [form, setForm] = useState<Form | null>(null);
  const [busy, setBusy] = useState(false);
  const [notice, setNotice] = useState<Notice | null>(null);
  const [toast, setToast] = useState<string | null>(null);
  const [quarantine, setQuarantine] = useState<QuarantineReport | null>(null);
  const [asking, setAsking] = useState<string | null>(null);
  const [pending, setPending] = useState<DocItem | null>(null);
  const draft = useRef<Draft | null>(null);
  const [query, setQuery] = useState<Record<DocCol, string>>({ inst: "", donor: "" });
  const [mode, setMode] = useState<Record<DocCol, GroupMode>>({ inst: defaultMode("inst"), donor: defaultMode("donor") });
  // what the person opened or closed by hand; it is kept when the grouping changes
  const [open, setOpen] = useState<Record<string, boolean>>({});
  // the search inside the text of the documents (IA5), once the person stops typing
  const [asked, setAsked] = useState("");
  useEffect(() => {
    const timer = setTimeout(() => setAsked(query.inst.trim()), 500);
    return () => clearTimeout(timer);
  }, [query.inst]);
  const searchable = asked.length >= 3 && (docs.data?.length ?? 0) > 0;
  const found = useQuery({ queryKey: ["documents-search", asked], queryFn: () => documentsSearch(asked), enabled: searchable, staleTime: 60_000 });
  const inside: Inside | null =
    query.inst.trim().length < 3 ? null : { loading: !searchable || asked !== query.inst.trim() || found.isFetching, passages: found.data?.passages ?? [] };

  useEffect(() => {
    if (!toast) return;
    const timer = setTimeout(() => setToast(null), 4000);
    return () => clearTimeout(timer);
  }, [toast]);

  // a scan or a photo is read as pictures while it is saved (OCR, ADR-035): its line says how far it goes
  const progress = useReadingProgress(pending !== null);
  const inst = useMemo(() => {
    const real = institutionItems(docs.data ?? []);
    if (!pending) return real;
    const detail = progress ? es.common.readingPictures(progress.done, progress.total) : pending.detail;
    return [{ ...pending, detail }, ...real];
  }, [docs.data, pending, progress]);
  const donor = useMemo(() => donorItems(), []);
  const donors = useMemo(() => [...new Set(donor.map((d) => d.donor).filter((d): d is string => !!d))], [donor]);

  async function add(d: Draft, decision?: Decision) {
    setBusy(true);
    setNotice(null);
    draft.current = d;
    try {
      if (d.scope === "donor") {
        const out = await addDonorDocument(d);
        if (out.status === "not_connected") {
          setNotice({ tone: "warn", text: t.form.donorSoon });
        }
        return;
      }
      // the window closes at once and the document shows as «Leyendo…» until the backend answers
      setForm(null);
      const year = String(new Date().getFullYear());
      setPending({ id: "pending", name: d.name.trim(), ext: d.file?.ext ?? "txt", type: t.kinds.internal!, year, detail: t.reading, reading: true });
      setOpen((cur) => ({ ...cur, [`inst:type:${t.kinds.internal}`]: true, [`inst:year:${year}`]: true }));
      const out = d.blob
        ? await documentAddFile(d.name, d.blob.name, await toBase64(d.blob), decision)
        : await documentAddText(d.name, d.text, decision);
      if (out.status === "saved") {
        setQuarantine(null);
        draft.current = null;
        setToast(es.common.saved);
        const doubtful = out.document.doubtful_pages;
        if (doubtful.length > 0) setNotice({ tone: "warn", text: t.savedDoubtful(es.common.pageList(doubtful)) });
        await qc.invalidateQueries({ queryKey: ["documents"] });
        await qc.invalidateQueries({ queryKey: ["documents-search"] });
      } else if (out.status === "quarantine") {
        setQuarantine(out.report);
      } else if (out.status === "unreadable") {
        setQuarantine(null);
        setNotice({ tone: "warn", text: t.fileUnreadable[out.reason] ?? es.errors.generic });
        setForm({ target: d.target, file: null, initial: d });
      } else {
        setQuarantine(null);
        setNotice({ tone: "warn", text: es.quarantine.roster });
        setForm({ target: d.target, file: null, initial: d });
      }
    } catch (e) {
      setQuarantine(null);
      setNotice({ tone: "error", text: toAppError(e).message });
      setForm({ target: d.target, file: null, initial: d });
    } finally {
      setPending(null);
      setBusy(false);
    }
  }

  async function remove(id: string) {
    setBusy(true);
    try {
      await documentEmergencyDelete(id);
      setAsking(null);
      setToast(t.deleted);
      await qc.invalidateQueries({ queryKey: ["documents"] });
      await qc.invalidateQueries({ queryKey: ["documents-search"] });
    } catch (e) {
      setNotice({ tone: "error", text: toAppError(e).message });
    } finally {
      setBusy(false);
    }
  }

  const openForm = (target: UploadTarget | null, file: File | null = null, scope?: DocCol) => {
    setNotice(null);
    setForm({ target, file, initial: null, scope });
  };

  const column = (col: DocCol, items: DocItem[]) => (
    <DocColumn
      col={col}
      items={items}
      query={query[col]}
      onQuery={(q) => setQuery((cur) => ({ ...cur, [col]: q }))}
      mode={mode[col]}
      onMode={(m) => setMode((cur) => ({ ...cur, [col]: m }))}
      open={open}
      onToggle={(id, now) => setOpen((cur) => ({ ...cur, [id]: now }))}
      onUpload={(target, scope) => openForm(target, null, scope)}
      onDropFile={(target, file) => openForm(target, file)}
      asking={asking}
      onAsk={setAsking}
      onRemove={(id) => void remove(id)}
      busy={busy}
      inside={col === "inst" ? inside : null}
    />
  );

  return (
    <div className="space-y-6">
      <PageHeader
        title={t.title}
        intro={t.intro}
        action={
          <Button variant="primary" onClick={() => openForm(null)}>
            <Icon name="upload" size={18} />
            {t.upload}
          </Button>
        }
      />

      {notice && !form && <Alert tone={notice.tone}>{notice.text}</Alert>}

      <div className="grid items-start gap-x-4 gap-y-6 min-[1000px]:grid-cols-2">
        {column("inst", inst)}
        {column("donor", donor)}
      </div>

      {form && (
        <UploadModal
          target={form.target}
          initial={form.initial}
          scope={form.scope}
          file={form.file}
          donors={donors}
          busy={busy}
          notice={notice}
          onSubmit={(d) => void add(d)}
          onClose={() => {
            setForm(null);
            setNotice(null);
          }}
        />
      )}

      {quarantine && (
        <QuarantineDialog
          report={quarantine}
          busy={busy}
          onRedact={() => draft.current && void add(draft.current, "redact")}
          onNotPersonal={() => draft.current && void add(draft.current, "not_personal")}
          onCancel={() => {
            setQuarantine(null);
            if (draft.current) setForm({ target: draft.current.target, file: null, initial: draft.current });
          }}
        />
      )}

      {toast && <Toast>{toast}</Toast>}
    </div>
  );
}
