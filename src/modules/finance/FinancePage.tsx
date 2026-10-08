import { useQuery, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import { Alert } from "../../components/ui";
import { ModulePage, useNotice } from "../../components/ModulePage";
import { QuarantineDialog } from "../../components/QuarantineDialog";
import { es } from "../../i18n/es-MX";
import { profileGet, toAppError } from "../../lib/tauri";
import type { Decision, FinanceInput, ProfileIssue, ProfileTotals, QuarantineReport } from "../../lib/types";
import { FINANCE_KEY, financeGet, financeSave } from "./api";
import { FinanceEdit, type MoneyEdit } from "./FinanceEdit";
import { BalanceCard, ExpensesCard, IncomeCard } from "./FinanceCards";
import { toFinance } from "./financeForm";

const ZERO: ProfileTotals = {
  population: 0, staff_paid: 0, staff_volunteer: 0,
  payroll_monthly_mxn: 0, payroll_annual_mxn: 0, payroll_benefits_annual_mxn: 0, payroll_cost_annual_mxn: 0, benefits_assumed: 0,
  staff_support_annual_mxn: 0, external_staff_annual_mxn: 0,
  fee_payers: 0, fees_monthly_mxn: 0, fees_annual_mxn: 0,
};

/**
 * Finanzas (ADR-026, ADR-032): the balance, the income by kind and the expenses. Every figure comes from Rust; the
 * payroll and the stay fees come from the staff and the people served, already added up.
 */
export function FinancePage() {
  const qc = useQueryClient();
  const finance = useQuery({ queryKey: FINANCE_KEY, queryFn: financeGet });
  // the payroll the expenses show comes with the totals of the profile
  const profile = useQuery({ queryKey: ["profile"], queryFn: profileGet });
  const [edit, setEdit] = useState<MoneyEdit | null>(null);
  const [busy, setBusy] = useState(false);
  const [issues, setIssues] = useState<ProfileIssue[]>([]);
  const [quarantine, setQuarantine] = useState<{ input: FinanceInput; report: QuarantineReport; onSaved?: () => void } | null>(null);
  const [notice, notify] = useNotice();

  const money = finance.data ?? null;
  const totals = profile.data?.totals ?? ZERO;
  const headsUp = (money?.issues ?? []).filter((i) => !i.blocking);

  /** Saves the whole money with a change. Returns whether it was saved; a window closes only then. */
  async function commit(input: FinanceInput, decision?: Decision, onSaved?: () => void): Promise<boolean> {
    setBusy(true);
    setIssues([]);
    try {
      const out = await financeSave(input, decision);
      if (out.status === "saved") {
        setQuarantine(null);
        qc.setQueryData(FINANCE_KEY, out.finance);
        notify(es.common.saved);
        onSaved?.();
        return true;
      }
      if (out.status === "invalid") setIssues(out.issues);
      else setQuarantine({ input, report: out.report, onSaved });
      return false;
    } catch (e) {
      notify(toAppError(e).message, "error");
      return false;
    } finally {
      setBusy(false);
    }
  }

  function removeItem(kind: "income" | "expenses", index: number) {
    if (!money) return;
    const m: FinanceInput = { ...money.input, income: [...money.input.income], expenses: [...money.input.expenses] };
    m[kind].splice(index, 1);
    void commit(m);
  }

  const open = (e: MoneyEdit) => {
    setIssues([]);
    setEdit(e);
  };

  return (
    <ModulePage title={es.modules.finance.title} intro={es.modules.finance.intro} notice={notice}>
      {headsUp.map((i, n) => (
        <Alert key={n} tone="warn">
          {es.issues[i.code]}
        </Alert>
      ))}
      {money && (
        <>
          <BalanceCard money={money} />
          <div className="grid items-start gap-4 min-[1000px]:grid-cols-2">
            <IncomeCard money={money} busy={busy} onAdd={() => open({ kind: "income", index: null })} onEdit={(index) => open({ kind: "income", index })} onRemove={(index) => removeItem("income", index)} />
            <ExpensesCard
              money={money}
              totals={totals}
              busy={busy}
              onAdd={() => open({ kind: "expense", index: null })}
              onEdit={(index) => open({ kind: "expense", index })}
              onRemove={(index) => removeItem("expenses", index)}
              onEditEstimate={() => open({ kind: "estimate" })}
            />
          </div>
        </>
      )}

      {edit && (
        <FinanceEdit
          edit={edit}
          money={money?.input ?? null}
          issues={issues}
          busy={busy}
          onCommit={(values, onSaved) => void commit(toFinance(values), undefined, onSaved)}
          onClose={() => setEdit(null)}
        />
      )}
      {quarantine && (
        <QuarantineDialog
          report={quarantine.report}
          busy={busy}
          onRedact={() => void commit(quarantine.input, "redact", quarantine.onSaved)}
          onNotPersonal={() => void commit(quarantine.input, "not_personal", quarantine.onSaved)}
          onCancel={() => setQuarantine(null)}
        />
      )}
    </ModulePage>
  );
}
