import { Icon } from "../../components/icons";
import { Bar, Button, Card, Inset, RowActions, RowActionsSlot, Tag } from "../../components/ui";
import type { Tone } from "../../components/ui";
import { es } from "../../i18n/es-MX";
import type { FinanceLine, FinanceView, ProfileTotals } from "../../lib/types";
import { balanceMissing, balanceState, incomeShares, isEditable, kindTone, TONE_BG } from "./finance";

const t = es.profile;
const f = t.finance;
const money = (n: number) => n.toLocaleString("es-MX");
const peso = (n: number) => `$${money(n)}`;

type Actions = {
  busy: boolean;
  onAdd: () => void;
  onEdit: (index: number) => void;
  onRemove: (index: number) => void;
};

/** One line of money: a mark, what it is, what kind (or why it does not add up) and its yearly amount on the right. */
function MoneyRow({
  mark,
  label,
  detail,
  tag,
  amount,
  dim,
  note,
  actions,
}: {
  mark?: Tone;
  label: string;
  detail?: string;
  tag?: string;
  amount: number | null;
  dim?: boolean;
  note?: string;
  actions?: { onEdit: () => void; onRemove: () => void; busy: boolean };
}) {
  return (
    <li className="group py-2">
      <div className={`flex items-center gap-3 ${dim ? "opacity-60" : ""}`}>
        {mark && <span aria-hidden="true" className={`h-3 w-3 shrink-0 rounded-pill ${TONE_BG[mark] ?? "bg-ink-3"}`} />}
        <div className="min-w-0 flex-1">
          <b className="block break-words text-ui font-bold">{label}</b>
          {(detail || tag) && (
            <span className="mt-0.5 flex flex-wrap items-center gap-x-2 gap-y-1 text-small text-ink-3">
              {detail}
              {tag && <Tag variant="line">{tag}</Tag>}
            </span>
          )}
        </div>
        <RowActionsSlot>{actions && <RowActions onEdit={actions.onEdit} onRemove={actions.onRemove} busy={actions.busy} />}</RowActionsSlot>
        <span className="tabular min-w-[96px] shrink-0 text-right text-ui font-bold">{amount !== null ? peso(amount) : "—"}</span>
      </div>
      {note && <p className={`mt-1 text-small text-ink-3 ${mark ? "pl-6" : ""}`}>{note}</p>}
    </li>
  );
}

const periodOf = (money: FinanceView, list: "income" | "expenses", line: FinanceLine) => {
  if (line.index === null) return undefined;
  const item = money.input[list][line.index];
  return item && item.period === "monthly" && item.amount_mxn !== null ? f.perMonth(peso(item.amount_mxn)) : undefined;
};

/** Income by kind: the total, what is fixed and what varies, the bar of where it comes from and every line. */
export function IncomeCard({ money, busy, onAdd, onEdit, onRemove }: { money: FinanceView } & Actions) {
  const fin = money.finances;
  const shares = incomeShares(fin);
  return (
    <Card className="flex flex-col gap-4">
      <div className="flex flex-wrap items-center justify-between gap-3">
        <h2 className="text-heading font-bold">{t.cards.income}</h2>
        <Button size="sm" variant="secondary" onClick={onAdd}>
          <Icon name="plus" size={16} strokeWidth={2.4} />
          {t.addSource}
        </Button>
      </div>
      {fin.income.length === 0 ? (
        <p className="text-ui text-ink-3">{t.incomeEmpty}</p>
      ) : (
        <>
          <div>
            <div className="tabular text-hero font-normal tracking-tight">{peso(fin.income_annual_mxn)}</div>
            <div className="text-small text-ink-3">{t.incomeSources(fin.income.length)}</div>
          </div>
          <div className="grid grid-cols-2 gap-3">
            <Inset className="!px-4 !py-3">
              <div className="text-small font-semibold text-ink-2">{f.fixed}</div>
              <div className="tabular text-heading font-bold">{peso(fin.income_fixed_annual_mxn)}</div>
              <div className="text-caption text-ink-3">{f.fixedNote}</div>
            </Inset>
            <Inset className="!px-4 !py-3">
              <div className="text-small font-semibold text-ink-2">{f.variable}</div>
              <div className="tabular text-heading font-bold">{peso(fin.income_variable_annual_mxn)}</div>
              <div className="text-caption text-ink-3">{f.variableNote}</div>
            </Inset>
          </div>
          {shares.length > 0 && (
            <div role="img" aria-label={t.cards.income} className="flex h-3 gap-1 overflow-hidden rounded-pill">
              {shares.map((s) => (
                <span key={s.kind} className={`h-full ${TONE_BG[kindTone(s.kind)]}`} style={{ width: `${s.share * 100}%` }} />
              ))}
            </div>
          )}
          <ul className="divide-y divide-line">
            {fin.income.map((l, i) => (
              <MoneyRow
                key={i}
                mark={kindTone(l.kind)}
                label={l.label}
                detail={[f.kindShort[l.kind], periodOf(money, "income", l)].filter(Boolean).join(" · ")}
                tag={l.index === null ? f.fromRoster : !l.counted ? f.notCounted : undefined}
                amount={l.annual_mxn}
                dim={!l.counted}
                note={!l.counted ? (l.kind === "fee_estimate" ? es.issues.fee_estimate_ignored : f.notCountedGeneric) : undefined}
                actions={isEditable(l) ? { onEdit: () => onEdit(l.index), onRemove: () => onRemove(l.index), busy } : undefined}
              />
            ))}
          </ul>
        </>
      )}
    </Card>
  );
}

/**
 * Expenses: the list by concept plus the payroll the program computes. To start, the approximate yearly expense
 * (`annual_budget_mxn`) is a quick way in; the list replaces it as soon as it has one line.
 */
export function ExpensesCard({ money, totals, busy, onAdd, onEdit, onRemove, onEditEstimate }: { money: FinanceView; totals: ProfileTotals; onEditEstimate: () => void } & Actions) {
  const fin = money.finances;
  const estimate = money.input.annual_budget_mxn;
  const listed = money.input.expenses.length > 0;
  return (
    <Card className="flex flex-col gap-4">
      <div className="flex flex-wrap items-center justify-between gap-3">
        <h2 className="text-heading font-bold">{f.expenses.title}</h2>
        <Button size="sm" variant="secondary" onClick={onAdd}>
          <Icon name="plus" size={16} strokeWidth={2.4} />
          {f.expenses.add}
        </Button>
      </div>
      <div>
        <div className="tabular text-hero font-normal tracking-tight">{fin.expenses_annual_mxn !== null ? peso(fin.expenses_annual_mxn) : "—"}</div>
        <div className="text-small text-ink-3">{f.expenses.basis[fin.expenses_basis]}</div>
      </div>

      {!listed && (
        <Inset className="flex flex-col gap-3">
          <div>
            <b className="block font-bold">{f.expenses.quick.title}</b>
            <p className="mt-1 text-ui text-ink-2">{f.expenses.quick.text}</p>
          </div>
          <div className="flex flex-wrap items-center gap-3">
            {estimate !== null && <Tag tone="amber">{f.expenses.quick.value(peso(estimate))}</Tag>}
            <Button size="sm" variant={estimate === null ? "primary" : "secondary"} onClick={onEditEstimate}>
              <Icon name="pencil" size={16} />
              {estimate === null ? f.expenses.quick.action : f.expenses.quick.change}
            </Button>
          </div>
        </Inset>
      )}

      {fin.expenses.length > 0 && (
        <ul className="divide-y divide-line">
          {fin.expenses.map((l, i) => {
            const payroll = l.kind === "payroll";
            // the lines the program computes from the staff (ADR-026, ADR-027) carry their own words
            const computed = l.kind === "staff_support" ? [f.expenses.staffSupport, f.expenses.staffSupportCalc] : l.kind === "external_staff" ? [f.expenses.externalStaff, f.expenses.externalStaffCalc] : null;
            return (
              <MoneyRow
                key={i}
                label={payroll ? f.expenses.payroll : computed ? computed[0]! : l.label}
                detail={payroll ? f.expenses.payrollCalc : computed ? computed[1] : periodOf(money, "expenses", l)}
                tag={payroll || computed ? f.fromRoster : undefined}
                amount={l.annual_mxn}
                dim={!l.counted}
                note={
                  payroll
                    ? [!l.counted ? f.expenses.payrollIncluded : null, totals.benefits_assumed > 0 ? f.payroll.assumedShort(totals.benefits_assumed) : null].filter(Boolean).join(" ") || undefined
                    : undefined
                }
                actions={isEditable(l) ? { onEdit: () => onEdit(l.index), onRemove: () => onRemove(l.index), busy } : undefined}
              />
            );
          })}
        </ul>
      )}
      {listed && estimate !== null && (
        <p className="text-small text-ink-3">
          {f.expenses.quick.value(peso(estimate))}.{" "}
          <button type="button" onClick={onEditEstimate} className="font-bold text-ink underline underline-offset-4">
            {f.expenses.quick.change}
          </button>
        </p>
      )}
    </Card>
  );
}

const BALANCE_TAG: Record<string, { tone: Tone; icon: "check" | "warn" | "info" }> = {
  surplus: { tone: "green", icon: "check" },
  deficit: { tone: "amber", icon: "warn" },
  even: { tone: "sky", icon: "info" },
  unknown: { tone: "neutral", icon: "info" },
};

/** Income minus expenses in a year, as Rust computed it: surplus, deficit or a missing piece; and where the expense comes from. */
export function BalanceCard({ money }: { money: FinanceView }) {
  const fin = money.finances;
  const state = balanceState(fin);
  const b = fin.balance_annual_mxn;
  const tag = BALANCE_TAG[state]!;
  const out = fin.expenses_annual_mxn ?? 0;
  const top = Math.max(fin.income_annual_mxn, out, 1);
  const text = {
    surplus: f.balance.surplusText,
    deficit: f.balance.deficitText,
    even: f.balance.evenText,
    unknown: { income: f.balance.missingIncome, expenses: f.balance.missingExpenses, both: f.balance.missingBoth }[balanceMissing(fin)],
  }[state];
  return (
    <Card className="flex flex-col gap-4">
      <div className="flex flex-wrap items-center justify-between gap-3">
        <h2 className="text-heading font-bold">{f.balance.title}</h2>
        <Tag tone={tag.tone} icon={tag.icon}>
          {f.balance[state]}
        </Tag>
      </div>
      <div>
        <div className="tabular text-hero font-normal tracking-tight">{b === null ? "—" : `${b < 0 ? "−" : ""}${peso(Math.abs(b))}`}</div>
        <div className="text-small text-ink-2">{text}</div>
      </div>
      {b !== null && (
        <div className="space-y-3">
          <div>
            <div className="mb-1 flex justify-between text-small font-semibold text-ink-2">
              <span>{f.balance.income}</span>
              <span className="tabular">{peso(fin.income_annual_mxn)}</span>
            </div>
            <Bar percent={(fin.income_annual_mxn / top) * 100} label={f.balance.income} tone="green" />
          </div>
          <div>
            <div className="mb-1 flex justify-between text-small font-semibold text-ink-2">
              <span>{f.balance.expenses}</span>
              <span className="tabular">{peso(out)}</span>
            </div>
            <Bar percent={(out / top) * 100} label={f.balance.expenses} tone={state === "deficit" ? "amber" : "ink"} />
          </div>
        </div>
      )}
      {fin.expenses_basis !== "unknown" && <p className="text-small text-ink-3">{fin.expenses_basis === "list" ? f.balance.fromList : f.balance.fromEstimate}</p>}
    </Card>
  );
}
