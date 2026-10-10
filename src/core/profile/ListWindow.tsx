import { useQuery, useQueryClient } from "@tanstack/react-query";
import { useEffect, useState } from "react";
import { FormRenderer } from "../../components/FormRenderer";
import { Icon } from "../../components/icons";
import { Alert, Button, MaskedField, Modal } from "../../components/ui";
import { es } from "../../i18n/es-MX";
import {
  institutionClabeReveal,
  institutionListDelete,
  institutionListForm,
  institutionListSave,
  institutionListSetActive,
  institutionLists,
  toAppError,
} from "../../lib/tauri";
import type { FormValues, InstitutionLists, ListKind, ProfileIssue } from "../../lib/types";
import { useSession } from "../access/session";
import { OVERVIEW_KEY } from "./gaps";

const t = es.profile.lists;

export const LISTS_KEY = ["institution", "lists"];

export const useInstitutionLists = () => useQuery({ queryKey: LISTS_KEY, queryFn: institutionLists });

/** A row to edit (its id and whether it is still in the board, or the account still open), or `null` to add one. */
export type ListRowRef = { id: string; active: boolean } | null;

/**
 * The window of a person of the board or a bank account (ADR-033 §3). The fields come from Rust, which checks them
 * and keeps them; nothing of this reaches the AI. A saved account shows its CLABE covered and does not ask it again;
 * «Mostrar» brings it in full, and the audit log keeps that it was shown.
 */
export function ListWindow({ kind, row, onDone, onClose }: { kind: ListKind; row: ListRowRef; onDone: (message: string) => void; onClose: () => void }) {
  const qc = useQueryClient();
  // without the right to delete, a deletion becomes a request for the administrator (ADR-028)
  const access = useSession();
  const form = useQuery({ queryKey: ["institution", "list-form", kind, row?.id ?? null], queryFn: () => institutionListForm(kind, row?.id ?? null), gcTime: 0 });
  const [values, setValues] = useState<FormValues | null>(null);
  const [issues, setIssues] = useState<ProfileIssue[]>([]);
  const [busy, setBusy] = useState(false);
  const [failure, setFailure] = useState<string | null>(null);
  const [clabe, setClabe] = useState<string | null>(null);
  const [asking, setAsking] = useState(false);

  // what is saved is the starting point, once
  useEffect(() => {
    if (form.data && values === null) setValues(form.data.values);
  }, [form.data, values]);

  const board = kind === "board_member";
  const title = board ? (row ? t.editMember : t.newMember) : row ? t.editAccount : t.newAccount;

  async function run(action: () => Promise<InstitutionLists | null>, message: string) {
    setBusy(true);
    setFailure(null);
    try {
      const lists = await action();
      if (lists) {
        qc.setQueryData(LISTS_KEY, lists);
        void qc.invalidateQueries({ queryKey: OVERVIEW_KEY });
        onDone(message);
        onClose();
      }
    } catch (e) {
      setFailure(toAppError(e).message);
    } finally {
      setBusy(false);
    }
  }

  const save = () =>
    run(async () => {
      if (!values) return null;
      setIssues([]);
      const out = await institutionListSave(kind, row?.id ?? null, values);
      if (out.status === "invalid") {
        setIssues(out.issues);
        return null;
      }
      return out.lists;
    }, es.common.saved);

  const errors = Object.fromEntries(issues.map((i) => [i.field, es.issues[i.code] ?? es.issues.code_unknown]));
  const unplaced = issues.filter((i) => !form.data?.spec.sections.some((s) => s.fields.some((f) => f.id === i.field)));
  const toggle = row && (board ? (row.active ? t.markLeft : t.markBack) : row.active ? t.markClosed : t.markOpen);

  return (
    <>
      <Modal
        title={title}
        onClose={onClose}
        footer={
          <>
            {row && (
              <Button variant="plain" className="mr-auto !text-red-ink" disabled={busy} onClick={() => setAsking(true)}>
                <Icon name="trash" size={16} />
                {t.remove}
              </Button>
            )}
            {row && toggle && (
              <Button disabled={busy} onClick={() => run(() => institutionListSetActive(kind, row.id, !row.active), es.common.saved)}>
                {toggle}
              </Button>
            )}
            <Button onClick={onClose}>{es.common.cancel}</Button>
            <Button variant="primary" disabled={busy || !values} onClick={() => void save()}>
              {busy ? es.common.saving : es.common.save}
            </Button>
          </>
        }
      >
        {form.isError && <Alert tone="error">{toAppError(form.error).message}</Alert>}
        {row && form.data?.masked && (
          <MaskedField
            label={t.clabe}
            value={clabe ?? form.data.masked}
            hint={clabe ? t.shownNote : t.clabeKept}
            actions={
              clabe === null && (
                <Button size="sm" variant="plain" disabled={busy} onClick={() => void institutionClabeReveal(row.id).then(setClabe, (e) => setFailure(toAppError(e).message))}>
                  {t.show}
                </Button>
              )
            }
          />
        )}
        {form.data && values && <FormRenderer spec={form.data.spec} values={values} onChange={setValues} errors={errors} />}
        {(failure || unplaced.length > 0) && <Alert tone="error">{failure ?? unplaced.map((i) => es.issues[i.code] ?? es.issues.code_unknown).join(" ")}</Alert>}
      </Modal>
      {asking && row && (
        <Modal
          title={t.remove}
          onClose={() => setAsking(false)}
          footer={
            <>
              <Button onClick={() => setAsking(false)}>{es.common.cancel}</Button>
              <Button variant="danger" disabled={busy} onClick={() => run(() => institutionListDelete(kind, row.id), access?.can("delete") ? t.removed : t.requested)}>
                {t.removeYes}
              </Button>
            </>
          }
        >
          <p className="text-ink-2">{t.removeSure}</p>
        </Modal>
      )}
    </>
  );
}
