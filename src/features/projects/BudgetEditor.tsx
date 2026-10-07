import { useState } from "react";
import { QuarantineDialog } from "../../components/QuarantineDialog";
import { AddSlot, Alert, Button, Inset, Modal, RowActions, Select, Switch, Tag, TextInput, THead } from "../../components/ui";
import { es } from "../../i18n/es-MX";
import { formatMxn, toNumber } from "../../lib/format";
import { budgetConfirm, budgetDeleteItem, budgetSaveItem, toAppError } from "../../lib/tauri";
import type { BudgetItemInput, BudgetItemView, Decision, DraftingView, Funder, QuarantineReport } from "../../lib/types";

const t = es.drafting;
const funderOptions = Object.entries(t.funders) as [string, string][];

interface Form {
  description: string;
  category: string;
  quantity: string;
  unit: string;
  price: string;
  vatIncluded: boolean;
  funder: Funder;
  administrative: boolean;
}

const empty: Form = { description: "", category: "", quantity: "1", unit: "", price: "", vatIncluded: false, funder: "requested", administrative: false };

const fromItem = (i: BudgetItemView): Form => ({
  description: i.description,
  category: i.category,
  quantity: String(i.quantity),
  unit: i.unit ?? "",
  price: i.unit_price_mxn > 0 ? String(i.unit_price_mxn) : "",
  vatIncluded: i.vat_included,
  funder: i.funded_by,
  administrative: i.administrative,
});

const asInput = (id: string | null, f: Form): BudgetItemInput => ({
  id,
  category: f.category,
  description: f.description,
  quantity: toNumber(f.quantity),
  unit: f.unit.trim() || null,
  unit_price_mxn: toNumber(f.price),
  vat_included: f.vatIncluded,
  funded_by: f.funder,
  administrative: f.administrative,
});

/** The cost of a line, written right in the table: it is saved when the person leaves the box. */
function PriceBox({ item, disabled, onSave }: { item: BudgetItemView; disabled: boolean; onSave: (price: number) => void }) {
  const initial = item.unit_price_mxn > 0 ? String(item.unit_price_mxn) : "";
  const [text, setText] = useState(initial);
  const bad = text.trim() !== "" && !Number.isFinite(toNumber(text));
  const commit = () => {
    if (text === initial || bad || text.trim() === "") return;
    onSave(toNumber(text));
  };
  return (
    <TextInput
      label={`${t.cost}: ${item.description}`}
      hideLabel
      prefix="$"
      inputMode="decimal"
      placeholder="0.00"
      value={text}
      disabled={disabled}
      error={bad ? t.item.priceHint : undefined}
      className="min-w-[8.5rem]"
      onChange={(e) => setText(e.target.value)}
      onBlur={commit}
      onKeyDown={(e) => {
        if (e.key === "Enter") {
          e.preventDefault();
          commit();
        }
      }}
    />
  );
}

/**
 * The budget as a table: the assistant proposes the lines from what the person already told it, and the person only
 * writes the cost of each one. The program adds everything up (IVA, totals, counterpart). A line can still be
 * changed or removed, and a new one added, in a window.
 */
export function BudgetEditor({ view, onView, disabled }: { view: DraftingView; onView: (v: DraftingView) => void; disabled: boolean }) {
  const project = view.project.id;
  const [editing, setEditing] = useState<string | "new" | null>(null);
  const [form, setForm] = useState<Form>(empty);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [pending, setPending] = useState<{ input: BudgetItemInput; report: QuarantineReport } | null>(null);
  const working = busy || disabled;
  const { items, totals, confirmed, missing_prices: missing } = view.budget;

  async function guarded(fn: () => Promise<void>) {
    setBusy(true);
    setError(null);
    try {
      await fn();
    } catch (e) {
      setPending(null);
      setError(toAppError(e).message);
    } finally {
      setBusy(false);
    }
  }

  const save = (item: BudgetItemInput, decision?: Decision) =>
    guarded(async () => {
      const out = await budgetSaveItem(project, item, decision);
      if (out.status === "quarantine") return setPending({ input: item, report: out.report });
      setPending(null);
      setEditing(null);
      setForm(empty);
      onView(out.view);
    });

  const savePrice = (i: BudgetItemView, price: number) => save({ ...asInput(i.id, fromItem(i)), unit_price_mxn: price });

  const numbersOk = Number.isFinite(toNumber(form.quantity)) && (form.price.trim() === "" || Number.isFinite(toNumber(form.price)));
  const canSave = form.description.trim() !== "" && form.category.trim() !== "" && numbersOk;
  const open = (i: BudgetItemView | null) => {
    setEditing(i ? i.id : "new");
    setForm(i ? fromItem(i) : empty);
  };

  return (
    <div className="space-y-6">
      <p className="max-w-2xl text-ui text-ink-2">{t.budgetIntro}</p>

      {items.length === 0 ? (
        <Inset className="py-8 text-center text-ui text-ink-2">{t.emptyBudget}</Inset>
      ) : (
        <div className="overflow-x-auto">
          <table className="table min-w-[720px]">
            <THead
              columns={[
                { key: "item", title: t.budgetColumns.item },
                { key: "qty", title: t.budgetColumns.quantity },
                { key: "cost", title: t.cost },
                { key: "total", title: t.budgetColumns.total, align: "right" },
                { key: "actions", title: "" },
              ]}
            />
            <tbody>
              {items.map((i) => (
                <tr key={i.id} className="group">
                  <td>
                    <p className="flex flex-wrap items-center gap-2">
                      {i.description}
                      {i.origin === "ai_assumption" && (
                        <Tag tone="sky" variant="soft">
                          {t.proposed}
                        </Tag>
                      )}
                    </p>
                    <p className="text-small font-medium text-ink-3">
                      {i.category} · {t.funders[i.funded_by]}
                      {i.administrative ? ` · ${t.item.administrative}` : ""}
                    </p>
                  </td>
                  <td className="tabular whitespace-nowrap">
                    {i.quantity}
                    {i.unit ? ` ${i.unit}` : ""}
                  </td>
                  <td>
                    <PriceBox key={`${i.id}-${i.unit_price_mxn}`} item={i} disabled={working} onSave={(price) => savePrice(i, price)} />
                  </td>
                  <td className="tabular whitespace-nowrap text-right font-bold">{i.unit_price_mxn > 0 ? formatMxn(i.line.total) : "—"}</td>
                  <td className="w-32 text-right">
                    <RowActions onEdit={() => open(i)} onRemove={() => guarded(async () => onView(await budgetDeleteItem(project, i.id)))} busy={working} />
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}

      <AddSlot onClick={() => !working && open(null)}>{t.addItem}</AddSlot>

      {error && <Alert tone="error">{error}</Alert>}

      {items.length > 0 && (
        <dl className="grid grid-cols-2 gap-3 sm:grid-cols-3">
          {(
            [
              [t.totals.total, formatMxn(totals.total)],
              [t.totals.vat, formatMxn(totals.vat)],
              [t.totals.requested, formatMxn(totals.requested)],
              [t.totals.institution, formatMxn(totals.institution)],
              [t.totals.other, formatMxn(totals.other)],
              [t.totals.counterpart, `${totals.counterpart_percent} %`],
            ] as [string, string][]
          ).map(([label, value], n) => (
            <Inset key={label} className="!py-3">
              <dt className="text-caption font-semibold text-ink-3">{label}</dt>
              <dd className={`tabular mt-1 text-heading font-bold ${n === 0 ? "" : "text-ink-2"}`}>{value}</dd>
            </Inset>
          ))}
        </dl>
      )}

      {items.length > 0 &&
        (confirmed ? (
          <Alert tone="ok">{t.budgetConfirmed}</Alert>
        ) : (
          <div className="flex flex-wrap items-center gap-3">
            <Button variant="primary" disabled={working || missing > 0} onClick={() => guarded(async () => onView(await budgetConfirm(project)))}>
              {t.confirmBudget}
            </Button>
            {missing > 0 && <p className="text-small font-bold text-amber-ink">{t.missingCosts(missing)}</p>}
          </div>
        ))}

      {editing !== null && (
        <Modal
          title={editing === "new" ? t.addItem : t.editItem}
          onClose={() => {
            setEditing(null);
            setForm(empty);
          }}
          footer={
            <>
              <Button
                onClick={() => {
                  setEditing(null);
                  setForm(empty);
                }}
                disabled={working}
              >
                {t.cancel}
              </Button>
              <Button type="submit" form="budget-item" variant="primary" disabled={working || !canSave}>
                {t.saveItem}
              </Button>
            </>
          }
        >
          <form
            id="budget-item"
            className="space-y-6"
            onSubmit={(e) => {
              e.preventDefault();
              if (canSave) void save(asInput(editing === "new" ? null : editing, form));
            }}
          >
            <TextInput label={t.item.description} autoFocus value={form.description} onChange={(e) => setForm({ ...form, description: e.target.value })} />
            <TextInput label={t.item.category} hint={t.item.categoryHint} value={form.category} onChange={(e) => setForm({ ...form, category: e.target.value })} />
            <div className="grid gap-3 sm:grid-cols-3">
              <TextInput label={t.item.quantity} inputMode="decimal" value={form.quantity} onChange={(e) => setForm({ ...form, quantity: e.target.value })} />
              <TextInput label={t.item.unit} value={form.unit} onChange={(e) => setForm({ ...form, unit: e.target.value })} />
              <TextInput label={t.item.price} hint={t.item.priceHint} inputMode="decimal" prefix="$" value={form.price} onChange={(e) => setForm({ ...form, price: e.target.value })} />
            </div>
            <Switch label={t.item.vatIncluded} checked={form.vatIncluded} onChange={(e) => setForm({ ...form, vatIncluded: e.target.checked })} />
            <Select label={t.item.funder} options={funderOptions} value={form.funder} onChange={(e) => setForm({ ...form, funder: e.target.value as Funder })} />
            <Switch label={t.item.administrative} checked={form.administrative} onChange={(e) => setForm({ ...form, administrative: e.target.checked })} />
          </form>
        </Modal>
      )}

      {pending && (
        <QuarantineDialog
          report={pending.report}
          busy={busy}
          onRedact={() => save(pending.input, "redact")}
          onNotPersonal={() => save(pending.input, "not_personal")}
          onCancel={() => setPending(null)}
        />
      )}
    </div>
  );
}
