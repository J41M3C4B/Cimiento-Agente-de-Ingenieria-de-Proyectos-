import { useQuery, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import { Alert, Avatar, Button, Card, Dock, Inset, Modal, PageHeader, Select, StatusDot, TabPanel, Tag, TextInput, Toast } from "../../components/ui";
import { es } from "../../i18n/es-MX";
import { toAppError } from "../../lib/tauri";
import { adminAudit, adminOverview, adminRecoveryCodeNew, adminRequestResolve, adminUserCreate, adminUserResetPassword, adminUserUpdate } from "./api";
import type { AdminOverview, PersonAccess, UserRow } from "./types";

const t = es.admin;
const KEY = ["admin"] as const;
type Tab = "people" | "requests" | "audit" | "recovery";
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
          <Inset className="space-y-1">
            <b className="block font-bold">
              {p.role}: {es.access.roles.manager}
            </b>
            <span className="block text-small text-ink-2">{p.roleNote}</span>
          </Inset>
        </>
      )}
      <TextInput label={p.temporary} hint={p.temporaryHint} autoFocus={resetting} value={password} onChange={(e) => setPassword(e.target.value)} />
      {error && <Alert tone="error">{error}</Alert>}
    </Modal>
  );
}

function PeopleTab({ data, onData }: { data: AdminOverview; onData: (o: AdminOverview) => void }) {
  const p = t.people;
  const [dialog, setDialog] = useState<{ person: PersonAccess | null; user: UserRow | null } | null>(null);
  const [error, setError] = useState<string | null>(null);
  const userOf = (id: string | null) => data.users.find((u) => u.id === id) ?? null;
  const outside = data.users.filter((u) => !data.people.some((x) => x.user_id === u.id));

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
    <li key={key} className="flex flex-wrap items-center gap-3 py-3">
      <Avatar size="sm" name={name} />
      <div className="min-w-[200px] flex-1">
        <b className="block font-bold">{name}</b>
        <span className="block text-small text-ink-3">{[u ? `@${u.username} · ${es.access.roles[u.role]}` : null, sub].filter(Boolean).join(" · ")}</span>
      </div>
      {accountState(u)}
      <div className="flex flex-wrap gap-2">
        {!u && person && person.status !== "left" && (
          <Button size="sm" variant="primary" onClick={() => setDialog({ person, user: null })}>
            {p.give}
          </Button>
        )}
        {u && u.role !== "admin" && (
          <>
            <Button size="sm" onClick={() => setDialog({ person: null, user: u })}>
              {p.reset}
            </Button>
            <Button size="sm" variant="plain" onClick={() => toggle(u)}>
              {u.active ? p.disable : p.enable}
            </Button>
          </>
        )}
      </div>
    </li>
  );

  return (
    <Card className="space-y-4">
      <div className="flex flex-wrap items-center justify-between gap-3">
        <p className="max-w-[70ch] text-ui text-ink-2">{p.help}</p>
        <Button size="sm" variant="secondary" onClick={() => setDialog({ person: null, user: null })}>
          {p.giveOutside}
        </Button>
      </div>
      {error && <Alert tone="error">{error}</Alert>}
      <ul className="divide-y divide-line">
        {outside.map((u) => row(u.id, u.display_name, u.role === "admin" ? null : p.outside, u, null))}
        {data.people.map((x) => row(x.person_id, x.full_name, x.status === "left" ? p.left : null, userOf(x.user_id), x))}
      </ul>
      {dialog && <AccountDialog person={dialog.person} user={dialog.user} onDone={onData} onClose={() => setDialog(null)} />}
    </Card>
  );
}

function RequestsTab({ data, onData }: { data: AdminOverview; onData: (o: AdminOverview) => void }) {
  const r = t.requests;
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  async function resolve(id: string, approve: boolean) {
    setBusy(true);
    setError(null);
    try {
      onData(await adminRequestResolve(id, approve));
    } catch (e) {
      setError(toAppError(e).message);
    } finally {
      setBusy(false);
    }
  }
  return (
    <Card className="space-y-4">
      <p className="text-ui text-ink-2">{r.help}</p>
      {error && <Alert tone="error">{error}</Alert>}
      {data.pending.length === 0 ? (
        <Inset className="text-ui text-ink-3">{r.empty}</Inset>
      ) : (
        <ul className="divide-y divide-line">
          {data.pending.map((x) => (
            <li key={x.id} className="flex flex-wrap items-center gap-3 py-3">
              <div className="min-w-[220px] flex-1">
                <b className="block font-bold">{x.target_label}</b>
                <span className="block text-small text-ink-3">
                  {r.kinds[x.kind]} · {r.asked(x.requested_by_name, when(x.requested_at))}
                </span>
              </div>
              <Button size="sm" disabled={busy} onClick={() => resolve(x.id, false)}>
                {r.reject}
              </Button>
              <Button size="sm" variant="danger" disabled={busy} onClick={() => resolve(x.id, true)}>
                {r.approve}
              </Button>
            </li>
          ))}
        </ul>
      )}
      {data.resolved.length > 0 && (
        <div className="space-y-2">
          <h3 className="field-label">{r.history}</h3>
          <ul className="divide-y divide-line">
            {data.resolved.map((x) => (
              <li key={x.id} className="flex flex-wrap items-center gap-3 py-2 text-ui">
                <span className="min-w-[200px] flex-1">
                  {x.target_label} <span className="text-small text-ink-3">· {r.kinds[x.kind]}</span>
                </span>
                <Tag tone={x.status === "approved" ? "red" : "green"}>{x.status === "approved" ? r.approved : r.rejected}</Tag>
                <span className="text-small text-ink-3">{x.resolved_at ? when(x.resolved_at) : ""}</span>
              </li>
            ))}
          </ul>
        </div>
      )}
    </Card>
  );
}

function AuditTab() {
  const a = t.audit;
  const [filter, setFilter] = useState("");
  const log = useQuery({ queryKey: ["admin-audit", filter], queryFn: () => adminAudit(filter || null) });
  return (
    <Card className="space-y-4">
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
                  <td className="tabular whitespace-nowrap">{when(x.at)}</td>
                  <td>{x.actor ?? a.system}</td>
                  <td>{a.events[x.event] ?? x.event}</td>
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
    <Card className="space-y-4">
      <p className="text-ui text-ink-2">{t.recovery.help}</p>
      {code && (
        <Inset className="space-y-2 text-center">
          <span className="tabular select-all text-title font-bold tracking-tight">{code}</span>
          <p className="text-small text-ink-2">{es.access.recovery.help}</p>
        </Inset>
      )}
      {error && <Alert tone="error">{error}</Alert>}
      <Button
        variant="secondary"
        onClick={async () => {
          try {
            setCode(await adminRecoveryCodeNew());
          } catch (e) {
            setError(toAppError(e).message);
          }
        }}
      >
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
