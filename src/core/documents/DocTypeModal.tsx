import { useState } from "react";
import { Button, Modal, Select } from "../../components/ui";
import { es } from "../../i18n/es-MX";
import { docTypeLabel } from "./documentsModel";
import type { DocItem } from "./documentsModel";

const t = es.documents;

/** The window where the person says what a document is (IA6): a document of the record, or «otro, o no lo sé». */
export function DocTypeModal({ doc, busy, onSave, onClose }: { doc: DocItem; busy: boolean; onSave: (docType: string | null) => void; onClose: () => void }) {
  const [value, setValue] = useState(doc.docType ?? "");
  const options: [string, string][] = [["", t.typeNone], ...Object.keys(es.profile.records.types).map((code): [string, string] => [code, docTypeLabel(code)])];
  return (
    <Modal
      title={t.typeTitle}
      onClose={onClose}
      footer={
        <>
          <Button onClick={onClose} disabled={busy}>
            {es.common.cancel}
          </Button>
          <Button variant="primary" disabled={busy} onClick={() => onSave(value || null)}>
            {busy ? es.common.saving : es.common.save}
          </Button>
        </>
      }
    >
      <div className="space-y-4">
        <p className="text-ui text-ink-2">
          <b className="font-bold text-ink">{doc.name}</b>
        </p>
        <p className="text-ui text-ink-2">{t.typeHint}</p>
        <Select label={t.typeLabel} value={value} onChange={(e) => setValue(e.target.value)} options={options} />
      </div>
    </Modal>
  );
}
