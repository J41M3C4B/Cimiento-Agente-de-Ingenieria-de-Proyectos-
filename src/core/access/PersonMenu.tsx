import { useState } from "react";
import { Icon } from "../../components/icons";
import { Alert, Avatar, Button, Inset, Modal, Tag, TextInput, Toast } from "../../components/ui";
import { es } from "../../i18n/es-MX";
import { toAppError } from "../../lib/tauri";
import { accessChangePassword } from "./api";
import { AVATAR_TONES, useAvatarTone } from "./avatarColor";
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

const MENU_ITEM = "flex h-ctl w-full items-center gap-3 rounded-field px-3 text-left text-ui font-bold transition-colors hover:bg-inset";

/**
 * Who is inside, at the right of the top bar: their avatar, and in the menu their name and role, what they can do about
 * their own account (change the password) and locking or closing the session (ADR-028).
 */
export function PersonMenu({ access }: { access: SessionApi }) {
  const [open, setOpen] = useState(false);
  const [changing, setChanging] = useState(false);
  const [notice, setNotice] = useState(false);
  const { display_name: name, username, role } = access.session;
  const [tone, setTone] = useAvatarTone(username);

  return (
    <div className="relative">
      <button
        type="button"
        aria-haspopup="menu"
        aria-expanded={open}
        aria-label={`${t.menu.account}: ${name}`}
        onClick={() => setOpen((v) => !v)}
        className="topbar-person"
      >
        <Avatar name={name} tone={tone} />
      </button>
      {open && (
        <>
          <div className="fixed inset-0 z-40" onClick={() => setOpen(false)} aria-hidden="true" />
          <div role="menu" className="absolute right-0 top-14 z-50 w-72 rounded-inset bg-card p-2 shadow-float">
            <Inset className="flex items-center gap-3 !p-3">
              <Avatar name={name} tone={tone} />
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
            <div role="group" aria-label={t.menu.color} className="px-3 pb-3 pt-1">
              <span className="mb-2 block text-small font-bold text-ink-3">{t.menu.color}</span>
              <div className="flex flex-wrap gap-2">
                {AVATAR_TONES.map((option) => (
                  <button
                    key={option}
                    type="button"
                    aria-pressed={option === tone}
                    aria-label={t.menu.colors[option] ?? option}
                    title={t.menu.colors[option] ?? option}
                    onClick={() => setTone(option)}
                    className={`avatar-swatch tone-${option}`}
                  >
                    {option === tone && <Icon name="check" size={14} strokeWidth={3} />}
                  </button>
                ))}
              </div>
            </div>
            <button
              type="button"
              role="menuitem"
              onClick={() => {
                setOpen(false);
                setChanging(true);
              }}
              className={MENU_ITEM}
            >
              <Icon name="pencil" size={16} className="text-ink-2" />
              {t.menu.changePassword}
            </button>
            <button
              type="button"
              role="menuitem"
              onClick={() => {
                setOpen(false);
                access.lock();
              }}
              className={MENU_ITEM}
            >
              <Icon name="lock" size={16} className="text-ink-2" />
              {t.menu.lock}
            </button>
            <button
              type="button"
              role="menuitem"
              onClick={() => {
                setOpen(false);
                access.logout();
              }}
              className={`${MENU_ITEM} text-red-ink`}
            >
              <Icon name="x" size={16} />
              {t.menu.logout}
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
