import { useQuery, useQueryClient } from "@tanstack/react-query";
import { useEffect, useState } from "react";
import { Icon } from "../../components/icons";
import { Button, FactRow, Facts, Figures, Tile, RowActions, Card, Segmented, Tag, TextButton } from "../../components/ui";
import { QuarantineDialog } from "../../components/QuarantineDialog";
import { es } from "../../i18n/es-MX";
import { devLoadFixture, profileConfirm, profileGet, profileSave, rosterOverview, toAppError } from "../../lib/tauri";
import type { Decision, ProfileInput, ProfileIssue, ProfileTotals, ProfileView, QuarantineReport } from "../../lib/types";
import { FacilitiesTab } from "./FacilitiesTab";
import { ProfileEdit } from "./ProfileEdit";
import type { Edit } from "./ProfileEdit";
import { fromView, toInput } from "./profileForm";
import { RosterTab } from "./RosterTab";

const t = es.profile;
const money = (n: number) => n.toLocaleString("es-MX");
const peso = (n: number) => `$${money(n)}`;
type Tab = "general" | "staff" | "population" | "facilities";

const ZERO: ProfileTotals = {
  population: 0, staff_paid: 0, staff_volunteer: 0, income_annual_mxn: 0,
  payroll_monthly_mxn: 0, payroll_annual_mxn: 0, fee_payers: 0, fees_monthly_mxn: 0, fees_annual_mxn: 0,
};

/**
 * Mi institución. Reading order, top to bottom: who the institution is (name, state, «Sobre nosotros»), what it
 * adds up to (four figures, each in its own color), then the detail by tab. The detail is flat sections separated
 * by a line; only the secondary things on the right (income, what is missing) are cards. What is on the screen is
 * what is saved: «Editar» opens a window with just those fields, so there is no half-edited page.
 */
export function ProfilePage() {
  const qc = useQueryClient();
  const profile = useQuery({ queryKey: ["profile"], queryFn: profileGet });
  const staffRoster = useQuery({ queryKey: ["roster", "staff"], queryFn: () => rosterOverview("staff") });
  const peopleRoster = useQuery({ queryKey: ["roster", "beneficiary"], queryFn: () => rosterOverview("beneficiary") });

  const [tab, setTab] = useState<Tab>("general");
  const [edit, setEdit] = useState<Edit | null>(null);
  const [busy, setBusy] = useState(false);
  const [toast, setToast] = useState<{ tone: "ok" | "error"; text: string } | null>(null);
  const [issues, setIssues] = useState<ProfileIssue[]>([]);
  const [quarantine, setQuarantine] = useState<{ input: ProfileInput; report: QuarantineReport; onSaved?: () => void } | null>(null);

  const view = profile.data ?? null;
  const inst = view?.input.institution;
  const [about, setAbout] = useState("");
  useEffect(() => setAbout(inst?.mission ?? ""), [inst?.mission]);

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
    } catch (e) {
      setToast({ tone: "error", text: toAppError(e).message });
    } finally {
      setBusy(false);
    }
  }

  function saveAbout() {
    if (!view || about.trim() === (inst?.mission ?? "")) return;
    const input = toInput(fromView(view));
    input.institution.mission = about.trim() || null;
    void commit(input);
  }

  function removeItem(kind: "income" | "facilities", index: number) {
    if (!view) return;
    const f = fromView(view);
    f[kind].splice(index, 1);
    void commit(toInput(f));
  }

  const open = (e: Edit) => {
    setIssues([]);
    setEdit(e);
  };

  const onRosterProfile = (p: ProfileView) => qc.setQueryData(["profile"], p);

  const totals = staffRoster.data?.totals ?? view?.totals ?? ZERO;
  const staffCount = staffRoster.data?.entries.length ?? 0;
  const peopleCount = peopleRoster.data?.entries.length ?? 0;
  const facilities = view?.input.facilities ?? [];
  const income = view?.input.income ?? [];
  const incomeTotal = income.reduce((n, it) => n + (it.annual_amount_mxn ?? 0), 0);
  const headsUp = (view?.issues ?? []).filter((i) => !i.blocking);

  // what is still missing, each one leading to where it is filled in
  const todo: { text: string; go: () => void }[] = [];
  if (view) {
    if (!inst?.contact_phone && !inst?.contact_email) todo.push({ text: t.todo.contact, go: () => open({ kind: "contact" }) });
    if (staffRoster.isSuccess && staffCount === 0) todo.push({ text: t.todo.staff, go: () => setTab("staff") });
    if (peopleRoster.isSuccess && peopleCount === 0) todo.push({ text: t.todo.population, go: () => setTab("population") });
    if (facilities.length === 0) todo.push({ text: t.todo.facilities, go: () => setTab("facilities") });
  }

  const edition = (e: Edit) => <TextButton onClick={() => open(e)}>{t.edit}</TextButton>;

  // what the money is made of: each source as a share of the total, drawn as one bar
  const shades = ["bg-stone-900", "bg-stone-600", "bg-stone-400", "bg-stone-300", "bg-stone-200"];
  const [showTodo, setShowTodo] = useState(false);

  return (
    <div className="mx-auto max-w-[1360px] px-6 py-6">
      <Card flush>
        <header className="px-12 pb-6 pt-8">
          <div className="flex flex-wrap items-start justify-between gap-x-8 gap-y-4">
            <div className="min-w-0 flex-1">
              <h1 className={`break-words text-[28px] font-semibold leading-tight tracking-[-0.02em] ${inst?.name ? "text-stone-900" : "text-stone-400"}`}>
                {inst?.name || t.banner.namePlaceholder}
              </h1>
              <textarea
                aria-label={t.about.label}
                rows={1}
                value={about}
                onChange={(e) => setAbout(e.target.value)}
                onBlur={saveAbout}
                readOnly={!view}
                placeholder={view ? t.about.placeholder : t.about.needsName}
                className="-mx-2 mt-1.5 block w-full max-w-3xl resize-none rounded-md bg-transparent px-2 py-1 text-[14px] leading-relaxed text-stone-600 transition-colors [field-sizing:content] placeholder:text-stone-400 hover:bg-stone-50 focus-visible:bg-stone-50 focus-visible:outline-none"
              />
            </div>
            {view && (
              <div className="flex shrink-0 gap-2">
                {view.is_draft && (
                  <Button variant="primary" onClick={confirm} disabled={busy}>
                    <Icon name="check" size={15} strokeWidth={2.4} />
                    {t.confirm}
                  </Button>
                )}
                <Button variant="secondary" onClick={() => open({ kind: "institution" })}>
                  <Icon name="pencil" size={14} />
                  {t.edit}
                </Button>
              </div>
            )}
          </div>

          {view && (
            <div className="mt-3 flex flex-wrap items-center gap-x-6 gap-y-2 text-[13px] text-stone-600">
              <Tag tone={view.is_draft ? "amber" : "green"}>
                <Icon name={view.is_draft ? "warn" : "check"} size={13} strokeWidth={2.6} />
                {view.is_draft ? t.status.draft : t.status.confirmed}
              </Tag>
              {todo.length > 0 && (
                <button type="button" aria-expanded={showTodo} onClick={() => setShowTodo((v) => !v)} className="inline-flex items-center gap-1 font-medium text-stone-900 hover:underline">
                  {t.todo.summary(todo.length)}
                  <Icon name="down" size={14} className={`transition-transform ${showTodo ? "rotate-180" : ""}`} />
                </button>
              )}
            </div>
          )}
          {showTodo && todo.length > 0 && (
            <ul className="mt-3 flex flex-wrap gap-2">
              {todo.map((x) => (
                <li key={x.text}>
                  <button type="button" onClick={x.go} className="inline-flex items-center gap-1.5 rounded-lg bg-stone-100 px-3 py-1.5 text-[13px] text-stone-800 transition-colors hover:bg-stone-200">
                    {x.text}
                    <Icon name="next" size={13} className="text-stone-500" />
                  </button>
                </li>
              ))}
            </ul>
          )}
          {headsUp.length > 0 && (
            <ul className="mt-4 space-y-2">
              {headsUp.map((i, n) => (
                <li key={n} className="flex items-center gap-2.5 rounded-lg bg-amber-50 px-3.5 py-2.5 text-[13px] text-amber-800">
                  <Icon name="warn" size={15} />
                  {es.issues[i.code]}
                </li>
              ))}
            </ul>
          )}
        </header>

        {profile.isSuccess && !view && (
          <section className="mx-12 mb-10 flex flex-wrap items-center justify-between gap-5 rounded-2xl bg-stone-50 px-8 py-8">
            <div className="max-w-xl">
              <h2 className="text-[18px] font-semibold">{t.onboarding.title}</h2>
              <p className="mt-1 text-stone-600">{t.onboarding.text}</p>
            </div>
            <Button variant="primary" onClick={() => open({ kind: "institution" })}>
              <Icon name="plus" size={16} strokeWidth={2.4} />
              {t.onboarding.action}
            </Button>
          </section>
        )}

        {view && (
          <>
            <div className="border-y border-stone-200 px-12 py-5">
              <Figures
                items={[
                  { label: t.kpi.people, value: money(totals.population), sub: view.input.capacity_total ? t.kpi.peopleOf(money(view.input.capacity_total)) : undefined, fill: view.input.capacity_total ? (totals.population / view.input.capacity_total) * 100 : undefined },
                  { label: t.kpi.payroll, value: peso(totals.payroll_monthly_mxn), sub: t.kpi.perYear(peso(totals.payroll_annual_mxn)) },
                  { label: t.kpi.fees, value: peso(totals.fees_monthly_mxn), sub: t.kpi.payers(totals.fee_payers) },
                  { label: t.cards.income, value: peso(incomeTotal), sub: income.length > 0 ? t.incomeSources(income.length) : t.incomeEmpty },
                ]}
              />
            </div>

            <div className="border-b border-stone-200 px-12 py-4">
              <Segmented
                label={t.sections.basics}
                value={tab}
                onChange={setTab}
                items={[
                  { id: "general", label: t.tabs.general },
                  { id: "staff", label: t.tabs.staff, count: staffCount },
                  { id: "population", label: t.tabs.population, count: peopleCount },
                  { id: "facilities", label: t.tabs.facilities, count: facilities.length },
                ]}
              />
            </div>

            <div role="tabpanel" id="panel-general" aria-labelledby="tab-general" hidden={tab !== "general"} className="px-12 py-2">
              <FactRow title={t.cards.institution} action={edition({ kind: "institution" })}>
                <Facts items={[[t.fields.name, inst?.name], [t.fields.kind, inst ? t.kinds[inst.kind] : null]]} />
              </FactRow>
              <FactRow title={t.cards.contact} note={t.privateNote} action={edition({ kind: "contact" })}>
                <Facts items={[[t.fields.phone, inst?.contact_phone], [t.fields.email, inst?.contact_email]]} />
              </FactRow>
              <FactRow title={t.cards.legal} note={t.legalNote} action={edition({ kind: "legal" })}>
                <Facts items={[[t.fields.rfc, inst?.legal_rfc], [t.fields.legalRep, inst?.legal_rep_name]]} />
              </FactRow>
              <FactRow title={t.cards.capacity} action={edition({ kind: "capacity" })}>
                <Facts
                  items={[
                    [t.fields.capacity, view.input.capacity_total !== null ? `${money(view.input.capacity_total)} personas` : null],
                    [t.fields.annualBudget, view.input.annual_budget_mxn !== null ? peso(view.input.annual_budget_mxn) : null],
                    [t.fields.notes, view.input.notes],
                  ]}
                />
              </FactRow>
              <FactRow
                title={t.cards.income}
                action={<TextButton onClick={() => open({ kind: "income", index: null })}>{t.addSource}</TextButton>}
              >
                {income.length === 0 ? (
                  <p className="text-stone-500">{t.incomeEmpty}</p>
                ) : (
                  <div>
                    {incomeTotal > 0 && (
                      <div role="img" aria-label={t.cards.income} className="mb-4 flex h-1.5 gap-0.5 overflow-hidden rounded-full">
                        {income.map((it, i) => (
                          <span key={i} className={`h-full ${shades[i % shades.length]}`} style={{ width: `${((it.annual_amount_mxn ?? 0) / incomeTotal) * 100}%` }} />
                        ))}
                      </div>
                    )}
                    <ul className="divide-y divide-stone-100">
                      {income.map((it, i) => (
                        <li key={i} className="group flex items-center gap-3 py-2">
                          <span aria-hidden="true" className={`h-2 w-2 shrink-0 rounded-full ${shades[i % shades.length]}`} />
                          <span className="min-w-0 flex-1 break-words">{it.label}</span>
                          <span className="tabular-nums text-stone-600">{it.annual_amount_mxn !== null ? peso(it.annual_amount_mxn) : "—"}</span>
                          <RowActions onEdit={() => open({ kind: "income", index: i })} onRemove={() => removeItem("income", i)} busy={busy} />
                        </li>
                      ))}
                    </ul>
                  </div>
                )}
              </FactRow>
            </div>

            <div role="tabpanel" id="panel-staff" aria-labelledby="tab-staff" hidden={tab !== "staff"} className="px-12 py-8">
              <RosterTab entity="staff" onProfile={onRosterProfile} />
            </div>
            <div role="tabpanel" id="panel-population" aria-labelledby="tab-population" hidden={tab !== "population"} className="px-12 py-8">
              <RosterTab entity="beneficiary" onProfile={onRosterProfile} />
            </div>
            <div role="tabpanel" id="panel-facilities" aria-labelledby="tab-facilities" hidden={tab !== "facilities"} className="px-12 py-8">
              <FacilitiesTab
                facilities={facilities}
                busy={busy}
                onAdd={() => open({ kind: "facility", index: null })}
                onEdit={(i) => open({ kind: "facility", index: i })}
                onRemove={(i) => removeItem("facilities", i)}
              />
            </div>
          </>
        )}
      </Card>

      {import.meta.env.DEV && (
        <p className="mt-4 flex items-center justify-center gap-3 text-[12px] text-stone-500">
          {t.banner.loadExample}
          <button type="button" onClick={() => loadExample("asilo")} disabled={busy} className="font-medium text-stone-700 hover:underline disabled:opacity-50">
            {t.devAsilo}
          </button>
          <button type="button" onClick={() => loadExample("casa-hogar")} disabled={busy} className="font-medium text-stone-700 hover:underline disabled:opacity-50">
            {t.devCasaHogar}
          </button>
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

      {toast && (
        <div role="status" className="fixed bottom-6 left-1/2 z-[60] flex -translate-x-1/2 items-center gap-2.5 rounded-xl bg-stone-900 px-4 py-2.5 text-[14px] font-medium text-white shadow-lift">
          <Tile icon={toast.tone === "ok" ? "check" : "alert"} tone="neutral" small />
          {toast.text}
        </div>
      )}
    </div>
  );
}
