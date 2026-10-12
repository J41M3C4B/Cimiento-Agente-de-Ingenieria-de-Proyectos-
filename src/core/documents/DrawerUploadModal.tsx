import { useState } from "react";
import { Button, Modal, RadioCard, Select } from "../../components/ui";
import { es } from "../../i18n/es-MX";
import { recordTitle, useRecords } from "../profile/RecordWindow";
import type { RecordRef } from "../profile/RecordWindow";

const t = es.documents;

/** A new «Otro documento», as the select says it. */
const NEW_OTHER = "other:new";

/**
 * The tray (ADR-036): before a file, which drawer it goes to and which document it is in it. Today the only drawer is
 * the Expediente; the document is one of its catalog or «Otro documento». Then the window of that document of the
 * Expediente takes the file, exactly as from «Mi institución».
 */
export function DrawerUploadModal({ onPick, onClose }: { onPick: (record: RecordRef) => void; onClose: () => void }) {
  const records = useRecords();
  const [choice, setChoice] = useState("");
  const list = records.data?.records ?? [];
  const value = (r: { type: string; id: string | null }) => (r.type === "other" ? `other:${r.id}` : r.type);
  const options: [string, string][] = [
    ["", t.whichPick],
    ...list.map((r): [string, string] => [value(r), r.file ? t.hasFile(recordTitle(r)) : recordTitle(r)]),
    [NEW_OTHER, t.otherDocument],
  ];

  const next = () => {
    if (choice === NEW_OTHER) return onPick({ type: "other", id: null });
    const r = list.find((x) => value(x) === choice);
    if (r) onPick({ type: r.type, id: r.id, custom_name: r.custom_name });
  };

  return (
    <Modal
      title={t.drawerTitle}
      onClose={onClose}
      footer={
        <>
          <Button onClick={onClose}>{es.common.cancel}</Button>
          <Button variant="primary" disabled={choice === ""} onClick={next}>
            {t.next}
          </Button>
        </>
      }
    >
      <div className="space-y-6">
        <fieldset className="space-y-2">
          <legend className="field-label">{t.drawer}</legend>
          <RadioCard name="drawer" title={t.drawers.record![0]} note={t.drawers.record![1]} checked onChange={() => {}} />
        </fieldset>
        <div className="space-y-2">
          <Select label={t.whichDocument} value={choice} onChange={(e) => setChoice(e.target.value)} options={options} />
          <p className="text-small text-ink-3">{t.whichHint}</p>
        </div>
      </div>
    </Modal>
  );
}
