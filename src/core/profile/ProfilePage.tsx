import { useQuery, useQueryClient } from "@tanstack/react-query";
import { useEffect, useState } from "react";
import { Icon } from "../../components/icons";
import { Alert, Button, Card, Eyebrow, FactRow, Facts, Inset, Metric, Tag, TextButton, Tile, Toast } from "../../components/ui";
import type { Page } from "../../components/Shell";
import { MODULE_META } from "../../components/modules";
import type { ModuleId } from "../../components/modules";
import { QuarantineDialog } from "../../components/QuarantineDialog";
import { es } from "../../i18n/es-MX";
import { devLoadFixture, profileConfirm, profileGet, profileSave, toAppError } from "../../lib/tauri";
import type { Decision, ProfileInput, ProfileIssue, ProfileTotals, QuarantineReport } from "../../lib/types";
import { CapacityCard } from "./CapacityCard";
import { ProfileEdit } from "./ProfileEdit";
import type { Edit } from "./ProfileEdit";
import { toInput } from "./profileForm";
import { FINANCE_KEY, financeGet } from "../../modules/finance/api";
import { careOverview } from "../../modules/care/api";
import { facilitiesOverview } from "../../modules/facilities/api";
import { FACILITIES_KEY } from "../../modules/facilities/FacilitiesTab";
import { CARE_KEY } from "../../modules/care/CareTab";
import { hrOverview } from "../../modules/hr/api";
import { HR_KEY } from "../../modules/hr/StaffTab";
import { useSession } from "../access/session";

const t = es.profile;
const count = (n: number) => n.toLocaleString("es-MX");
const peso = (n: number) => `$${count(n)}`;

const ZERO: ProfileTotals = {
  population: 0, staff_paid: 0, staff_volunteer: 0,
  payroll_monthly_mxn: 0, payroll_annual_mxn: 0, payroll_benefits_annual_mxn: 0, payroll_cost_annual_mxn: 0, benefits_assumed: 0,
  staff_support_annual_mxn: 0, external_staff_annual_mxn: 0,
  fee_payers: 0, fees_monthly_mxn: 0, fees_annual_mxn: 0,
};

/**
 * Mi institución (docs/13 §10), the core of the app (ADR-032). One header tray in two halves: who the institution is
 * (name, mission, state) and its four figures; below, its data and a line for each module, which opens it. What is
 * on the screen is what is saved: «Editar» opens a window with just those fields, so there is no half-edited page.
 */
export function ProfilePage({ onGo }: { onGo: (page: Page) => void }) {
  const qc = useQueryClient();
  // the example data replaces records: only the administrator, in development (ADR-028)
  const access = useSession();
  const profile = useQuery({ queryKey: ["profile"], queryFn: profileGet });
  // what each module has, for its line (ADR-032)
  const finance = useQuery({ queryKey: FINANCE_KEY, queryFn: financeGet });
  // the staff and the people served live in their own modules (ADR-027, ADR-029)
  const staffModule = useQuery({ queryKey: HR_KEY, queryFn: hrOverview });
  const peopleModule = useQuery({ queryKey: CARE_KEY, queryFn: careOverview });
  // and the facilities in theirs (ADR-030)
  const facilitiesModule = useQuery({ queryKey: FACILITIES_KEY, queryFn: facilitiesOverview });

  const [edit, setEdit] = useState<Edit | null>(null);
  const [busy, setBusy] = useState(false);
  const [toast, setToast] = useState<{ tone: "ok" | "error"; text: string } | null>(null);
  const [issues, setIssues] = useState<ProfileIssue[]>([]);
  const [quarantine, setQuarantine] = useState<{ input: ProfileInput; report: QuarantineReport; onSaved?: () => void } | null>(null);

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

  const open = (e: Edit) => {
    setIssues([]);
    setEdit(e);
  };


  const totals = staffModule.data?.totals ?? view?.totals ?? ZERO;
  const staffCount = staffModule.data?.people.filter((p) => p.status !== "left").length ?? 0;
  const peopleCount = peopleModule.data?.board.indicators.served ?? 0;
  const spacesCount = facilitiesModule.data?.indicators.spaces ?? 0;
  const headsUp = (view?.issues ?? []).filter((i) => !i.blocking);

  // what is still missing, each one leading to where it is filled in
  const todo: { text: string; go: () => void }[] = [];
  if (view) {
    if (!inst?.contact_phone && !inst?.contact_email) todo.push({ text: t.todo.contact, go: () => open({ kind: "contact" }) });
    if (staffModule.isSuccess && staffCount === 0) todo.push({ text: t.todo.staff, go: () => onGo("staff") });
    if (peopleModule.isSuccess && peopleCount === 0) todo.push({ text: t.todo.population, go: () => onGo("people") });
    if (facilitiesModule.isSuccess && spacesCount === 0) todo.push({ text: t.todo.facilities, go: () => onGo("facilities") });
  }

  const edition = (e: Edit) => <TextButton onClick={() => open(e)}>{t.edit}</TextButton>;

  // a line for each module: what it has, and the way into it
  const balance = money?.finances.balance_annual_mxn ?? null;
  const modules: { page: Exclude<ModuleId, "projects">; title: string; figure: string; label: string }[] = [
    { page: "staff", title: es.nav.staff, figure: count(staffCount), label: t.modules.unit.staff(staffCount) },
    { page: "people", title: es.nav.people, figure: count(peopleCount), label: t.modules.unit.people(peopleCount) },
    { page: "facilities", title: es.nav.facilities, figure: count(spacesCount), label: t.modules.unit.facilities(spacesCount) },
    {
      page: "finance",
      title: es.nav.finance,
      figure: balance === null ? "—" : `${balance < 0 ? "−" : ""}${peso(Math.abs(balance))}`,
      label: balance === null ? t.modules.financeUnknown : t.modules.unit.finance,
    },
  ];

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
        <>
          <div className="grid items-start gap-4 min-[1280px]:grid-cols-[minmax(0,2fr)_minmax(0,1fr)]">
            <Card>
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
            </Card>

            <CapacityCard view={view} onEdit={() => open({ kind: "capacity" })} />
          </div>
          <Card className="flex flex-col gap-4">
            <div>
              <h2 className="text-heading font-bold">{t.modules.title}</h2>
              <p className="mt-1 text-small text-ink-3">{t.modules.help}</p>
            </div>
            <ul className="grid gap-3 sm:grid-cols-2 min-[1200px]:grid-cols-4">
              {modules.map((m) => (
                <li key={m.page}>
                  <button type="button" onClick={() => onGo(m.page)} className={`module-tile tone-${MODULE_META[m.page].tone}`}>
                    <span className="flex w-full items-center gap-3">
                      <Tile icon={MODULE_META[m.page].icon} tone={MODULE_META[m.page].tone} />
                      <b className="text-ui font-bold">{m.title}</b>
                    </span>
                    <span className="block">
                      <span className="tabular block text-title font-normal leading-none tracking-tight">{m.figure}</span>
                      <span className="mt-1.5 block text-small text-ink-2">{m.label}</span>
                    </span>
                    <span className="module-tile-go">
                      {t.modules.open}
                      <Icon name="next" size={14} strokeWidth={2.6} />
                    </span>
                  </button>
                </li>
              ))}
            </ul>
          </Card>
        </>
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
          issues={issues}
          busy={busy}
          onCommit={(values, onSaved) => void commit(toInput(values), undefined, onSaved)}
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

      {toast && <Toast tone={toast.tone}>{toast.text}</Toast>}
    </div>
  );
}
