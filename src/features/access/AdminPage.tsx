import { useQuery, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import { Icon } from "../../components/icons";
import type { IconName } from "../../components/icons";
import { Alert, Avatar, Button, Card, Dock, Inset, Modal, PageHeader, Select, StatusDot, TabPanel, Tag, Tile, TextInput, Toast } from "../../components/ui";
import type { Tone } from "../../components/ui";
import { es } from "../../i18n/es-MX";
import { toAppError } from "../../lib/tauri";
import { adminAudit, adminOverview, adminRecoveryCodeNew, adminRequestResolve, adminUserCreate, adminUserResetPassword, adminUserUpdate } from "./api";
import { RecoveryCodeBox } from "./RecoveryCodeBox";
import type { AdminOverview, PersonAccess, RequestRow, UserRow } from "./types";

const t = es.admin;
const KEY = ["admin"] as const;
type Tab = "people" | "requests" | "audit" | "recovery";
const day = (iso: string) => new Date(iso).toLocaleDateString("es-MX", { dateStyle: "medium" });
const hour = (iso: string) => new Date(iso).toLocaleTimeString("es-MX", { timeStyle: "short" });
const when = (iso: string) => new Date(iso).toLocaleString("es-MX", { dateStyle: "medium", timeStyle: "short" });

/** A window to give access (to a person of the staff or to someone outside it) or to set a new temporary password. */
function AccountDialog({
  person, user, onDone, onClose,
}: {
  person: PersonAccess | null;
  user: UserRow | null;
  onDone: (o: AdminOverview) => void;
  onClose: () => void;
}) {
  const p = t.people;
  const [name, setName] = useState("");
  const [username, setUsername] = useState(person?.suggested_username ?? "");
  const [password, setPassword] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const resetting = user !== null;

  async function save() {
    setBusy(true);
    setError(null);
    try {
      onDone(
        resetting
          ? await adminUserResetPassword(user.id, password)
          : await adminUserCreate({ personId: person?.person_id ?? null, displayName: person ? null : name, username, role: "manager", temporaryPassword: password }),
      );
      onClose();
    } catch (e) {
      setError(toAppError(e).message);
    } finally {
      setBusy(false);
    }
  }

  const title = resetting ? p.resetTitle(user.display_name) : person ? p.giveTitle(person.full_name) : p.giveOutside;
  return (
    <Modal
      title={title}
      onClose={onClose}
      footer={
        <>
          <Button onClick={onClose}>{es.common.cancel}</Button>
          <Button variant="primary" disabled={busy || password.length < 8 || (!resetting && !username) || (!resetting && !person && !name)} onClick={save}>
            {resetting ? p.resetSave : p.create}
          </Button>
        </>
      }
    >
      {!resetting && !person && <TextInput label={p.name} autoFocus value={name} onChange={(e) => setName(e.target.value)} />}
      {!resetting && (
        <>
          <TextInput label={es.access.username} autoFocus={!!person} value={username} onChange={(e) => setUsername(e.target.value.toLowerCase())} />
          <Inset className="flex items-start gap-3">
            <Tile icon="shield" tone="sky" small />
            <div className="min-w-0 space-y-1">
              <b className="block font-bold">
                {p.role}: {es.access.roles.manager}
              </b>
              <span className="block text-small text-ink-2">{p.roleNote}</span>
            </div>
          </Inset>
        </>
      )}
      <TextInput label={p.temporary} hint={p.temporaryHint} autoFocus={resetting} value={password} onChange={(e) => setPassword(e.target.value)} />
      {error && <Alert tone="error">{error}</Alert>}
    </Modal>
  );
}

/** A small count with its words, over an inset. */
function Count({ value, label, tone }: { value: number; label: string; tone: Tone }) {
  return (
    <Inset className="flex items-center gap-3 !px-4 !py-3">
      <span className={`tag tone-${tone} !h-ctl-sm !min-w-ctl-sm justify-center !text-heading !font-extrabold`}>{value}</span>
      <span className="text-small font-semibold text-ink-2">{label}</span>
    </Inset>
  );
}

function PeopleTab({ data, onData }: { data: AdminOverview; onData: (o: AdminOverview) => void }) {
  const p = t.people;
  const [dialog, setDialog] = useState<{ person: PersonAccess | null; user: UserRow | null } | null>(null);
  const [error, setError] = useState<string | null>(null);
  const userOf = (id: string | null) => data.users.find((u) => u.id === id) ?? null;
  const withAccount = [
    ...data.users.filter((u) => !data.people.some((x) => x.user_id === u.id)).map((u) => ({ key: u.id, name: u.display_name, sub: u.role === "admin" ? null : p.outside, user: u, person: null as PersonAccess | null })),
    ...data.people.filter((x) => x.user_id).map((x) => ({ key: x.person_id, name: x.full_name, sub: x.status === "left" ? p.left : null, user: userOf(x.user_id), person: x as PersonAccess | null })),
  ];
  const without = data.people.filter((x) => !x.user_id);
  const active = data.users.filter((u) => u.active).length;
  const mustChange = data.users.filter((u) => u.active && u.must_change_password).length;
  const missing = without.filter((x) => x.status !== "left").length;

  async function toggle(u: UserRow) {
    setError(null);
    try {
      onData(await adminUserUpdate(u.id, u.role, !u.active));
    } catch (e) {
      setError(toAppError(e).message);
    }
  }

  const accountState = (u: UserRow | null) =>
    !u ? (
      <StatusDot tone="neutral">{p.noAccount}</StatusDot>
    ) : !u.active ? (
      <StatusDot tone="red">{p.inactive}</StatusDot>
    ) : u.waiting ? (
      <StatusDot tone="amber">{p.waiting}</StatusDot>
    ) : u.must_change_password ? (
      <StatusDot tone="amber">{p.mustChange}</StatusDot>
    ) : (
      <StatusDot tone="green">{u.last_login_at ? p.lastLogin(when(u.last_login_at)) : p.never}</StatusDot>
    );

  const row = (key: string, name: string, sub: string | null, u: UserRow | null, person: PersonAccess | null) => (
    <li key={key} className="flex flex-wrap items-center gap-x-4 gap-y-3 py-3">
      <Avatar name={name} />
      <div className="min-w-[200px] flex-1">
        <b className="block font-bold">{name}</b>
        <span className="mt-0.5 flex flex-wrap items-center gap-x-2 gap-y-1 text-small text-ink-3">
          {u && (
            <>
              @{u.username}
              <Tag tone={u.role === "admin" ? "violet" : "sky"} variant="soft">
                {es.access.roles[u.role]}
              </Tag>
            </>
          )}
          {sub}
        </span>
      </div>
      <span className="text-small font-semibold">{accountState(u)}</span>
      <div className="flex flex-wrap items-center gap-2">
        {!u && person && person.status !== "left" && (
          <Button size="sm" variant="primary" onClick={() => setDialog({ person, user: null })}>
            {p.give}
          </Button>
        )}
        {u && u.role !== "admin" && (
          <>
            <Button size="sm" variant="secondary" onClick={() => setDialog({ person: null, user: u })}>
              {p.reset}
            </Button>
            <Button size="sm" variant="plain" className={u.active ? "!text-red-ink" : ""} onClick={() => toggle(u)}>
              {u.active ? p.disable : p.enable}
            </Button>
          </>
        )}
      </div>
    </li>
  );

  return (
    <Card className="dock-attach space-y-5">
      <div className="flex flex-wrap items-center justify-between gap-3">
        <p className="max-w-[70ch] text-ui text-ink-2">{p.help}</p>
        <Button size="sm" variant="secondary" onClick={() => setDialog({ person: null, user: null })}>
          <Icon name="plus" size={16} strokeWidth={2.4} />
          {p.giveOutside}
        </Button>
      </div>
      <div className="grid grid-cols-1 gap-3 sm:grid-cols-3">
        <Count value={active} label={p.summaryActive} tone="green" />
        <Count value={mustChange} label={p.summaryPending} tone="amber" />
        <Count value={missing} label={p.summaryWithout} tone="sky" />
      </div>
      {error && <Alert tone="error">{error}</Alert>}
      <ul className="divide-y divide-line">{withAccount.map((x) => row(x.key, x.name, x.sub, x.user, x.person))}</ul>
      {without.length > 0 && (
        <div className="space-y-1">
          <h3 className="eyebrow">{p.noAccount}</h3>
          <ul className="divide-y divide-line">{without.map((x) => row(x.person_id, x.full_name, x.status === "left" ? p.left : null, null, x))}</ul>
        </div>
      )}
      {dialog && <AccountDialog person={dialog.person} user={dialog.user} onDone={onData} onClose={() => setDialog(null)} />}
    </Card>
  );
}

const KIND_ICON: Record<string, IconName> = { document: "file", hr_person: "user", beneficiary: "heart", project: "folder", roster_field: "sliders", hr_field: "sliders" };

function RequestsTab({ data, onData }: { data: AdminOverview; onData: (o: AdminOverview) => void }) {
  const r = t.requests;
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [confirm, setConfirm] = useState<RequestRow | null>(null);
  async function resolve(id: string, approve: boolean) {
    setBusy(true);
    setError(null);
    try {
      onData(await adminRequestResolve(id, approve));
      setConfirm(null);
    } catch (e) {
      setError(toAppError(e).message);
    } finally {
      setBusy(false);
    }
  }
  return (
    <Card className="dock-attach space-y-5">
      <p className="text-ui text-ink-2">{r.help}</p>
      {error && <Alert tone="error">{error}</Alert>}
      {data.pending.length === 0 ? (
        <Inset className="flex flex-col items-center gap-3 !py-10 text-center">
          <Tile icon="check" tone="green" />
          <div>
            <b className="block font-bold">{r.empty}</b>
            <p className="mx-auto mt-1 max-w-sm text-small text-ink-3">{r.emptyNote}</p>
          </div>
        </Inset>
      ) : (
        <ul className="space-y-3">
          {data.pending.map((x) => (
            <li key={x.id}>
              <Inset className="flex flex-wrap items-center gap-x-4 gap-y-3">
                <Tile icon={KIND_ICON[x.kind] ?? "file"} tone="amber" />
                <div className="min-w-[220px] flex-1">
                  <b className="block break-words font-bold">{x.target_label}</b>
                  <span className="mt-0.5 flex flex-wrap items-center gap-x-2 gap-y-1 text-small text-ink-3">
                    <Tag variant="line">{r.kinds[x.kind]}</Tag>
                    {r.asked(x.requested_by_name, when(x.requested_at))}
                  </span>
                </div>
                <div className="flex flex-wrap items-center gap-2">
                  <Button size="sm" variant="secondary" disabled={busy} onClick={() => resolve(x.id, false)}>
                    {r.reject}
                  </Button>
                  <Button size="sm" variant="destructive" disabled={busy} onClick={() => setConfirm(x)}>
                    {r.approve}
                  </Button>
                </div>
              </Inset>
            </li>
          ))}
        </ul>
      )}
      {data.resolved.length > 0 && (
        <div className="space-y-1">
          <h3 className="eyebrow">{r.history}</h3>
          <ul className="divide-y divide-line">
            {data.resolved.map((x) => (
              <li key={x.id} className="flex flex-wrap items-center gap-x-4 gap-y-1 py-3 text-ui">
                <Tile icon={KIND_ICON[x.kind] ?? "file"} tone="neutral" small />
                <span className="min-w-[200px] flex-1">
                  <b className="font-bold">{x.target_label}</b> <span className="text-small text-ink-3">· {r.kinds[x.kind]}</span>
                </span>
                <Tag tone={x.status === "approved" ? "red" : "green"} variant="soft">
                  {x.status === "approved" ? r.approved : r.rejected}
                </Tag>
                <span className="tabular w-28 text-right text-small text-ink-3">{x.resolved_at ? day(x.resolved_at) : ""}</span>
              </li>
            ))}
          </ul>
        </div>
      )}
      {confirm && (
        <Modal
          title={r.confirmTitle}
          onClose={() => setConfirm(null)}
          footer={
            <>
              <Button onClick={() => setConfirm(null)}>{es.common.cancel}</Button>
              <Button variant="destructive" disabled={busy} onClick={() => resolve(confirm.id, true)}>
                {r.confirmYes}
              </Button>
            </>
          }
        >
          <p>
            <strong>{confirm.target_label}</strong>
          </p>
          <p className="text-ink-2">{r.confirmBody}</p>
        </Modal>
      )}
    </Card>
  );
}

/** The color of a kind of event in the log: the same family always has the same color. */
const EVENT_TONE: [string, Tone][] = [["access.denied", "red"], ["emergency.", "red"], ["auth.login_failed", "amber"], ["auth.locked", "amber"], ["auth.", "sky"], ["user.", "violet"], ["request.", "amber"], ["hr.", "teal"], ["backup.", "green"], ["scanner.", "cyan"]];
const toneOfEvent = (event: string): Tone => EVENT_TONE.find(([p]) => event.startsWith(p))?.[1] ?? "neutral";

function AuditTab() {
  const a = t.audit;
  const [filter, setFilter] = useState("");
  const log = useQuery({ queryKey: ["admin-audit", filter], queryFn: () => adminAudit(filter || null) });
  return (
    <Card className="dock-attach space-y-4">
      <div className="flex flex-wrap items-center justify-between gap-3">
        <p className="text-ui text-ink-2">{a.help}</p>
        <Select label={a.filter} hideLabel value={filter} onChange={(e) => setFilter(e.target.value)} options={[["", a.all], ...Object.entries(a.filters)]} className="w-full sm:w-auto sm:min-w-[220px]" />
      </div>
      {log.isError && <Alert tone="error">{toAppError(log.error).message}</Alert>}
      {log.data && log.data.length === 0 && <Inset className="text-ui text-ink-3">{a.empty}</Inset>}
      {log.data && log.data.length > 0 && (
        <div className="overflow-x-auto">
          <table className="table min-w-[640px]">
            <thead>
              <tr>
                <th>{a.when}</th>
                <th>{a.who}</th>
                <th>{a.what}</th>
              </tr>
            </thead>
            <tbody>
              {log.data.map((x, i) => (
                <tr key={i}>
                  <td className="whitespace-nowrap">
                    <span className="block font-bold">{day(x.at)}</span>
                    <span className="tabular block text-small font-medium text-ink-3">{hour(x.at)}</span>
                  </td>
                  <td>
                    <span className="flex items-center gap-2.5">
                      {x.actor ? <Avatar size="sm" name={x.actor} /> : <Tile icon="logo" tone="neutral" small />}
                      <span className="font-bold">{x.actor ?? a.system}</span>
                    </span>
                  </td>
                  <td>
                    <Tag tone={toneOfEvent(x.event)} variant="soft">
                      {a.events[x.event] ?? x.event}
                    </Tag>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </Card>
  );
}

function RecoveryTab() {
  const [code, setCode] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  return (
    <Card className="dock-attach space-y-5">
      <div className="flex items-start gap-4">
        <Tile icon="shield" tone="sky" />
        <div className="min-w-0 space-y-3">
          <p className="max-w-[70ch] text-ui text-ink-2">{t.recovery.help}</p>
          <Tag tone="amber" variant="soft" icon="warn">
            {t.recovery.renewWarn}
          </Tag>
        </div>
      </div>
      {error && <Alert tone="error">{error}</Alert>}
      {code && (
        <div className="space-y-2">
          <h3 className="eyebrow">{t.recovery.newCode}</h3>
          <RecoveryCodeBox code={code} />
          <p className="text-small text-ink-3">{es.access.recovery.help}</p>
        </div>
      )}
      <Button
        variant={code ? "secondary" : "primary"}
        onClick={async () => {
          try {
            setError(null);
            setCode(await adminRecoveryCodeNew());
          } catch (e) {
            setError(toAppError(e).message);
          }
        }}
      >
        <Icon name="sparkles" size={18} />
        {t.recovery.renew}
      </Button>
    </Card>
  );
}

/** The administration panel (ADR-028): only the administrator reaches it, and Rust refuses it to anyone else. */
export function AdminPage() {
  const qc = useQueryClient();
  const overview = useQuery({ queryKey: KEY, queryFn: adminOverview });
  const [tab, setTab] = useState<Tab>("people");
  const [toast, setToast] = useState<string | null>(null);
  const onData = (o: AdminOverview) => {
    qc.setQueryData(KEY, o);
    // a resolved request changes the lists of the rest of the app
    void qc.invalidateQueries({ predicate: (q) => q.queryKey[0] !== KEY[0] && q.queryKey[0] !== "access-status" });
    setToast(es.common.saved);
    window.setTimeout(() => setToast(null), 3000);
  };
  const data = overview.data;
  return (
    <div className="space-y-6">
      <PageHeader title={t.title} intro={t.intro} />
      {overview.isError && <Alert tone="error">{toAppError(overview.error).message}</Alert>}
      {data && (
        <Dock
          label={t.title}
          value={tab}
          onChange={setTab}
          items={[
            { id: "people", label: t.tabs.people, count: data.users.length },
            { id: "requests", label: t.tabs.requests, count: data.pending.length },
            { id: "audit", label: t.tabs.audit },
            { id: "recovery", label: t.tabs.recovery },
          ]}
        >
          <TabPanel id="people" active={tab === "people"}>
            <PeopleTab data={data} onData={onData} />
          </TabPanel>
          <TabPanel id="requests" active={tab === "requests"}>
            <RequestsTab data={data} onData={onData} />
          </TabPanel>
          <TabPanel id="audit" active={tab === "audit"}>
            {tab === "audit" && <AuditTab />}
          </TabPanel>
          <TabPanel id="recovery" active={tab === "recovery"}>
            <RecoveryTab />
          </TabPanel>
        </Dock>
      )}
      {toast && <Toast>{toast}</Toast>}
    </div>
  );
}
