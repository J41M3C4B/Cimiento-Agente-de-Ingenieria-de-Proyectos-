import { useQuery, useQueryClient } from "@tanstack/react-query";
import { useEffect, useRef, useState } from "react";
import { Icon } from "../../components/icons";
import { Alert, Button, Card, Eyebrow, FactRow, Facts, IconButton, Inset, ListRow, Status, TextButton, Toast } from "../../components/ui";
import type { Page } from "../../components/Shell";
import { ModuleCard } from "../../components/ModuleCard";
import type { ModuleId } from "../../components/modules";
import { es } from "../../i18n/es-MX";
import { devLoadFixture, profileGet, toAppError } from "../../lib/tauri";
import type { BankAccount, BoardMember, FormValue, InstitutionRecord, ListKind } from "../../lib/types";
import { CapacityCard } from "./CapacityCard";
import { FormWindow } from "./FormWindow";
import { LISTS_KEY, ListWindow, useInstitutionLists } from "./ListWindow";
import type { ListRowRef } from "./ListWindow";
import { RECORDS_KEY, RecordWindow, recordTitle, useRecords } from "./RecordWindow";
import type { RecordRef } from "./RecordWindow";
import { fromOverview, OVERVIEW_KEY, useOverview } from "./gaps";
import type { Where } from "./gaps";
import { FINANCE_KEY } from "../../modules/finance/api";
import { FACILITIES_KEY } from "../../modules/facilities/FacilitiesTab";
import { CARE_KEY } from "../../modules/care/CareTab";
import { HR_KEY } from "../../modules/hr/StaffTab";
import { useSession } from "../access/session";

const t = es.profile;
const count = (n: number) => n.toLocaleString("es-MX");
const peso = (n: number) => `$${count(n)}`;
/** The codes of a field in words, as its form names them («Niñez, Adolescencia»). */
const codesText = (field: string, codes?: string[] | null) =>
  codes?.length ? codes.map((c) => es.forms.fields[field]?.options?.[c] ?? c).join(", ") : null;
/** The label of a field of the catalog. */
const labelOf = (field: string) => es.forms.fields[field]?.label ?? field;

/** The windows of «Mi institución»: each one is a form described in Rust (ADR-033). */
type Edit = { kind: "institution" | "contact" | "legal" | "capacity" };
const FORM_OF: Record<Edit["kind"], string> = {
  institution: "institution.identity",
  contact: "institution.contact",
  legal: "institution.legal",
  capacity: "institution.capacity",
};

/** A date `YYYY-MM-DD` as people write it: «1 de mayo de 2020». */
const dateText = (v: string) => new Date(`${v}T12:00:00`).toLocaleDateString("es-MX", { day: "numeric", month: "long", year: "numeric" });

/** A person of the board in a line: the role, and since when. */
const memberDetail = (m: BoardMember) =>
  [m.role === "other" && m.role_other ? m.role_other : codesText("institution.board.role", [m.role]), m.since && t.lists.since(dateText(m.since))].filter(Boolean).join(" · ");

/** An account in a line: what it is for and whether it goes in the documents. */
const accountDetail = (a: BankAccount) => [codesText("institution.bank.purpose", a.purpose), a.in_documents ? t.lists.inDocuments : t.lists.notInDocuments].filter(Boolean).join(" · ");

/** A document of the record in a line: until when it is valid (or since when it is not) and its file. */
const recordDetail = (r: InstitutionRecord) =>
  [
    r.validity === "expired" && r.valid_until
      ? t.records.expiredOn(dateText(r.valid_until))
      : r.validity === "expiring" && r.days_left !== null
        ? t.records.expiresIn(r.days_left)
        : r.validity === "valid" && r.valid_until
          ? t.records.validUntil(dateText(r.valid_until))
          : null,
    r.file && (r.file.readable ? r.file.name : `${r.file.name}, ${t.records.noText}`),
  ]
    .filter(Boolean)
    .join(" · ");

/** The state of a document, told with its words: what the institution has is also said expiring or expired. */
function RecordState({ r }: { r: InstitutionRecord }) {
  if (r.status === "have" && r.validity === "expired") return <Status kind="error">{t.records.stateExpired}</Status>;
  if (r.status === "have" && r.validity === "expiring") return <Status kind="warn">{t.records.stateExpiring}</Status>;
  const kind = r.status === "have" ? "ok" : r.status === "in_progress" ? "pending" : "idle";
  return <Status kind={kind}>{t.records.status[r.status]}</Status>;
}

/** A datum kept by the catalog, in words: a code by its label, a date as people write it. Empty is `null`. */
function detailText(details: Record<string, FormValue> | null | undefined, field: string): string | null {
  const v = details?.[field];
  if (v === undefined || v === null || v === "" || (Array.isArray(v) && v.length === 0)) return null;
  if (typeof v === "string" && /^\d{4}-\d{2}-\d{2}$/.test(v)) return dateText(v);
  const options = es.forms.fields[field]?.options;
  return typeof v === "string" && options ? (options[v] ?? v) : String(v);
}

/**
 * Mi institución (docs/13 §10), the core of the app (ADR-032). One header tray in two halves: who the institution is
 * (name, mission, state) and its four figures; below, its data and a line for each module, which opens it. What is
 * on the screen is what is saved: «Editar» opens a window with just those fields, so there is no half-edited page.
 */
export function ProfilePage({
  onGo,
  openWindow,
  onWindowOpened,
}: {
  onGo: (page: Page) => void;
  /** a window to open on arrival (the Assistant's «Ir a…») */
  openWindow?: { where: Where; n: number } | null;
  /** the request was used: whoever made it forgets it, so coming back does not open the window again */
  onWindowOpened?: () => void;
}) {
  const qc = useQueryClient();
  // the example data replaces records: only the administrator, in development (ADR-028)
  const access = useSession();
  const profile = useQuery({ queryKey: ["profile"], queryFn: profileGet });
  // what each module has and what is missing, composed in Rust (ADR-033): this page only draws it
  const overview = useOverview();

  const [edit, setEdit] = useState<Edit | null>(null);
  // the board and the accounts of the institution (ADR-033 §3): never to the AI
  const lists = useInstitutionLists();
  const [list, setList] = useState<{ kind: ListKind; row: ListRowRef } | null>(null);
  // the record of the institution (ADR-033 §5): what it must have, with its state
  const records = useRecords();
  const [record, setRecord] = useState<RecordRef | null>(null);
  const recordsPart = useRef<HTMLDivElement>(null);
  const [toRecords, setToRecords] = useState(0);
  const [busy, setBusy] = useState(false);
  const [toast, setToast] = useState<{ tone: "ok" | "error"; text: string } | null>(null);

  const view = profile.data ?? null;
  const o = overview.data ?? null;
  const inst = view?.input.institution;

  // the notice goes away by itself
  useEffect(() => {
    if (!toast) return;
    const id = window.setTimeout(() => setToast(null), 3500);
    return () => window.clearTimeout(id);
  }, [toast]);

  async function loadExample(name: "asilo" | "casa-hogar") {
    setBusy(true);
    try {
      qc.setQueryData(["profile"], await devLoadFixture(name));
      await qc.invalidateQueries({ queryKey: CARE_KEY });
      await qc.invalidateQueries({ queryKey: HR_KEY });
      await qc.invalidateQueries({ queryKey: FACILITIES_KEY });
      await qc.invalidateQueries({ queryKey: FINANCE_KEY });
      await qc.invalidateQueries({ queryKey: OVERVIEW_KEY });
      await qc.invalidateQueries({ queryKey: LISTS_KEY });
      await qc.invalidateQueries({ queryKey: RECORDS_KEY });
    } catch (e) {
      setToast({ tone: "error", text: toAppError(e).message });
    } finally {
      setBusy(false);
    }
  }

  const open = (e: Edit) => setEdit(e);


  const headsUp = (view?.issues ?? []).filter((i) => !i.blocking);
  const d = inst?.details;
  const dt = (field: string) => detailText(d, field);
  // the address in one line, as it goes on an envelope
  const address =
    [
      [dt("institution.street"), dt("institution.ext_number")].filter(Boolean).join(" "),
      dt("institution.int_number") && `${t.addressInterior} ${dt("institution.int_number")}`,
      dt("institution.neighborhood"),
      dt("institution.postal_code") && `${t.addressPostal} ${dt("institution.postal_code")}`,
    ]
      .filter(Boolean)
      .join(", ") || null;
  // a row that shows only when it has something: what applies is decided in Rust, which keeps nothing that does not
  const onlyIf = (row: [string, string | null]): [string, string | null][] => (row[1] ? [row] : []);

  // what is still missing, each one leading to where it is filled in (Rust says what the data still need)
  const { ready: gapsReady, gaps } = fromOverview(overview.data);
  const goTo = (w: Where) =>
    w === "records"
      ? setToRecords((n) => n + 1)
      : w === "board"
        ? setList({ kind: "board_member", row: null })
        : w === "institution" || w === "contact" || w === "legal" || w === "capacity"
          ? open({ kind: w })
          : onGo(w);

  // the Assistant sent the person to the record: it is brought into view once it is drawn. In braces: a WebView2
  // returns a promise from scrollIntoView, and an effect must not return it
  useEffect(() => {
    if (toRecords === 0 || !recordsPart.current) return;
    recordsPart.current.scrollIntoView?.({ block: "start", behavior: "smooth" });
    setToRecords(0);
  }, [toRecords, records.data]);

  // the Assistant sent the person to a window: it opens once per request
  useEffect(() => {
    if (!openWindow) return;
    goTo(openWindow.where);
    onWindowOpened?.();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [openWindow?.n]);

  const edition = (e: Edit) => <TextButton onClick={() => open(e)}>{t.edit}</TextButton>;

  // a tile for each module: what it has, and the way into it. Rust says whether a figure is the quick one of the
  // first start (while the module has no records), and it is marked «≈»
  const people = o?.people ?? { value: 0, approx: false };
  const staff = o?.staff ?? { value: 0, approx: false };
  const spaces = o?.spaces ?? 0;
  const balance = o?.balance_annual_mxn ?? null;
  const shown = (f: { value: number; approx: boolean }) => (f.approx ? `≈ ${count(f.value)}` : count(f.value));
  const modules: { page: Exclude<ModuleId, "projects">; title: string; figure: string; label: string; approx?: boolean }[] = [
    {
      page: "people",
      title: es.nav.people,
      figure: shown(people),
      label: o?.capacity ? t.kpi.peopleOf(count(o.capacity)) : t.modules.unit.people(people.value),
      approx: people.approx,
    },
    { page: "staff", title: es.nav.staff, figure: shown(staff), label: t.modules.unit.staff(staff.value), approx: staff.approx },
    { page: "facilities", title: es.nav.facilities, figure: count(spaces), label: t.modules.unit.facilities(spaces) },
    {
      page: "finance",
      title: es.nav.finance,
      figure: balance === null ? "—" : `${balance < 0 ? "−" : ""}${peso(Math.abs(balance))}`,
      label: balance === null ? t.modules.financeUnknown : t.modules.unit.finance,
    },
  ];

  const [showTodo, setShowTodo] = useState(false);

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
              {gapsReady && (gaps.length === 0 ? <Status kind="ok">{t.completion.done}</Status> : <Status kind="pending">{t.completion.pending(gaps.length)}</Status>)}
              {gaps.length > 0 && (
                <TextButton aria-expanded={showTodo} onClick={() => setShowTodo((v) => !v)} className="inline-flex items-center gap-1">
                  {showTodo ? t.completion.hide : t.completion.show}
                  <Icon name="down" size={14} className={`transition-transform ${showTodo ? "rotate-180" : ""}`} />
                </TextButton>
              )}
              <Button size="sm" variant="secondary" onClick={() => open({ kind: "institution" })}>
                <Icon name="pencil" size={16} />
                {t.edit}
              </Button>
            </div>
          )}
          {showTodo && gaps.length > 0 && (
            <ul className="flex flex-wrap gap-2">
              {gaps.map((x) => (
                <li key={x.code}>
                  <Button size="sm" variant="soft" onClick={() => goTo(x.where)}>
                    {x.text}
                    <Icon name="next" size={14} />
                  </Button>
                </li>
              ))}
            </ul>
          )}
        </div>

        {view ? (
          <ul className="grid min-w-0 grid-cols-2 gap-3 max-[520px]:grid-cols-1">
            {modules.map((m) => (
              <li key={m.page}>
                <ModuleCard module={m.page} title={m.title} figure={m.figure} label={m.label} approx={m.approx ? t.kpi.approx : undefined} note={m.approx ? t.kpi.approxNote : undefined} onOpen={() => onGo(m.page)} />
              </li>
            ))}
          </ul>
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

        {/* the suggestions of the AI still to review, window by window (IA4) */}
        {o && o.suggestions.length > 0 && (
          <div className="space-y-2 min-[1000px]:col-span-2">
            {o.suggestions.map((x) => (
              <Alert key={x.place} tone="info">
                <p>{es.suggestions.pending(x.count, es.assistant.places[`profile.${x.place}`] ?? x.place)}</p>
                <Button size="sm" variant="secondary" className="mt-2" onClick={() => goTo(x.place)}>
                  {es.suggestions.review}
                </Button>
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
                <Facts
                  columns={2}
                  items={[
                    [t.fields.name, inst?.name],
                    [t.fields.attention, codesText("institution.populations", inst?.attention?.populations)],
                    [t.fields.modalities, codesText("institution.modalities", inst?.attention?.modalities)],
                    [labelOf("institution.services"), dt("institution.services")],
                  ]}
                />
              </FactRow>
              <FactRow title={t.cards.contact} note={t.privateNote} action={edition({ kind: "contact" })}>
                <Facts
                  columns={2}
                  items={[
                    [t.fields.phone, inst?.contact_phone],
                    [t.fields.email, inst?.contact_email],
                    [t.fields.address, address],
                    [es.institution.municipality, inst?.municipality],
                    [es.institution.state, inst?.state ? es.institution.states[inst.state] : null],
                  ]}
                />
              </FactRow>
              <FactRow title={t.cards.legal} note={t.legalNote} action={edition({ kind: "legal" })}>
                <Facts
                  columns={2}
                  items={[
                    [labelOf("institution.legal_name"), dt("institution.legal_name")],
                    [t.fields.rfc, inst?.legal_rfc ?? null],
                    [es.institution.legalForm, inst?.legal_form ? es.institution.legalForms[inst.legal_form] : null],
                    [es.institution.foundedYear, inst?.founded_year?.toString() ?? null],
                    ...onlyIf([labelOf("institution.junta_folio"), dt("institution.junta_folio")]),
                    [labelOf("institution.tax_regime"), dt("institution.tax_regime")],
                    [es.institution.authorizedDonee, inst?.authorized_donee ? es.institution.registry[inst.authorized_donee] : null],
                    ...onlyIf([labelOf("institution.donee_category"), dt("institution.donee_category")]),
                    [es.institution.cluni, inst?.cluni ? es.institution.registry[inst.cluni] : null],
                    ...onlyIf([labelOf("institution.cluni_key"), dt("institution.cluni_key")]),
                    [labelOf("institution.legal_rep_name"), inst?.legal_rep_name ?? null],
                    ...onlyIf([labelOf("institution.legal_rep_valid_until"), dt("institution.legal_rep_valid_until")]),
                  ]}
                />
              </FactRow>
              <FactRow
                title={t.cards.board}
                note={t.lists.boardNote}
                action={<TextButton onClick={() => setList({ kind: "board_member", row: null })}>{t.lists.addMember}</TextButton>}
              >
                {lists.data?.board.length ? (
                  <ul className="-mx-2">
                    {lists.data.board.map((m) => (
                      <ListRow
                        key={m.id}
                        title={m.name}
                        detail={memberDetail(m)}
                        state={!m.active && <Status kind="idle">{t.lists.left}</Status>}
                        actions={<IconButton icon="pencil" label={t.edit} variant="plain" size="sm" onClick={() => setList({ kind: "board_member", row: m })} />}
                      />
                    ))}
                  </ul>
                ) : (
                  lists.isSuccess && <p className="text-ui text-ink-3">{t.lists.noBoard}</p>
                )}
              </FactRow>
              <FactRow
                title={t.cards.accounts}
                note={t.lists.accountsNote}
                action={<TextButton onClick={() => setList({ kind: "bank_account", row: null })}>{t.lists.addAccount}</TextButton>}
              >
                {lists.data?.accounts.length ? (
                  <ul className="-mx-2">
                    {lists.data.accounts.map((a) => (
                      <ListRow
                        key={a.id}
                        title={<span className="tabular">{`${a.bank ?? t.lists.clabe} •••• ${a.masked.slice(-4)}`}</span>}
                        detail={accountDetail(a)}
                        state={!a.active && <Status kind="idle">{t.lists.closed}</Status>}
                        actions={<IconButton icon="pencil" label={t.edit} variant="plain" size="sm" onClick={() => setList({ kind: "bank_account", row: a })} />}
                      />
                    ))}
                  </ul>
                ) : (
                  lists.isSuccess && <p className="text-ui text-ink-3">{t.lists.noAccounts}</p>
                )}
              </FactRow>
            </Card>

            <CapacityCard view={view} onEdit={() => open({ kind: "capacity" })} />
          </div>

          <div ref={recordsPart} className="scroll-mt-4">
            <Card>
              <FactRow
                title={t.records.title}
                note={t.records.intro}
                action={<TextButton onClick={() => setRecord({ type: "other", id: null })}>{t.records.addOther}</TextButton>}
              >
                {records.data && (
                  <p className="mb-2 text-ui font-semibold">
                    {[
                      t.records.count(records.data.summary.have, records.data.summary.expected),
                      records.data.summary.in_progress > 0 && t.records.inProgress(records.data.summary.in_progress),
                      records.data.summary.expiring > 0 && t.records.expiring(records.data.summary.expiring),
                      records.data.summary.expired > 0 && t.records.expired(records.data.summary.expired),
                    ]
                      .filter(Boolean)
                      .join(" · ")}
                  </p>
                )}
                {records.isError && <Alert tone="error">{toAppError(records.error).message}</Alert>}
                <ul className="-mx-2">
                  {records.data?.records.map((r) => (
                    <ListRow
                      key={r.id ?? r.type}
                      title={recordTitle(r)}
                      detail={recordDetail(r) || undefined}
                      state={<RecordState r={r} />}
                      actions={<IconButton icon="pencil" label={t.edit} variant="plain" size="sm" onClick={() => setRecord({ type: r.type, id: r.id, custom_name: r.custom_name })} />}
                    />
                  ))}
                </ul>
              </FactRow>
            </Card>
          </div>
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
        <FormWindow
          id={FORM_OF[edit.kind]}
          title={t.modal[edit.kind]}
          onSaved={() => setToast({ tone: "ok", text: es.common.saved })}
          onClose={() => setEdit(null)}
        />
      )}

      {record && <RecordWindow record={record} onDone={(text) => setToast({ tone: "ok", text })} onClose={() => setRecord(null)} />}

      {list && <ListWindow kind={list.kind} row={list.row} onDone={(text) => setToast({ tone: "ok", text })} onClose={() => setList(null)} />}

      {toast && <Toast tone={toast.tone}>{toast.text}</Toast>}
    </div>
  );
}
