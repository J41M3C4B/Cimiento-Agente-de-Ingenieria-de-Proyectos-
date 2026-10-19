import { useQuery, useQueryClient } from "@tanstack/react-query";
import { useEffect, useMemo, useState } from "react";
import { Icon } from "../../components/icons";
import { Alert, Button, Toast, PageHeader } from "../../components/ui";
import { es } from "../../i18n/es-MX";
import { documentEmergencyDelete, documentsList, documentsSearch, toAppError } from "../../lib/tauri";
import { RECORDS_KEY, RecordWindow } from "../profile/RecordWindow";
import type { RecordRef } from "../profile/RecordWindow";
import { DocColumn } from "./DocColumn";
import type { Inside } from "./DocColumn";
import { DrawerUploadModal } from "./DrawerUploadModal";
import { defaultMode, institutionItems } from "./documentsModel";
import type { GroupMode } from "./documentsModel";

const t = es.documents;

/**
 * The tray of documents (ADR-036): every file goes to one drawer, and here the person says which. Today the core has
 * one drawer, the Expediente: a file comes in through the window of its document of the Expediente, the same one of
 * «Mi institución», so it does not matter where it was uploaded from.
 */
export function DocumentsPage() {
  const qc = useQueryClient();
  const docs = useQuery({ queryKey: ["documents"], queryFn: documentsList });
  const [choosing, setChoosing] = useState(false);
  const [record, setRecord] = useState<RecordRef | null>(null);
  const [busy, setBusy] = useState(false);
  const [failure, setFailure] = useState<string | null>(null);
  const [toast, setToast] = useState<string | null>(null);
  const [asking, setAsking] = useState<string | null>(null);
  const [query, setQuery] = useState("");
  const [mode, setMode] = useState<GroupMode>(defaultMode);
  // what the person opened or closed by hand; it is kept when the grouping changes
  const [open, setOpen] = useState<Record<string, boolean>>({});
  // the search inside the text of the documents (IA5), once the person stops typing
  const [asked, setAsked] = useState("");
  useEffect(() => {
    const timer = setTimeout(() => setAsked(query.trim()), 500);
    return () => clearTimeout(timer);
  }, [query]);
  const searchable = asked.length >= 3 && (docs.data?.length ?? 0) > 0;
  const found = useQuery({ queryKey: ["documents-search", asked], queryFn: () => documentsSearch(asked), enabled: searchable, staleTime: 60_000 });
  const inside: Inside | null =
    query.trim().length < 3 ? null : { loading: !searchable || asked !== query.trim() || found.isFetching, passages: found.data?.passages ?? [] };

  useEffect(() => {
    if (!toast) return;
    const timer = setTimeout(() => setToast(null), 4000);
    return () => clearTimeout(timer);
  }, [toast]);

  const items = useMemo(() => institutionItems(docs.data ?? []), [docs.data]);

  // what changed in a window of the Expediente shows here and in «Mi institución»
  const refresh = async () => {
    await qc.invalidateQueries({ queryKey: ["documents"] });
    await qc.invalidateQueries({ queryKey: ["documents-search"] });
    await qc.invalidateQueries({ queryKey: RECORDS_KEY });
  };

  async function remove(id: string) {
    setBusy(true);
    setFailure(null);
    try {
      await documentEmergencyDelete(id);
      setAsking(null);
      setToast(t.deleted);
      await refresh();
    } catch (e) {
      setFailure(toAppError(e).message);
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className="space-y-6">
      <PageHeader
        title={t.title}
        intro={t.intro}
        action={
          <Button variant="primary" onClick={() => setChoosing(true)}>
            <Icon name="upload" size={18} />
            {t.upload}
          </Button>
        }
      />

      {failure && <Alert tone="error">{failure}</Alert>}

      <DocColumn
        items={items}
        query={query}
        onQuery={setQuery}
        mode={mode}
        onMode={setMode}
        open={open}
        onToggle={(id, now) => setOpen((cur) => ({ ...cur, [id]: now }))}
        onUpload={() => setChoosing(true)}
        onOpen={(d) => d.record && setRecord(d.record)}
        asking={asking}
        onAsk={setAsking}
        onRemove={(id) => void remove(id)}
        busy={busy}
        inside={inside}
      />

      {choosing && (
        <DrawerUploadModal
          onPick={(r) => {
            setChoosing(false);
            setRecord(r);
          }}
          onClose={() => setChoosing(false)}
        />
      )}

      {record && (
        <RecordWindow
          record={record}
          onDone={(text) => setToast(text)}
          onClose={() => {
            setRecord(null);
            void refresh();
          }}
        />
      )}

      {toast && <Toast>{toast}</Toast>}
    </div>
  );
}
