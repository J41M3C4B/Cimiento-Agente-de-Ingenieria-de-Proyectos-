import { useQuery, useQueryClient } from "@tanstack/react-query";
import { useEffect, useState } from "react";
import { Icon } from "../../components/icons";
import { Block, Button, Facts, IconTile, RowActions, StatStrip, Tabs, TextButton, Widget } from "../../components/ui";
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

  const initial = (inst?.name?.trim() || "C").charAt(0).toUpperCase();

  return (
    <div className="px-6 py-6">
      <header className="flex flex-wrap items-start justify-between gap-x-6 gap-y-4 pb-6">
        <div className="flex min-w-0 flex-1 items-start gap-4">
          <span aria-hidden="true" className="flex h-14 w-14 shrink-0 items-center justify-center rounded-2xl bg-stone-900 font-display text-[26px] font-medium text-white shadow-card">
            {initial}
          </span>
          <div className="min-w-0 flex-1">
            <p className="text-[12px] font-medium uppercase tracking-[0.12em] text-stone-500">{t.title}</p>
            <h1 className={`mt-0.5 break-words text-[34px] font-medium leading-[1.1] ${inst?.name ? "text-stone-900" : "text-stone-500"}`}>{inst?.name || t.banner.namePlaceholder}</h1>
            <div className="mt-3 max-w-2xl">
              <h2 className="text-[12px] font-medium text-stone-500">{t.about.title}</h2>
              <textarea
                aria-label={t.about.label}
                rows={1}
                value={about}
                onChange={(e) => setAbout(e.target.value)}
                onBlur={saveAbout}
                readOnly={!view}
                placeholder={view ? t.about.placeholder : t.about.needsName}
                className="mt-0.5 block w-full resize-none bg-transparent text-[14px] leading-relaxed text-stone-700 [field-sizing:content] placeholder:text-stone-500 focus-visible:outline-none"
              />
            </div>
          </div>
        </div>
        <div className="flex flex-col items-end gap-3">
          {import.meta.env.DEV && (
            <p className="flex items-center gap-3 text-[12px] text-stone-500">
              {t.banner.loadExample}
              <button type="button" onClick={() => loadExample("asilo")} disabled={busy} className="font-medium text-stone-800 hover:underline disabled:opacity-50">
                {t.devAsilo}
              </button>
              <button type="button" onClick={() => loadExample("casa-hogar")} disabled={busy} className="font-medium text-stone-800 hover:underline disabled:opacity-50">
                {t.devCasaHogar}
              </button>
            </p>
          )}
          {view && (
            <div className="flex gap-2">
              {view.is_draft && (
                <Button variant="primary" onClick={confirm} disabled={busy}>
                  <Icon name="check" size={15} strokeWidth={2.6} />
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
      </header>

      <div className="space-y-5">
        {profile.isSuccess && !view && (
          <section className="flex flex-wrap items-center justify-between gap-5 rounded-2xl bg-white px-6 py-8 shadow-card">
            <div className="max-w-xl">
              <h2 className="text-[18px] font-semibold">{t.onboarding.title}</h2>
              <p className="mt-1 text-stone-700">{t.onboarding.text}</p>
            </div>
            <Button variant="primary" size="lg" onClick={() => open({ kind: "institution" })}>
              <Icon name="plus" size={17} strokeWidth={2.4} />
              {t.onboarding.action}
            </Button>
          </section>
        )}

        {view && (
          <>
            <StatStrip
              items={[
                { icon: "heart", tone: "violet", label: t.kpi.people, value: money(totals.population), sub: view.input.capacity_total ? t.kpi.peopleOf(money(view.input.capacity_total)) : undefined },
                { icon: "briefcase", tone: "teal", label: t.kpi.staff, value: money(totals.staff_paid + totals.staff_volunteer), sub: t.kpi.staffPaid(totals.staff_paid) },
                { icon: "banknote", tone: "amber", label: t.kpi.payroll, value: peso(totals.payroll_monthly_mxn), sub: t.kpi.perYear(peso(totals.payroll_annual_mxn)) },
                { icon: "wallet", tone: "green", label: t.kpi.fees, value: peso(totals.fees_monthly_mxn), sub: t.kpi.payers(totals.fee_payers) },
              ]}
            />

            {headsUp.length > 0 && (
              <ul className="space-y-2">
                {headsUp.map((i, n) => (
                  <li key={n} className="flex items-center gap-2.5 rounded-xl bg-amber-50 px-3.5 py-2.5 text-amber-800">
                    <Icon name="warn" size={16} />
                    {es.issues[i.code]}
                  </li>
                ))}
              </ul>
            )}

            <Tabs
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

            <div role="tabpanel" id="panel-general" aria-labelledby="tab-general" hidden={tab !== "general"} className="grid items-start gap-5 xl:grid-cols-[minmax(0,1fr)_360px]">
              <div className="divide-y divide-stone-200 rounded-2xl bg-white px-7 py-3 shadow-card">
                <Block title={t.cards.institution} action={edition({ kind: "institution" })}>
                  <Facts items={[[t.fields.name, inst?.name], [t.fields.kind, inst ? t.kinds[inst.kind] : null]]} />
                </Block>
                <Block title={t.cards.contact} note={t.privateNote} action={edition({ kind: "contact" })}>
                  <Facts items={[[t.fields.phone, inst?.contact_phone], [t.fields.email, inst?.contact_email]]} />
                </Block>
                <Block title={t.cards.legal} note={t.legalNote} action={edition({ kind: "legal" })}>
                  <Facts items={[[t.fields.rfc, inst?.legal_rfc], [t.fields.legalRep, inst?.legal_rep_name]]} />
                </Block>
                <Block title={t.cards.capacity} action={edition({ kind: "capacity" })}>
                  <Facts
                    items={[
                      [t.fields.capacity, view.input.capacity_total !== null ? `${money(view.input.capacity_total)} personas` : null],
                      [t.fields.annualBudget, view.input.annual_budget_mxn !== null ? peso(view.input.annual_budget_mxn) : null],
                      [t.fields.notes, view.input.notes],
                    ]}
                  />
                </Block>
              </div>

              <aside className="space-y-4">
                <Widget
                  title={t.cards.income}
                  action={
                    <Button size="sm" variant="soft" onClick={() => open({ kind: "income", index: null })}>
                      {t.addSource}
                    </Button>
                  }
                >
                  {income.length === 0 ? (
                    <p className="text-stone-700">{t.incomeEmpty}</p>
                  ) : (
                    <>
                      <p className="mb-2 flex items-baseline gap-2">
                        <span className="text-[24px] font-semibold tracking-tight tabular-nums">{peso(view.totals.income_annual_mxn)}</span>
                        <span className="text-[13px] text-stone-600">{t.incomeTotal}</span>
                      </p>
                      <ul className="divide-y divide-stone-200 border-t border-stone-200">
                        {income.map((it, i) => (
                          <li key={i} className="group flex items-center gap-2 py-2">
                            <span className="min-w-0 flex-1 break-words leading-snug">{it.label}</span>
                            <span className="font-medium tabular-nums">{it.annual_amount_mxn !== null ? peso(it.annual_amount_mxn) : "—"}</span>
                            <RowActions onEdit={() => open({ kind: "income", index: i })} onRemove={() => removeItem("income", i)} busy={busy} />
                          </li>
                        ))}
                      </ul>
                    </>
                  )}
                </Widget>

                {todo.length > 0 && (
                  <Widget title={t.todo.title}>
                    <ul className="-my-1 divide-y divide-stone-200">
                      {todo.map((x) => (
                        <li key={x.text}>
                          <button type="button" onClick={x.go} className="group flex w-full items-center gap-3 py-2.5 text-left">
                            <span aria-hidden="true" className="h-2 w-2 shrink-0 rounded-full bg-amber-300" />
                            <span className="min-w-0 flex-1 text-[13.5px]">{x.text}</span>
                            <Icon name="next" size={15} className="text-stone-500 transition-colors group-hover:text-blue-800" />
                          </button>
                        </li>
                      ))}
                    </ul>
                  </Widget>
                )}
              </aside>
            </div>

            <div role="tabpanel" id="panel-staff" aria-labelledby="tab-staff" hidden={tab !== "staff"} className="rounded-2xl bg-white p-6 shadow-card">
              <RosterTab entity="staff" onProfile={onRosterProfile} />
            </div>
            <div role="tabpanel" id="panel-population" aria-labelledby="tab-population" hidden={tab !== "population"} className="rounded-2xl bg-white p-6 shadow-card">
              <RosterTab entity="beneficiary" onProfile={onRosterProfile} />
            </div>
            <div role="tabpanel" id="panel-facilities" aria-labelledby="tab-facilities" hidden={tab !== "facilities"} className="rounded-2xl bg-white p-6 shadow-card">
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
      </div>

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
          <IconTile icon={toast.tone === "ok" ? "check" : "alert"} tone="neutral" small />
          {toast.text}
        </div>
      )}
    </div>
  );
}
