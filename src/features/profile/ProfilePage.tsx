import { useQuery, useQueryClient } from "@tanstack/react-query";
import { useEffect, useState } from "react";
import { Icon } from "../../components/icons";
import { Alert, Button, Card, Dock, Eyebrow, FactRow, Facts, Inset, Metric, TabPanel, Tag, TextButton, Toast } from "../../components/ui";
import { QuarantineDialog } from "../../components/QuarantineDialog";
import { es } from "../../i18n/es-MX";
import { devLoadFixture, profileConfirm, profileGet, profileSave, rosterOverview, toAppError } from "../../lib/tauri";
import type { Decision, ProfileInput, ProfileIssue, ProfileTotals, ProfileView, QuarantineReport } from "../../lib/types";
import { FacilitiesTab } from "./FacilitiesTab";
import { BalanceCard, ExpensesCard, IncomeCard } from "./FinanceCards";
import { ProfileEdit } from "./ProfileEdit";
import type { Edit } from "./ProfileEdit";
import { fromView, toInput } from "./profileForm";
import { RosterTab } from "./RosterTab";
import { hrOverview } from "../hr/api";
import { HR_KEY, StaffTab } from "../hr/StaffTab";
import { useSession } from "../access/session";

const t = es.profile;
const money = (n: number) => n.toLocaleString("es-MX");
const peso = (n: number) => `$${money(n)}`;
type Tab = "general" | "staff" | "population" | "facilities";

const ZERO: ProfileTotals = {
  population: 0, staff_paid: 0, staff_volunteer: 0, income_annual_mxn: 0,
  payroll_monthly_mxn: 0, payroll_annual_mxn: 0, payroll_benefits_annual_mxn: 0, payroll_cost_annual_mxn: 0, benefits_assumed: 0,
  staff_support_annual_mxn: 0, external_staff_annual_mxn: 0,
  fee_payers: 0, fees_monthly_mxn: 0, fees_annual_mxn: 0,
};

/**
 * Mi institución (docs/13 §10). One header tray in two halves: who the institution is (name, mission, state) and
 * its four figures; below, the detail by cut-out tab. What is on the screen is what is saved: «Editar» opens a
 * window with just those fields, so there is no half-edited page.
 */
export function ProfilePage() {
  const qc = useQueryClient();
  // the example data replaces records: only the administrator, in development (ADR-028)
  const access = useSession();
  const profile = useQuery({ queryKey: ["profile"], queryFn: profileGet });
  // the staff lives in its own module (ADR-027); the people served, in the roster (ADR-020)
  const staffModule = useQuery({ queryKey: HR_KEY, queryFn: hrOverview });
  const peopleRoster = useQuery({ queryKey: ["roster", "beneficiary"], queryFn: () => rosterOverview("beneficiary") });

  const [tab, setTab] = useState<Tab>("general");
  const [edit, setEdit] = useState<Edit | null>(null);
  const [busy, setBusy] = useState(false);
  const [toast, setToast] = useState<{ tone: "ok" | "error"; text: string } | null>(null);
  const [issues, setIssues] = useState<ProfileIssue[]>([]);
  const [quarantine, setQuarantine] = useState<{ input: ProfileInput; report: QuarantineReport; onSaved?: () => void } | null>(null);

  const view = profile.data ?? null;
  const inst = view?.input.institution;

  // the notice goes away by itself
  useEffect(() => {
    if (!toast) return;
    const id = window.setTimeout(() => setToast(null), 3500);
    return () => window.clearTimeout(id);
  }, [toast]);

  /** Saves the whole profile with a change. Returns whether it was saved; a window closes only then. */
  async function commit(input: ProfileInput, decision?: Decision, onSaved?: () => void): Promise<boolean> {
    setBusy(true);
    setIssues([]);
    try {
      const out = await profileSave(input, decision);
      if (out.status === "saved") {
        setQuarantine(null);
        qc.setQueryData(["profile"], out.profile);
        setToast({ tone: "ok", text: es.common.saved });
        onSaved?.();
        return true;
      }
      if (out.status === "invalid") setIssues(out.issues);
      else setQuarantine({ input, report: out.report, onSaved });
      return false;
    } catch (e) {
      setToast({ tone: "error", text: toAppError(e).message });
      return false;
    } finally {
      setBusy(false);
    }
  }

  async function confirm() {
    setBusy(true);
    try {
      qc.setQueryData(["profile"], await profileConfirm());
      setToast({ tone: "ok", text: t.confirmed });
    } catch (e) {
      setToast({ tone: "error", text: toAppError(e).message });
    } finally {
      setBusy(false);
    }
  }

  async function loadExample(name: "asilo" | "casa-hogar") {
    setBusy(true);
    try {
      qc.setQueryData(["profile"], await devLoadFixture(name));
      await qc.invalidateQueries({ queryKey: ["roster"] });
      await qc.invalidateQueries({ queryKey: HR_KEY });
    } catch (e) {
      setToast({ tone: "error", text: toAppError(e).message });
    } finally {
      setBusy(false);
    }
  }

  function removeItem(kind: "income" | "expenses" | "facilities", index: number) {
    if (!view) return;
    const f = fromView(view);
    f[kind].splice(index, 1);
    void commit(toInput(f));
  }

  const open = (e: Edit) => {
    setIssues([]);
    setEdit(e);
  };

  const notify = (text: string) => setToast({ tone: "ok", text });
  const onRosterProfile = (p: ProfileView) => qc.setQueryData(["profile"], p);

  const totals = staffModule.data?.totals ?? view?.totals ?? ZERO;
  const staffCount = staffModule.data?.people.filter((p) => p.status !== "left").length ?? 0;
  const peopleCount = peopleRoster.data?.entries.length ?? 0;
  const facilities = view?.input.facilities ?? [];
  const headsUp = (view?.issues ?? []).filter((i) => !i.blocking);

  // what is still missing, each one leading to where it is filled in
  const todo: { text: string; go: () => void }[] = [];
  if (view) {
    if (!inst?.contact_phone && !inst?.contact_email) todo.push({ text: t.todo.contact, go: () => open({ kind: "contact" }) });
    if (staffModule.isSuccess && staffCount === 0) todo.push({ text: t.todo.staff, go: () => setTab("staff") });
    if (peopleRoster.isSuccess && peopleCount === 0) todo.push({ text: t.todo.population, go: () => setTab("population") });
    if (facilities.length === 0) todo.push({ text: t.todo.facilities, go: () => setTab("facilities") });
  }

  const edition = (e: Edit) => <TextButton onClick={() => open(e)}>{t.edit}</TextButton>;

  const [showTodo, setShowTodo] = useState(false);
  const capacity = view?.input.capacity_total ?? 0;

  return (
    <div className="flex flex-col gap-4">
      <Card className="grid gap-6 min-[1000px]:grid-cols-[minmax(0,5fr)_minmax(0,7fr)]">
        <div className="flex min-w-0 flex-col gap-4">
          <Eyebrow>{t.title}</Eyebrow>
          <div className="my-auto flex flex-col gap-4 py-2">
            <h1 className={`break-words text-hero font-bold tracking-tight ${inst?.name ? "text-ink" : "text-ink-3"}`}>{inst?.name || t.banner.namePlaceholder}</h1>
            <p className={`max-w-[44ch] text-body ${inst?.mission ? "text-ink-2" : "text-ink-3"}`}>{inst?.mission || (view ? t.about.placeholder : t.about.needsName)}</p>
          </div>
          {view && (
            <div className="flex flex-wrap items-center gap-3">
              <Tag tone={view.is_draft ? "amber" : "green"} icon={view.is_draft ? "warn" : "check"}>
                {view.is_draft ? t.status.draft : t.status.confirmed}
              </Tag>
              {view.is_draft && (
                <Button size="sm" variant="primary" onClick={confirm} disabled={busy}>
                  <Icon name="check" size={16} strokeWidth={2.4} />
                  {t.confirm}
                </Button>
              )}
              <Button size="sm" variant="secondary" onClick={() => open({ kind: "institution" })}>
                <Icon name="pencil" size={16} />
                {t.edit}
              </Button>
              {todo.length > 0 && (
                <TextButton aria-expanded={showTodo} onClick={() => setShowTodo((v) => !v)} className="inline-flex items-center gap-1">
                  {t.todo.summary(todo.length)}
                  <Icon name="down" size={14} className={`transition-transform ${showTodo ? "rotate-180" : ""}`} />
                </TextButton>
              )}
            </div>
          )}
          {showTodo && todo.length > 0 && (
            <ul className="flex flex-wrap gap-2">
              {todo.map((x) => (
                <li key={x.text}>
                  <Button size="sm" variant="soft" onClick={x.go}>
                    {x.text}
                    <Icon name="next" size={14} />
                  </Button>
                </li>
              ))}
            </ul>
          )}
        </div>

        {view ? (
          <div className="grid min-w-0 grid-cols-2 gap-3 max-[520px]:grid-cols-1">
            <Metric icon="heart" tone="violet" label={t.kpi.people} value={money(totals.population)} sub={capacity ? t.kpi.peopleOf(money(capacity)) : undefined} fill={capacity ? (totals.population / capacity) * 100 : undefined} />
            <Metric icon="briefcase" tone="teal" label={t.kpi.staff} value={money(staffModule.isSuccess ? staffCount : totals.staff_paid + totals.staff_volunteer)} sub={t.kpi.staffPaid(totals.staff_paid)} />
            <Metric icon="banknote" tone="amber" label={t.finance.payroll.label} value={peso(totals.payroll_cost_annual_mxn)} sub={t.finance.payroll.sub(peso(totals.payroll_annual_mxn), peso(totals.payroll_benefits_annual_mxn))} note={totals.benefits_assumed > 0 ? t.finance.payroll.assumed(totals.benefits_assumed) : undefined} />
            <Metric icon="wallet" tone="green" label={t.kpi.fees} value={peso(totals.fees_monthly_mxn)} sub={t.kpi.payers(totals.fee_payers)} />
          </div>
        ) : (
          profile.isSuccess && (
            <Inset className="flex flex-col justify-center gap-4 !p-6">
              <div>
                <h2 className="text-heading font-bold">{t.onboarding.title}</h2>
                <p className="mt-1 text-ui text-ink-2">{t.onboarding.text}</p>
              </div>
              <div>
                <Button variant="primary" onClick={() => open({ kind: "institution" })}>
                  <Icon name="plus" size={16} strokeWidth={2.4} />
                  {t.onboarding.action}
                </Button>
              </div>
            </Inset>
          )
        )}

        {headsUp.length > 0 && (
          <div className="space-y-2 min-[1000px]:col-span-2">
            {headsUp.map((i, n) => (
              <Alert key={n} tone="warn">
                {es.issues[i.code]}
              </Alert>
            ))}
          </div>
        )}
      </Card>

      {view && (
        <Dock
          label={t.sections.basics}
          value={tab}
          onChange={setTab}
          items={[
            { id: "general", label: t.tabs.general },
            { id: "staff", label: t.tabs.staff, count: staffCount },
            { id: "population", label: t.tabs.population, count: peopleCount },
            { id: "facilities", label: t.tabs.facilities, count: facilities.length },
          ]}
        >
          <TabPanel id="general" active={tab === "general"}>
            <div className="grid items-start gap-4 min-[1280px]:grid-cols-[minmax(0,2fr)_minmax(0,1fr)]">
              <Card className="dock-attach">
                <FactRow title={t.cards.institution} action={edition({ kind: "institution" })}>
                  <Facts columns={2} items={[[t.fields.name, inst?.name], [t.fields.kind, inst ? t.kinds[inst.kind] : null]]} />
                </FactRow>
                <FactRow title={t.cards.contact} note={t.privateNote} action={edition({ kind: "contact" })}>
                  <Facts columns={2} items={[[t.fields.phone, inst?.contact_phone], [t.fields.email, inst?.contact_email]]} />
                </FactRow>
                <FactRow title={t.cards.legal} note={t.legalNote} action={edition({ kind: "legal" })}>
                  <Facts columns={2} items={[[t.fields.rfc, inst?.legal_rfc], [t.fields.legalRep, inst?.legal_rep_name]]} />
                </FactRow>
                <FactRow title={t.cards.capacity} action={edition({ kind: "capacity" })}>
                  <Facts
                    columns={2}
                    items={[
                      [t.fields.capacity, view.input.capacity_total !== null ? `${money(view.input.capacity_total)} personas` : null],
                      [t.fields.annualBudget, view.input.annual_budget_mxn !== null ? peso(view.input.annual_budget_mxn) : null],
                      [t.fields.notes, view.input.notes],
                    ]}
                  />
                </FactRow>
              </Card>

              <BalanceCard view={view} />
            </div>
            <div className="mt-4 grid items-start gap-4 min-[1000px]:grid-cols-2">
              <IncomeCard view={view} busy={busy} onAdd={() => open({ kind: "income", index: null })} onEdit={(index) => open({ kind: "income", index })} onRemove={(index) => removeItem("income", index)} />
              <ExpensesCard
                view={view}
                busy={busy}
                onAdd={() => open({ kind: "expense", index: null })}
                onEdit={(index) => open({ kind: "expense", index })}
                onRemove={(index) => removeItem("expenses", index)}
                onEditEstimate={() => open({ kind: "capacity" })}
              />
            </div>
          </TabPanel>
          <TabPanel id="staff" active={tab === "staff"}>
            <Card className="dock-attach">
              <StaffTab onProfile={onRosterProfile} onNotice={notify} />
            </Card>
          </TabPanel>
          <TabPanel id="population" active={tab === "population"}>
            <Card className="dock-attach">
              <RosterTab entity="beneficiary" onProfile={onRosterProfile} onNotice={notify} />
            </Card>
          </TabPanel>
          <TabPanel id="facilities" active={tab === "facilities"}>
            <Card className="dock-attach">
              <FacilitiesTab
                facilities={facilities}
                busy={busy}
                onAdd={() => open({ kind: "facility", index: null })}
                onEdit={(i) => open({ kind: "facility", index: i })}
                onRemove={(i) => removeItem("facilities", i)}
              />
            </Card>
          </TabPanel>
        </Dock>
      )}

      {import.meta.env.DEV && access?.can("settings") && (
        <p className="flex flex-wrap items-center justify-center gap-3 text-small text-ink-3">
          {t.banner.loadExample}
          <Button size="sm" variant="plain" onClick={() => loadExample("asilo")} disabled={busy}>
            {t.devAsilo}
          </Button>
          <Button size="sm" variant="plain" onClick={() => loadExample("casa-hogar")} disabled={busy}>
            {t.devCasaHogar}
          </Button>
        </p>
      )}

      {edit && <ProfileEdit edit={edit} view={view} issues={issues} busy={busy} onCommit={(input, onSaved) => void commit(input, undefined, onSaved)} onClose={() => setEdit(null)} />}

      {quarantine && (
        <QuarantineDialog
          report={quarantine.report}
          busy={busy}
          onRedact={() => void commit(quarantine.input, "redact", quarantine.onSaved)}
          onNotPersonal={() => void commit(quarantine.input, "not_personal", quarantine.onSaved)}
          onCancel={() => setQuarantine(null)}
        />
      )}

      {toast && <Toast tone={toast.tone}>{toast.text}</Toast>}
    </div>
  );
}
