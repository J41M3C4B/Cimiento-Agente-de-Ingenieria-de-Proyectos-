import { useState } from "react";
import { Icon } from "../../components/icons";
import { Alert, Avatar, Button, Inset, Modal, Tag, TextInput, Toast } from "../../components/ui";
import { es } from "../../i18n/es-MX";
import { toAppError } from "../../lib/tauri";
import { accessChangePassword } from "./api";
import type { SessionApi } from "./session";

const t = es.access;

/** The window to change one's own password: the current one, and the new one twice. */
function ChangePasswordDialog({ onDone, onClose }: { onDone: () => void; onClose: () => void }) {
  const [current, setCurrent] = useState("");
  const [password, setPassword] = useState("");
  const [again, setAgain] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const differ = again !== "" && password !== again;

  async function save() {
    setBusy(true);
    setError(null);
    try {
      await accessChangePassword(current, password);
      onDone();
    } catch (e) {
      setError(toAppError(e).message);
    } finally {
      setBusy(false);
    }
  }

  return (
    <Modal
      title={t.menu.changePassword}
      onClose={onClose}
      footer={
        <>
          <Button onClick={onClose}>{es.common.cancel}</Button>
          <Button variant="primary" disabled={busy || !current || !password || password !== again} onClick={save}>
            {t.change.save}
          </Button>
        </>
      }
    >
      <form
        className="space-y-4"
        onSubmit={(e) => {
          e.preventDefault();
          if (current && password && password === again) void save();
        }}
      >
        <TextInput label={t.menu.currentPassword} type="password" autoComplete="current-password" autoFocus value={current} onChange={(e) => setCurrent(e.target.value)} />
        <TextInput label={t.change.new} hint={t.passwordHint} type="password" autoComplete="new-password" value={password} onChange={(e) => setPassword(e.target.value)} />
        <TextInput label={t.passwordAgain} type="password" autoComplete="new-password" value={again} error={differ ? t.passwordsDiffer : undefined} onChange={(e) => setAgain(e.target.value)} />
        {error && <Alert tone="error">{error}</Alert>}
      </form>
    </Modal>
  );
}

/**
 * Who is inside, at the top right: their name and role, and what they can do about their own account (change the
 * password). Locking and closing the session stay in the rail (ADR-028).
 */
export function PersonMenu({ access }: { access: SessionApi }) {
  const [open, setOpen] = useState(false);
  const [changing, setChanging] = useState(false);
  const [notice, setNotice] = useState(false);
  const { display_name: name, username, role } = access.session;

  return (
    <div className="relative">
      <button
        type="button"
        aria-haspopup="menu"
        aria-expanded={open}
        aria-label={`${t.menu.account}: ${name}`}
        onClick={() => setOpen((v) => !v)}
        className="flex items-center gap-2.5 border-l border-line pl-4 text-ui font-bold"
      >
        <span className="hidden max-w-[220px] truncate md:inline">{name}</span>
        <Avatar name={name} tone="violet" />
        <Icon name="down" size={16} className={`hidden text-ink-3 transition-transform md:block ${open ? "rotate-180" : ""}`} />
      </button>
      {open && (
        <>
          <div className="fixed inset-0 z-40" onClick={() => setOpen(false)} aria-hidden="true" />
          <div role="menu" className="absolute right-0 top-14 z-50 w-72 rounded-inset bg-card p-2 shadow-float">
            <Inset className="flex items-center gap-3 !p-3">
              <Avatar name={name} tone="violet" />
              <div className="min-w-0 flex-1">
                <b className="block truncate font-bold">{name}</b>
                <span className="block truncate text-small text-ink-3">@{username}</span>
              </div>
            </Inset>
            <div className="px-3 py-2">
              <Tag tone={role === "admin" ? "violet" : "sky"} variant="soft">
                {t.roles[role]}
              </Tag>
            </div>
            <button
              type="button"
              role="menuitem"
              onClick={() => {
                setOpen(false);
                setChanging(true);
              }}
              className="flex h-ctl w-full items-center gap-3 rounded-field px-3 text-left text-ui font-bold transition-colors hover:bg-inset"
            >
              <Icon name="lock" size={16} className="text-ink-2" />
              {t.menu.changePassword}
            </button>
          </div>
        </>
      )}
      {changing && (
        <ChangePasswordDialog
          onClose={() => setChanging(false)}
          onDone={() => {
            setChanging(false);
            setNotice(true);
            window.setTimeout(() => setNotice(false), 3500);
          }}
        />
      )}
      {notice && <Toast>{t.menu.changed}</Toast>}
    </div>
  );
}
