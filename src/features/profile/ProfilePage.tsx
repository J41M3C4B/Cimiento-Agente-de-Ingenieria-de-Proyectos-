import { useQuery, useQueryClient } from "@tanstack/react-query";
import { useEffect, useState } from "react";
import { Icon } from "../../components/icons";
import { Alert, Button, Card, Dock, Eyebrow, FactRow, Facts, Inset, Metric, TabPanel, Tag, TextButton, Toast } from "../../components/ui";
import { QuarantineDialog } from "../../components/QuarantineDialog";
import { es } from "../../i18n/es-MX";
import { devLoadFixture, profileConfirm, profileGet, profileSave, toAppError } from "../../lib/tauri";
import type { Decision, FinanceInput, ProfileInput, ProfileIssue, ProfileTotals, ProfileView, QuarantineReport } from "../../lib/types";
import { BalanceCard, ExpensesCard, IncomeCard } from "./FinanceCards";
import { isMoney, ProfileEdit } from "./ProfileEdit";
import type { Edit } from "./ProfileEdit";
import { toFinance, toInput } from "./profileForm";
import { FINANCE_KEY, financeGet, financeSave } from "../finance/api";
import { careOverview } from "../care/api";
import { facilitiesOverview } from "../facilities/api";
import { FACILITIES_KEY, FacilitiesTab } from "../facilities/FacilitiesTab";
import { CARE_KEY, CareTab } from "../care/CareTab";
import { hrOverview } from "../hr/api";
import { HR_KEY, StaffTab } from "../hr/StaffTab";
import { useSession } from "../access/session";

const t = es.profile;
const count = (n: number) => n.toLocaleString("es-MX");
const peso = (n: number) => `$${count(n)}`;
export type ProfileTab = "general" | "staff" | "population" | "facilities";
type Tab = ProfileTab;

const ZERO: ProfileTotals = {
  population: 0, staff_paid: 0, staff_volunteer: 0,
  payroll_monthly_mxn: 0, payroll_annual_mxn: 0, payroll_benefits_annual_mxn: 0, payroll_cost_annual_mxn: 0, benefits_assumed: 0,
  staff_support_annual_mxn: 0, external_staff_annual_mxn: 0,
  fee_payers: 0, fees_monthly_mxn: 0, fees_annual_mxn: 0,
};

/**
 * Mi institución (docs/13 §10). One header tray in two halves: who the institution is (name, mission, state) and
 * its four figures; below, the detail by cut-out tab. What is on the screen is what is saved: «Editar» opens a
 * window with just those fields, so there is no half-edited page.
 */
export function ProfilePage({ initialTab = "general" }: { initialTab?: ProfileTab }) {
  const qc = useQueryClient();
  // the example data replaces records: only the administrator, in development (ADR-028)
  const access = useSession();
  const profile = useQuery({ queryKey: ["profile"], queryFn: profileGet });
  // the money lives in its own module (ADR-032)
  const finance = useQuery({ queryKey: FINANCE_KEY, queryFn: financeGet });
  // the staff and the people served live in their own modules (ADR-027, ADR-029)
  const staffModule = useQuery({ queryKey: HR_KEY, queryFn: hrOverview });
  const peopleModule = useQuery({ queryKey: CARE_KEY, queryFn: careOverview });
  // and the facilities in theirs (ADR-030)
  const facilitiesModule = useQuery({ queryKey: FACILITIES_KEY, queryFn: facilitiesOverview });

  const [tab, setTab] = useState<Tab>(initialTab);
  const [edit, setEdit] = useState<Edit | null>(null);
  const [busy, setBusy] = useState(false);
  const [toast, setToast] = useState<{ tone: "ok" | "error"; text: string } | null>(null);
  const [issues, setIssues] = useState<ProfileIssue[]>([]);
  const [quarantine, setQuarantine] = useState<
    | { target: "profile"; input: ProfileInput; report: QuarantineReport; onSaved?: () => void }
    | { target: "money"; input: FinanceInput; report: QuarantineReport; onSaved?: () => void }
    | null
  >(null);

  const view = profile.data ?? null;
  const money = finance.data ?? null;
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
      else setQuarantine({ target: "profile", input, report: out.report, onSaved });
      return false;
    } catch (e) {
      setToast({ tone: "error", text: toAppError(e).message });
      return false;
    } finally {
      setBusy(false);
    }
  }

  /** Saves the whole money with a change, in its module. Returns whether it was saved. */
  async function commitMoney(input: FinanceInput, decision?: Decision, onSaved?: () => void): Promise<boolean> {
    setBusy(true);
    setIssues([]);
    try {
      const out = await financeSave(input, decision);
      if (out.status === "saved") {
        setQuarantine(null);
        qc.setQueryData(FINANCE_KEY, out.finance);
        setToast({ tone: "ok", text: es.common.saved });
        onSaved?.();
        return true;
      }
      if (out.status === "invalid") setIssues(out.issues);
      else setQuarantine({ target: "money", input, report: out.report, onSaved });
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
      await qc.invalidateQueries({ queryKey: CARE_KEY });
      await qc.invalidateQueries({ queryKey: HR_KEY });
      await qc.invalidateQueries({ queryKey: FACILITIES_KEY });
      await qc.invalidateQueries({ queryKey: FINANCE_KEY });
    } catch (e) {
      setToast({ tone: "error", text: toAppError(e).message });
    } finally {
      setBusy(false);
    }
  }

  function removeItem(kind: "income" | "expenses", index: number) {
    if (!money) return;
    const m: FinanceInput = { ...money.input, income: [...money.input.income], expenses: [...money.input.expenses] };
    m[kind].splice(index, 1);
    void commitMoney(m);
  }

  /** The person decided what to do with what the scanner found: the same save again, with the decision. */
  function resolveQuarantine(decision: Decision) {
    if (!quarantine) return;
    return quarantine.target === "money"
      ? commitMoney(quarantine.input, decision, quarantine.onSaved)
      : commit(quarantine.input, decision, quarantine.onSaved);
  }

  const open = (e: Edit) => {
    setIssues([]);
    setEdit(e);
  };

  const notify = (text: string) => setToast({ tone: "ok", text });
  const onRosterProfile = (p: ProfileView) => {
    qc.setQueryData(["profile"], p);
    // the payroll and the fees of the balance come from the staff and the people served
    void qc.invalidateQueries({ queryKey: FINANCE_KEY });
  };

  const totals = staffModule.data?.totals ?? view?.totals ?? ZERO;
  const staffCount = staffModule.data?.people.filter((p) => p.status !== "left").length ?? 0;
  const peopleCount = peopleModule.data?.board.indicators.served ?? 0;
  const spacesCount = facilitiesModule.data?.indicators.spaces ?? 0;
  const headsUp = [...(view?.issues ?? []), ...(money?.issues ?? [])].filter((i) => !i.blocking);

  // what is still missing, each one leading to where it is filled in
  const todo: { text: string; go: () => void }[] = [];
  if (view) {
    if (!inst?.contact_phone && !inst?.contact_email) todo.push({ text: t.todo.contact, go: () => open({ kind: "contact" }) });
    if (staffModule.isSuccess && staffCount === 0) todo.push({ text: t.todo.staff, go: () => setTab("staff") });
    if (peopleModule.isSuccess && peopleCount === 0) todo.push({ text: t.todo.population, go: () => setTab("population") });
    if (facilitiesModule.isSuccess && spacesCount === 0) todo.push({ text: t.todo.facilities, go: () => setTab("facilities") });
  }

  const edition = (e: Edit) => <TextButton onClick={() => open(e)}>{t.edit}</TextButton>;

  const [showTodo, setShowTodo] = useState(false);
  const capacity = view?.input.capacity_total ?? 0;
  // while nobody is registered in the modules, the quick figures the person gave at the start stand in, and say so
  const servedEstimate = view?.input.served_estimate ?? null;
  const paidEstimate = view?.input.staff_paid_estimate ?? 0;
  const volunteerEstimate = view?.input.staff_volunteer_estimate ?? 0;
  const peopleApprox = servedEstimate !== null && peopleCount === 0 && totals.population === 0;
  const staffApprox = staffCount === 0 && totals.staff_paid + totals.staff_volunteer === 0 && view?.input.staff_paid_estimate != null;

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
            <Metric
              icon="heart"
              tone="violet"
              label={t.kpi.people}
              value={peopleApprox ? `≈ ${count(servedEstimate!)}` : count(totals.population)}
              approx={peopleApprox ? t.kpi.approx : undefined}
              hint={peopleApprox ? t.kpi.approxNote : undefined}
              sub={capacity ? t.kpi.peopleOf(count(capacity)) : undefined}
              fill={capacity ? ((peopleApprox ? servedEstimate! : totals.population) / capacity) * 100 : undefined}
            />
            <Metric
              icon="briefcase"
              tone="teal"
              label={t.kpi.staff}
              value={staffApprox ? `≈ ${count(paidEstimate + volunteerEstimate)}` : count(staffModule.isSuccess ? staffCount : totals.staff_paid + totals.staff_volunteer)}
              approx={staffApprox ? t.kpi.approx : undefined}
              hint={staffApprox ? t.kpi.approxNote : undefined}
              sub={t.kpi.staffPaid(staffApprox ? paidEstimate : totals.staff_paid)}
            />
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
            { id: "facilities", label: t.tabs.facilities, count: spacesCount },
          ]}
        >
          <TabPanel id="general" active={tab === "general"}>
            <div className="grid items-start gap-4 min-[1280px]:grid-cols-[minmax(0,2fr)_minmax(0,1fr)]">
              <Card className="dock-attach">
                <FactRow title={t.cards.institution} action={edition({ kind: "institution" })}>
                  <Facts columns={2} items={[[t.fields.name, inst?.name], [t.fields.kind, inst ? t.kinds[inst.kind] : null]]} />
                </FactRow>
                <FactRow title={t.cards.contact} note={t.privateNote} action={edition({ kind: "contact" })}>
                  <Facts
                    columns={2}
                    items={[
                      [t.fields.phone, inst?.contact_phone],
                      [t.fields.email, inst?.contact_email],
                      [es.institution.state, inst?.state ? es.institution.states[inst.state] : null],
                      [es.institution.municipality, inst?.municipality],
                    ]}
                  />
                </FactRow>
                <FactRow title={t.cards.legal} note={t.legalNote} action={edition({ kind: "legal" })}>
                  <Facts
                    columns={2}
                    items={[
                      [t.fields.rfc, inst?.legal_rfc],
                      [t.fields.legalRep, inst?.legal_rep_name],
                      [es.institution.legalForm, inst?.legal_form ? es.institution.legalForms[inst.legal_form] : null],
                      [es.institution.foundedYear, inst?.founded_year?.toString()],
                      [es.institution.authorizedDonee, inst?.authorized_donee ? es.institution.registry[inst.authorized_donee] : null],
                      [es.institution.cluni, inst?.cluni ? es.institution.registry[inst.cluni] : null],
                    ]}
                  />
                </FactRow>
                <FactRow title={t.cards.capacity} action={edition({ kind: "capacity" })}>
                  <Facts
                    columns={2}
                    items={[
                      [t.fields.capacity, view.input.capacity_total !== null ? `${count(view.input.capacity_total)} personas` : null],
                      [es.institution.servedEstimate, view.input.served_estimate !== null ? `${count(view.input.served_estimate)} ${es.institution.approx}` : null],
                      [es.institution.staffPaidEstimate, view.input.staff_paid_estimate !== null ? `${count(view.input.staff_paid_estimate)} ${es.institution.approx}` : null],
                      [es.institution.staffVolunteerEstimate, view.input.staff_volunteer_estimate !== null ? `${count(view.input.staff_volunteer_estimate)} ${es.institution.approx}` : null],
                      [t.fields.notes, view.input.notes],
                    ]}
                  />
                </FactRow>
              </Card>

              {money && <BalanceCard money={money} />}
            </div>
            <div className="mt-4 grid items-start gap-4 min-[1000px]:grid-cols-2">
              {money && <IncomeCard money={money} busy={busy} onAdd={() => open({ kind: "income", index: null })} onEdit={(index) => open({ kind: "income", index })} onRemove={(index) => removeItem("income", index)} />}
              {money && <ExpensesCard
                money={money}
                totals={totals}
                busy={busy}
                onAdd={() => open({ kind: "expense", index: null })}
                onEdit={(index) => open({ kind: "expense", index })}
                onRemove={(index) => removeItem("expenses", index)}
                onEditEstimate={() => open({ kind: "estimate" })}
              />}
            </div>
          </TabPanel>
          <TabPanel id="staff" active={tab === "staff"}>
            <Card className="dock-attach">
              <StaffTab onProfile={onRosterProfile} onNotice={notify} />
            </Card>
          </TabPanel>
          <TabPanel id="population" active={tab === "population"}>
            <Card className="dock-attach">
              <CareTab onProfile={onRosterProfile} onNotice={notify} />
            </Card>
          </TabPanel>
          <TabPanel id="facilities" active={tab === "facilities"}>
            <Card className="dock-attach">
              <FacilitiesTab onNotice={notify} />
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

      {edit && (
        <ProfileEdit
          edit={edit}
          view={view}
          money={money?.input ?? null}
          issues={issues}
          busy={busy}
          onCommit={(values, onSaved) => void (isMoney(edit) ? commitMoney(toFinance(values), undefined, onSaved) : commit(toInput(values), undefined, onSaved))}
          onClose={() => setEdit(null)}
        />
      )}

      {quarantine && (
        <QuarantineDialog
          report={quarantine.report}
          busy={busy}
          onRedact={() => void resolveQuarantine("redact")}
          onNotPersonal={() => void resolveQuarantine("not_personal")}
          onCancel={() => setQuarantine(null)}
        />
      )}

      {toast && <Toast tone={toast.tone}>{toast.text}</Toast>}
    </div>
  );
}
