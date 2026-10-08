import type { ReactNode } from "react";
import { Icon } from "../../components/icons";
import { Button, Card, Inset, RowActions, RowActionsSlot, Tag, Tile } from "../../components/ui";
import type { Tone } from "../../components/ui";
import { es } from "../../i18n/es-MX";
import type { FinanceLine, FinanceView, ProfileTotals } from "../../lib/types";
import { balanceMissing, balanceState, expenseTone, incomeShares, isEditable, kindTone, percent, shareOf, TONE_BG } from "./finance";

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

/** The head of a money card: its title, the big yearly figure and what that figure is made of. */
function MoneyHead({ title, action, figure, caption }: { title: string; action: ReactNode; figure: ReactNode; caption?: string }) {
  return (
    <>
      <div className="flex flex-wrap items-center justify-between gap-3">
        <h2 className="text-heading font-bold">{title}</h2>
        {action}
      </div>
      <div>
        <div className="tabular text-hero font-normal tracking-tight">{figure}</div>
        {caption && <div className="mt-0.5 text-small text-ink-3">{caption}</div>}
      </div>
    </>
  );
}

/** One bar cut in the parts a total is made of, with its legend below (a dot, the name and its share). Words always go with the color. */
function Composition({ label, parts }: { label: string; parts: { key: string; name: string; tone: Tone; share: number }[] }) {
  return (
    <div className="space-y-2.5">
      <div role="img" aria-label={label} className="flex h-3.5 gap-1 overflow-hidden rounded-pill">
        {parts.map((p) => (
          <span key={p.key} className={`h-full ${TONE_BG[p.tone] ?? "bg-ink-3"}`} style={{ width: `${p.share * 100}%` }} />
        ))}
      </div>
      <ul className="flex flex-wrap gap-x-4 gap-y-1.5 text-small text-ink-2">
        {parts.map((p) => (
          <li key={p.key} className="flex items-center gap-1.5">
            <span aria-hidden="true" className={`h-2.5 w-2.5 rounded-pill ${TONE_BG[p.tone] ?? "bg-ink-3"}`} />
            <span className="font-semibold">{p.name}</span>
            <span className="tabular text-ink-3">{percent(p.share)}</span>
          </li>
        ))}
      </ul>
    </div>
  );
}

/** One line of money: a mark, what it is, what kind (or why it does not add up), its yearly amount and the part of the total it is. */
function MoneyRow({
  mark,
  label,
  detail,
  tag,
  amount,
  share,
  dim,
  note,
  actions,
}: {
  mark?: Tone;
  label: string;
  detail?: string;
  tag?: string;
  amount: number | null;
  share?: number | null;
  dim?: boolean;
  note?: string;
  actions?: { onEdit: () => void; onRemove: () => void; busy: boolean };
}) {
  return (
    <li className="group py-3">
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
        <span className="min-w-[96px] shrink-0 text-right">
          <span className="tabular block text-ui font-bold">{amount !== null ? peso(amount) : "—"}</span>
          {share != null && !dim && <span className="tabular block text-caption text-ink-3">{percent(share)}</span>}
        </span>
      </div>
      {note && <p className={`mt-1 text-small text-ink-3 ${mark ? "pl-6" : ""}`}>{note}</p>}
    </li>
  );
}

const periodOf = (m: FinanceView, list: "income" | "expenses", line: FinanceLine) => {
  if (line.index === null) return undefined;
  const item = m.input[list][line.index];
  return item && item.period === "monthly" && item.amount_mxn !== null ? f.perMonth(peso(item.amount_mxn)) : undefined;
};

/** Income by kind: the total, what is fixed and what varies, where it comes from (bar and legend) and every line. */
export function IncomeCard({ money: m, busy, onAdd, onEdit, onRemove }: { money: FinanceView } & Actions) {
  const fin = m.finances;
  const shares = incomeShares(fin);
  const add = (
    <Button size="sm" variant="secondary" onClick={onAdd}>
      <Icon name="plus" size={16} strokeWidth={2.4} />
      {t.addSource}
    </Button>
  );
  return (
    <Card className="flex flex-col gap-5">
      {fin.income.length === 0 ? (
        <>
          <div className="flex flex-wrap items-center justify-between gap-3">
            <h2 className="text-heading font-bold">{t.cards.income}</h2>
            {add}
          </div>
          <Inset className="flex items-center gap-4">
            <Tile icon="wallet" tone="green" />
            <p className="text-ui text-ink-2">{t.incomeEmpty}</p>
          </Inset>
        </>
      ) : (
        <>
          <MoneyHead title={t.cards.income} action={add} figure={peso(fin.income_annual_mxn)} caption={t.incomeSources(fin.income.length)} />
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
          {shares.length > 0 && <Composition label={f.legendLabel} parts={shares.map((s) => ({ key: s.kind, name: f.kindShort[s.kind] ?? s.kind, tone: kindTone(s.kind), share: s.share }))} />}
          <ul className="divide-y divide-line border-t border-line">
            {fin.income.map((l, i) => (
              <MoneyRow
                key={i}
                mark={kindTone(l.kind)}
                label={l.label}
                detail={[f.kindShort[l.kind], periodOf(m, "income", l)].filter(Boolean).join(" · ")}
                tag={l.index === null ? f.fromRoster : !l.counted ? f.notCounted : undefined}
                amount={l.annual_mxn}
                share={l.counted ? shareOf(l.annual_mxn, fin.income_annual_mxn) : null}
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
 * Expenses: the list by concept plus the payroll the program computes. To start, the approximate yearly expense is a
 * quick way in (it has its own window); the list replaces it as soon as it has one line.
 */
export function ExpensesCard({ money: m, totals, busy, onAdd, onEdit, onRemove, onEditEstimate }: { money: FinanceView; totals: ProfileTotals; onEditEstimate: () => void } & Actions) {
  const fin = m.finances;
  const estimate = m.input.annual_budget_mxn;
  const listed = m.input.expenses.length > 0;
  const total = fin.expenses_annual_mxn;
  const parts = fin.expenses
    .filter((l) => l.counted && (l.annual_mxn ?? 0) > 0)
    .map((l, i) => ({ key: `${l.kind}-${i}`, name: l.kind === "payroll" ? f.expenses.payrollShort : l.kind === "staff_support" ? f.expenses.staffSupport : l.kind === "external_staff" ? f.expenses.externalStaff : l.label, tone: expenseTone(l.kind), share: shareOf(l.annual_mxn, total) ?? 0 }))
    .filter((p) => p.share > 0);
  return (
    <Card className="flex flex-col gap-5">
      <MoneyHead
        title={f.expenses.title}
        action={
          <Button size="sm" variant="secondary" onClick={onAdd}>
            <Icon name="plus" size={16} strokeWidth={2.4} />
            {f.expenses.add}
          </Button>
        }
        figure={total !== null ? peso(total) : "—"}
        caption={f.expenses.basis[fin.expenses_basis]}
      />

      {!listed && (
        <Inset className="flex flex-col gap-4 !p-5">
          <div className="flex items-start gap-3">
            <Tile icon="pencil" tone="amber" />
            <div className="min-w-0">
              <b className="block font-bold">{estimate === null ? f.expenses.quick.title : f.expenses.quick.valueTitle}</b>
              {estimate !== null ? (
                <p className="tabular mt-1 flex flex-wrap items-baseline gap-x-2 text-heading font-bold">
                  ≈ {peso(estimate)}
                  <small className="text-small font-medium text-ink-3">{f.expenses.perYear}</small>
                </p>
              ) : (
                <p className="mt-1 text-ui text-ink-2">{f.expenses.quick.text}</p>
              )}
            </div>
            {estimate !== null && (
              <Tag tone="amber" variant="soft" className="ml-auto">
                {f.expenses.approxTag}
              </Tag>
            )}
          </div>
          <div>
            <Button size="sm" variant={estimate === null ? "primary" : "secondary"} onClick={onEditEstimate}>
              <Icon name="pencil" size={16} />
              {estimate === null ? f.expenses.quick.action : f.expenses.quick.change}
            </Button>
          </div>
        </Inset>
      )}

      {parts.length > 0 && <Composition label={f.expenses.legendLabel} parts={parts} />}

      {fin.expenses.length > 0 && (
        <ul className="divide-y divide-line border-t border-line">
          {fin.expenses.map((l, i) => {
            const payroll = l.kind === "payroll";
            // the lines the program computes from the staff (ADR-026, ADR-027) carry their own words
            const computed = l.kind === "staff_support" ? [f.expenses.staffSupport, f.expenses.staffSupportCalc] : l.kind === "external_staff" ? [f.expenses.externalStaff, f.expenses.externalStaffCalc] : null;
            return (
              <MoneyRow
                key={i}
                mark={expenseTone(l.kind)}
                label={payroll ? f.expenses.payroll : computed ? computed[0]! : l.label}
                detail={payroll ? f.expenses.payrollCalc : computed ? computed[1] : periodOf(m, "expenses", l)}
                tag={payroll || computed ? f.fromRoster : undefined}
                amount={l.annual_mxn}
                share={l.counted ? shareOf(l.annual_mxn, total) : null}
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

/** One side of the scale: its name, a bar as long as its amount and, on the shorter side, a dashed stretch for what is missing or left over. */
function ScaleRow({ label, amount, top, tone, gap }: { label: string; amount: number; top: number; tone: Tone; gap?: { amount: number; tone: Tone; text: string } }) {
  return (
    <div className="space-y-1.5">
      <div className="flex items-baseline justify-between gap-3 text-small font-semibold text-ink-2">
        <span>{label}</span>
        <span className="tabular text-ui font-bold text-ink">{peso(amount)}</span>
      </div>
      <div className="flex h-4 overflow-hidden rounded-pill bg-inset" role="img" aria-label={`${label}: ${peso(amount)}`}>
        <span className={`h-full rounded-pill ${TONE_BG[tone] ?? "bg-ink"}`} style={{ width: `${(amount / top) * 100}%` }} />
        {gap && gap.amount > 0 && (
          <span className={`ml-1 h-full rounded-pill border-2 border-dashed ${gap.tone === "amber" ? "border-amber" : "border-green"}`} style={{ width: `calc(${(gap.amount / top) * 100}% - 4px)` }} />
        )}
      </div>
      {gap && gap.amount > 0 && <div className={`text-small font-bold ${gap.tone === "amber" ? "text-amber-ink" : "text-green-ink"}`}>{gap.text}</div>}
    </div>
  );
}

/** Income minus expenses in a year, as Rust computed it: the figure and, beside it, the two sides drawn to scale so the gap can be seen. */
export function BalanceCard({ money: m }: { money: FinanceView }) {
  const fin = m.finances;
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
  const gap = b === null ? 0 : Math.abs(b);
  return (
    <Card className="grid gap-6 min-[1000px]:grid-cols-[minmax(0,5fr)_minmax(0,7fr)] min-[1000px]:items-center">
      <div className="flex min-w-0 flex-col gap-4">
        <div className="flex flex-wrap items-center gap-3">
          <h2 className="text-heading font-bold">{f.balance.title}</h2>
          <Tag tone={tag.tone} icon={tag.icon}>
            {f.balance[state]}
          </Tag>
        </div>
        <div>
          <div className="tabular text-title font-normal tracking-tight min-[1000px]:text-hero">{b === null ? "—" : `${b < 0 ? "−" : ""}${peso(gap)}`}</div>
          <p className="mt-1 max-w-[42ch] text-ui text-ink-2">{text}</p>
        </div>
        {fin.expenses_basis !== "unknown" && <p className="text-small text-ink-3">{fin.expenses_basis === "list" ? f.balance.fromList : f.balance.fromEstimate}</p>}
      </div>
      {b !== null ? (
        <Inset className="space-y-5 !p-5">
          <ScaleRow label={f.balance.income} amount={fin.income_annual_mxn} top={top} tone="green" gap={state === "deficit" ? { amount: gap, tone: "amber", text: `${f.balance.shortBy(peso(gap))} ${f.balance.shortByNote}` } : undefined} />
          <ScaleRow label={f.balance.expenses} amount={out} top={top} tone="ink" gap={state === "surplus" ? { amount: gap, tone: "green", text: `${f.balance.leftOver(peso(gap))} ${f.balance.leftOverNote}` } : undefined} />
        </Inset>
      ) : (
        <Inset className="flex items-center gap-4 !p-5">
          <Tile icon="info" tone="neutral" />
          <p className="text-ui text-ink-2">{f.balance.waiting}</p>
        </Inset>
      )}
    </Card>
  );
}
