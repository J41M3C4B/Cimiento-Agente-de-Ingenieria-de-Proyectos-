import { useQuery, useQueryClient } from "@tanstack/react-query";
import { useMemo, useState } from "react";
import type { KeyboardEvent } from "react";
import { Icon } from "../../components/icons";
import { Alert, Button, Modal, RowActions, Select, TextArea, TextInput, THead } from "../../components/ui";
import { es } from "../../i18n/es-MX";
import { rosterEntryDelete, rosterEntrySave, rosterFieldDelete, rosterFieldSave, rosterOverview, toAppError } from "../../lib/tauri";
import type { Entity, FieldKind, ProfileTotals, ProfileView, RosterEntry, RosterField, RosterOverview } from "../../lib/types";

const r = es.roster;
const money = (n: number) => n.toLocaleString("es-MX");
const peso = (n: number) => `$${money(n)}`;
const PAGE = 10;
const THIS_YEAR = new Date().getFullYear();
const YEARS: [string, string][] = Array.from({ length: THIS_YEAR - 1959 }, (_, i) => String(THIS_YEAR - i)).map((y) => [y, y]);

/** How the form is grouped: the fields the app knows go under their heading, the person's own ones under «Otros datos». */
const GROUPS: Record<Entity, [keyof typeof r.groups, string[]][]> = {
  staff: [
    ["identity", ["full_name"]],
    ["job", ["role", "contract", "shift", "start_year"]],
    ["pay", ["monthly_salary_mxn", "paid"]],
    ["contact", ["phone", "email"]],
  ],
  beneficiary: [
    ["identity", ["full_name", "category", "age"]],
    ["care", ["dependency", "entry_year"]],
    ["fee", ["monthly_fee_mxn"]],
    ["contact", ["phone", "email"]],
  ],
};

/** What a stored value looks like in the table. */
function show(f: RosterField, value: string | undefined): string {
  if (!value) return "—";
  if (f.kind === "yesno") return value === "yes" ? r.form.yes : r.form.no;
  if (f.kind === "money") return peso(Number(value));
  if (f.kind === "select") return f.options.find((o) => o.value === value)?.label ?? value;
  return value;
}

/** One field of the form, drawn by its kind. */
function FieldControl({ f, value, error, first, onChange }: { f: RosterField; value: string; error?: string; first?: boolean; onChange: (v: string) => void }) {
  const label = f.required ? `${f.title} *` : f.title;
  if (f.kind === "yesno") {
    const on = value !== "no";
    return (
      <label className="flex min-h-[40px] cursor-pointer items-center gap-3 self-end text-[14px] font-medium">
        <input type="checkbox" className="peer sr-only" checked={on} onChange={(e) => onChange(e.target.checked ? "yes" : "no")} />
        <span className="relative h-6 w-10 shrink-0 rounded-full bg-stone-400 transition-colors after:absolute after:left-0.5 after:top-0.5 after:h-5 after:w-5 after:rounded-full after:bg-white after:shadow-sm after:transition-transform peer-checked:bg-blue-800 peer-checked:after:translate-x-4 peer-focus-visible:ring-[3px] peer-focus-visible:ring-blue-100" />
        {f.title}
      </label>
    );
  }
  if (f.kind === "select" || f.kind === "year") {
    const base: [string, string][] = f.kind === "year" ? YEARS : f.options.map((o) => [o.value, o.label]);
    const options: [string, string][] = [["", r.form.select], ...base];
    if (value && !base.some(([v]) => v === value)) options.push([value, value]); // a value an option once had
    return <Select label={label} error={error} options={options} value={value} onChange={(e) => onChange(e.target.value)} autoFocus={first} />;
  }
  const numeric = f.kind === "number" || f.kind === "money";
  return (
    <TextInput
      label={label}
      error={error}
      value={value}
      onChange={(e) => onChange(e.target.value)}
      prefix={f.kind === "money" ? "$" : undefined}
      inputMode={numeric ? "numeric" : f.kind === "phone" ? "tel" : f.kind === "email" ? "email" : undefined}
      type={f.kind === "email" ? "email" : undefined}
      autoFocus={first}
      className={f.kind === "text" ? "col-span-2" : ""}
    />
  );
}

/** The window to add or edit one person: its fields grouped, «Guardar y agregar otra» for filling in a list. */
function PersonModal({
  title, fields, entity, values, errors, notice, busy, editing, onChange, onSave, onClose,
}: {
  title: string; fields: RosterField[]; entity: Entity; values: Record<string, string>; errors: Record<string, string>;
  notice: string | null; busy: boolean; editing: boolean; onChange: (key: string, v: string) => void;
  onSave: (another: boolean) => void; onClose: () => void;
}) {
  const known = new Set(GROUPS[entity].flatMap(([, keys]) => keys));
  const groups = [
    ...GROUPS[entity].map(([id, keys]) => [r.groups[id], fields.filter((f) => keys.includes(f.key))] as [string, RosterField[]]),
    [r.groups.other, fields.filter((f) => !known.has(f.key))] as [string, RosterField[]],
  ].filter(([, list]) => list.length > 0);
  const value = (f: RosterField) => values[f.key] ?? (f.kind === "yesno" && !editing ? "yes" : "");

  // this window is not inside any form: Enter adds the person
  function onKey(e: KeyboardEvent) {
    if (e.key === "Enter" && e.target instanceof HTMLInputElement && e.target.type !== "checkbox") {
      e.preventDefault();
      onSave(false);
    }
  }

  return (
    <Modal
      title={title}
      onClose={onClose}
      size="lg"
      footer={
        <>
          <Button onClick={onClose}>{r.form.cancel}</Button>
          {!editing && (
            <Button disabled={busy} onClick={() => onSave(true)}>
              {r.form.saveAndAnother}
            </Button>
          )}
          <Button variant="primary" disabled={busy} onClick={() => onSave(false)}>
            {editing ? r.form.saveChanges : es.common.save}
          </Button>
        </>
      }
    >
      <div onKeyDown={onKey} className="space-y-6">
        {groups.map(([heading, list], gi) => (
          <fieldset key={heading}>
            <legend className="mb-3 w-full border-b border-stone-200 pb-1.5 text-[11px] font-semibold uppercase tracking-[0.1em] text-stone-600">{heading}</legend>
            <div className="grid grid-cols-2 gap-x-4 gap-y-3.5">
              {list.map((f, fi) => (
                <FieldControl key={f.key} f={f} first={gi === 0 && fi === 0} value={value(f)} error={errors[f.key]} onChange={(v) => onChange(f.key, v)} />
              ))}
            </div>
          </fieldset>
        ))}
        {notice && <Alert tone="error">{notice}</Alert>}
      </div>
    </Modal>
  );
}

/** Add a field of their own, or change the title and the options of one. */
function FieldsDialog({ entity, fields, onFields, onClose }: { entity: Entity; fields: RosterField[]; onFields: (f: RosterField[]) => void; onClose: () => void }) {
  const [editing, setEditing] = useState<{ key: string | null; title: string; kind: FieldKind; options: string } | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const current = editing?.key ? fields.find((f) => f.key === editing.key) : undefined;

  async function run(job: () => Promise<RosterField[]>) {
    setBusy(true);
    setError(null);
    try {
      onFields(await job());
      setEditing(null);
    } catch (e) {
      setError(toAppError(e).message);
    } finally {
      setBusy(false);
    }
  }

  return (
    <Modal
      title={r.fields.title}
      onClose={onClose}
      footer={
        editing ? (
          <>
            <Button onClick={() => setEditing(null)}>{r.fields.back}</Button>
            <Button
              variant="primary"
              disabled={busy || editing.title.trim() === ""}
              onClick={() => run(() => rosterFieldSave(entity, { key: editing.key, title: editing.title, kind: editing.kind, options: editing.options.split("\n") }))}
            >
              {r.fields.save}
            </Button>
          </>
        ) : (
          <>
            <Button onClick={onClose}>{es.common.close}</Button>
            <Button variant="primary" onClick={() => setEditing({ key: null, title: "", kind: "text", options: "" })}>
              <Icon name="plus" size={16} strokeWidth={2.4} />
              {r.fields.add}
            </Button>
          </>
        )
      }
    >
      {error && <Alert tone="error">{error}</Alert>}
      {!editing ? (
        <>
          <p className="text-stone-700">{r.fields.intro}</p>
          <ul className="divide-y divide-stone-200 rounded-lg border border-stone-200">
            {fields.map((f) => (
              <li key={f.key} className="group flex items-center gap-3 px-4 py-2.5">
                <div className="min-w-0 flex-1">
                  <p className="truncate font-medium">{f.title}</p>
                  <p className="text-[12.5px] text-stone-600">
                    {r.fields.kinds[f.kind]}
                    {f.kind === "select" && ` · ${f.options.length}`}
                    {f.builtin && ` · ${r.fields.builtin}`}
                  </p>
                </div>
                <Button size="sm" variant="plain" aria-label={r.fields.edit} title={r.fields.edit} onClick={() => setEditing({ key: f.key, title: f.title, kind: f.kind, options: f.options.map((o) => o.label).join("\n") })} className="!px-2">
                  <Icon name="pencil" size={16} />
                </Button>
                {!f.builtin && (
                  <Button size="sm" variant="plain" aria-label={es.common.remove} title={es.common.remove} disabled={busy} onClick={() => run(() => rosterFieldDelete(entity, f.key))} className="!px-2 hover:!text-red-800">
                    <Icon name="trash" size={16} />
                  </Button>
                )}
              </li>
            ))}
          </ul>
        </>
      ) : (
        <>
          <TextInput label={r.fields.name} placeholder={r.fields.namePlaceholder} value={editing.title} autoFocus onChange={(e) => setEditing({ ...editing, title: e.target.value })} />
          {!current?.builtin && (
            <Select
              label={r.fields.type}
              options={(["text", "select", "number"] as const).map((k) => [k, r.fields.kinds[k]] as [string, string])}
              value={editing.kind}
              onChange={(e) => setEditing({ ...editing, kind: e.target.value as FieldKind })}
            />
          )}
          {editing.kind === "select" &&
            (current?.locked_options ? (
              <p className="rounded-lg bg-stone-50 px-4 py-3 text-stone-700">{r.fields.lockedOptions}</p>
            ) : (
              <TextArea label={r.fields.options} hint={r.fields.optionsHelp} value={editing.options} onChange={(e) => setEditing({ ...editing, options: e.target.value })} />
            ))}
        </>
      )}
    </Modal>
  );
}

/**
 * One tab of the roster: the people registered, in a table with search and a filter, and a button to add one in a
 * window. The records stay in this computer; the profile only receives what they add up to (ADR-020).
 */
export function RosterTab({ entity, onProfile }: { entity: Entity; onProfile: (p: ProfileView) => void }) {
  const text = entity === "staff" ? r.staff : r.beneficiary;
  const qc = useQueryClient();
  const overview = useQuery({ queryKey: ["roster", entity], queryFn: () => rosterOverview(entity) });

  const [panel, setPanel] = useState<{ id: string | null; values: Record<string, string> } | null>(null);
  const [errors, setErrors] = useState<Record<string, string>>({});
  const [notice, setNotice] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [search, setSearch] = useState("");
  const [filter, setFilter] = useState("");
  const [page, setPage] = useState(0);
  const [configuring, setConfiguring] = useState(false);
  const [round, setRound] = useState(0);

  const fields = overview.data?.fields ?? [];
  const entries = overview.data?.entries ?? [];
  const columns = fields.slice(0, 5);
  // the filter is the first selector of the form (the position of staff, the group of people served)
  const filterField = fields.find((f) => f.kind === "select" && f.options.length > 1);

  const shown = useMemo(() => {
    const q = search.trim().toLowerCase();
    return entries.filter(
      (e) =>
        (!filterField || !filter || e.data[filterField.key] === filter) &&
        (!q || fields.some((f) => show(f, e.data[f.key]).toLowerCase().includes(q))),
    );
  }, [entries, fields, search, filter, filterField]);
  const pages = Math.max(1, Math.ceil(shown.length / PAGE));
  const current = Math.min(page, pages - 1);

  function apply(change: { entries: RosterEntry[]; totals: ProfileTotals; profile: ProfileView | null }) {
    qc.setQueryData<RosterOverview>(["roster", entity], (old) => old && { ...old, entries: change.entries, totals: change.totals });
    // the totals count both lists: the other tab shows them too
    qc.setQueryData<RosterOverview>(["roster", entity === "staff" ? "beneficiary" : "staff"], (old) => old && { ...old, totals: change.totals });
    if (change.profile) onProfile(change.profile);
  }

  function open(entry?: RosterEntry) {
    setPanel({ id: entry?.id ?? null, values: entry ? { ...entry.data } : {} });
    setErrors({});
    setNotice(null);
  }

  async function save(another: boolean) {
    if (!panel) return;
    const val = (f: RosterField) => (panel.values[f.key] ?? (f.kind === "yesno" && !panel.id ? "yes" : "")).trim();
    const missing: Record<string, string> = {};
    for (const f of fields) {
      const v = val(f);
      if (f.required && !v) missing[f.key] = r.form.requiredMissing;
      else if (v && (f.kind === "number" || f.kind === "money") && !/^\d+$/.test(v)) missing[f.key] = es.issues.not_a_number;
    }
    setErrors(missing);
    setNotice(null);
    if (Object.keys(missing).length > 0) return;
    setBusy(true);
    try {
      const data = Object.fromEntries(fields.map((f) => [f.key, val(f)]).filter(([, v]) => v !== ""));
      apply(await rosterEntrySave(entity, panel.id, data));
      // «guardar y agregar otra»: the window stays, empty, for the next one
      if (another && !panel.id) {
        setPanel({ id: null, values: {} });
        setRound((n) => n + 1);
      }
      else setPanel(null);
    } catch (e) {
      setNotice(toAppError(e).message);
    } finally {
      setBusy(false);
    }
  }

  async function remove(id: string) {
    setBusy(true);
    try {
      apply(await rosterEntryDelete(entity, id));
    } catch (e) {
      setNotice(toAppError(e).message);
    } finally {
      setBusy(false);
    }
  }

  const rows = shown.slice(current * PAGE, current * PAGE + PAGE);
  return (
    <div className="space-y-4">
      <div className="flex flex-wrap items-center gap-2.5">
        <label className="relative block w-full max-w-[260px]">
          <span className="sr-only">{text.search}</span>
          <Icon name="search" size={16} className="pointer-events-none absolute left-3 top-1/2 -translate-y-1/2 text-stone-600" />
          <input
            value={search}
            onChange={(e) => {
              setSearch(e.target.value);
              setPage(0);
            }}
            placeholder={text.search}
            className="min-h-[34px] w-full rounded-lg border border-stone-300 bg-white pl-9 pr-3 text-[14px] placeholder:text-stone-500 hover:border-stone-400 focus-visible:border-stone-900 focus-visible:outline-none focus-visible:ring-[3px] focus-visible:ring-stone-200"
          />
        </label>
        {filterField && (
          <Select
            label={filterField.title}
            hideLabel
            value={filter}
            onChange={(e) => {
              setFilter(e.target.value);
              setPage(0);
            }}
            options={[["", `${filterField.title}: ${r.table.all}`], ...filterField.options.map((o) => [o.value, o.label] as [string, string])]}
            className="min-w-[200px] [&_select]:min-h-[34px]"
          />
        )}
        <div className="ml-auto flex gap-2">
          <Button size="sm" variant="plain" onClick={() => setConfiguring(true)}>
            <Icon name="sliders" size={15} />
            {r.form.configure}
          </Button>
          <Button size="sm" variant="primary" disabled={!overview.isSuccess} onClick={() => open()}>
            <Icon name="plus" size={15} strokeWidth={2.4} />
            {text.add}
          </Button>
        </div>
      </div>

      {entries.length === 0 ? (
        <div className="rounded-2xl bg-stone-50 px-6 py-12 text-center">
          <p className="mx-auto max-w-sm text-stone-700">{text.empty}</p>
          <Button className="mt-4" variant="primary" disabled={!overview.isSuccess} onClick={() => open()}>
            <Icon name="plus" size={16} strokeWidth={2.4} />
            {text.add}
          </Button>
        </div>
      ) : (
        <div className="overflow-x-auto">
          <table className="w-full min-w-[640px] border-collapse text-left text-[14px]">
            <THead columns={[...columns.map((f) => ({ key: f.key, title: f.title })), { key: "actions", title: "" }]} />
            <tbody>
              {rows.map((e) => (
                <tr key={e.id} className="group border-b border-stone-100 transition-colors hover:bg-stone-50">
                  {columns.map((f, i) => (
                    <td key={f.key} className={`max-w-[240px] px-3 py-3 first:pl-0 ${f.kind === "money" || f.kind === "number" ? "tabular-nums" : ""} ${e.data[f.key] ? "" : "text-stone-300"}`}>
                      {i === 0 ? (
                        <button type="button" onClick={() => open(e)} className="block text-left font-medium hover:underline">
                          <span className="line-clamp-1">{show(f, e.data[f.key])}</span>
                        </button>
                      ) : (
                        <span className="line-clamp-1">{show(f, e.data[f.key])}</span>
                      )}
                    </td>
                  ))}
                  <td className="w-24 px-1 py-1.5 text-right">
                    <RowActions onEdit={() => open(e)} onRemove={() => remove(e.id)} busy={busy} />
                  </td>
                </tr>
              ))}
              {shown.length === 0 && (
                <tr>
                  <td colSpan={columns.length + 1} className="px-3 py-6 text-stone-700">
                    {r.table.noResults}
                  </td>
                </tr>
              )}
            </tbody>
          </table>
          <div className="flex items-center justify-between gap-3 px-0 py-3 text-[13px] text-stone-500">
            <span>{r.table.count(shown.length)}</span>
            {pages > 1 && (
              <span className="flex items-center gap-2">
                <Button size="sm" disabled={current === 0} onClick={() => setPage(current - 1)}>
                  {r.table.previous}
                </Button>
                <span className="tabular-nums">{r.table.page(current + 1, pages)}</span>
                <Button size="sm" disabled={current >= pages - 1} onClick={() => setPage(current + 1)}>
                  {r.table.next}
                </Button>
              </span>
            )}
          </div>
        </div>
      )}
      {notice && !panel && <Alert tone="error">{notice}</Alert>}

      {panel && (
        <PersonModal
          key={round}
          title={panel.id ? text.editing : text.add}
          fields={fields}
          entity={entity}
          values={panel.values}
          errors={errors}
          notice={notice}
          busy={busy}
          editing={panel.id !== null}
          onChange={(key, v) => setPanel((p) => p && { ...p, values: { ...p.values, [key]: v } })}
          onSave={save}
          onClose={() => setPanel(null)}
        />
      )}
      {configuring && (
        <FieldsDialog
          entity={entity}
          fields={fields}
          onFields={(next) => qc.setQueryData<RosterOverview>(["roster", entity], (old) => old && { ...old, fields: next })}
          onClose={() => setConfiguring(false)}
        />
      )}
    </div>
  );
}
