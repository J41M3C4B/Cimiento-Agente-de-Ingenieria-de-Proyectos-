import { useState } from "react";
import type { ReactNode } from "react";
import { Icon } from "../../components/icons";
import type { IconName } from "../../components/icons";
import { Alert, Button, Card, Inset, TextButton, TextInput } from "../../components/ui";
import { es } from "../../i18n/es-MX";
import { toAppError } from "../../lib/tauri";
import { accessChangePassword, accessLogin, accessLogout, accessRecover, accessSetupAdmin, accessUnlock } from "./api";
import type { AccessStatus, LoginOutcome, SessionView } from "./types";

const t = es.access;

/** The frame of every entry screen: the logo, a title, a line of help and the form. */
function Frame({ icon, title, help, children, onSubmit }: { icon: IconName; title: string; help: string; children: ReactNode; onSubmit: () => void }) {
  return (
    <div className="flex min-h-screen items-center justify-center bg-canvas p-4 text-body text-ink">
      <Card className="w-full max-w-md">
        <form
          className="space-y-6"
          onSubmit={(e) => {
            e.preventDefault();
            onSubmit();
          }}
        >
          <span className="grid h-ctl w-ctl place-items-center rounded-field bg-ink text-on-ink" title={es.app.name}>
            <Icon name="logo" size={22} />
          </span>
          <div className="space-y-1.5">
            <h1 className="flex items-center gap-2 text-subtitle font-bold leading-tight tracking-tight">
              <Icon name={icon} size={22} className="shrink-0 text-ink-2" />
              {title}
            </h1>
            <p className="text-ui text-ink-2">{help}</p>
          </div>
          {children}
        </form>
      </Card>
    </div>
  );
}

function message(out: LoginOutcome): string | null {
  if (out.status === "wrong") return t.wrong;
  if (out.status === "waiting") return t.waiting(out.wait_secs);
  if (out.status === "disabled") return t.disabled;
  return null;
}

/** Shown once after the first account or a recovery: the code to write down. */
function RecoveryCode({ code, onDone }: { code: string; onDone: () => void }) {
  return (
    <Frame icon="shield" title={t.recovery.title} help={t.recovery.help} onSubmit={onDone}>
      <Inset className="text-center">
        <span className="tabular select-all text-title font-bold tracking-tight">{code}</span>
      </Inset>
      <Button type="submit" variant="primary" className="w-full">
        {t.recovery.saved}
      </Button>
    </Frame>
  );
}

/**
 * Everything that happens before the app opens (ADR-028): making the administrator the first time, entering,
 * the administrator's recovery, changing a temporary password and opening the locked screen.
 */
export function AccessScreen({ status, onSession }: { status: AccessStatus; onSession: (s: SessionView | null) => void }) {
  const [username, setUsername] = useState("");
  const [name, setName] = useState("");
  const [password, setPassword] = useState("");
  const [again, setAgain] = useState("");
  const [pin, setPin] = useState("");
  const [code, setCode] = useState("");
  const [current, setCurrent] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [mode, setMode] = useState<"login" | "forgot" | "recover">("login");
  const [shownCode, setShownCode] = useState<{ code: string; session: SessionView } | null>(null);

  async function run(job: () => Promise<void>) {
    setBusy(true);
    setError(null);
    try {
      await job();
    } catch (e) {
      setError(toAppError(e).message);
    } finally {
      setBusy(false);
    }
  }
  const differ = again !== "" && password !== again;
  const footer = (label: string, disabled: boolean) => (
    <>
      {error && <Alert tone="warn">{error}</Alert>}
      <Button type="submit" variant="primary" className="w-full" disabled={busy || disabled}>
        {label}
      </Button>
    </>
  );

  if (shownCode) return <RecoveryCode code={shownCode.code} onDone={() => onSession(shownCode.session)} />;

  const s = status.session;
  if (status.needs_setup) {
    return (
      <Frame
        icon="shield"
        title={t.setup.title}
        help={t.setup.help}
        onSubmit={() =>
          run(async () => {
            const out = await accessSetupAdmin(name, username, password, status.setup_needs_pin ? pin : null);
            setShownCode({ code: out.recovery_code, session: out.session });
          })
        }
      >
        <TextInput label={t.setup.name} autoFocus value={name} onChange={(e) => setName(e.target.value)} />
        <TextInput label={t.username} autoComplete="username" value={username} onChange={(e) => setUsername(e.target.value)} />
        <TextInput label={t.password} hint={t.passwordHint} type="password" autoComplete="new-password" value={password} onChange={(e) => setPassword(e.target.value)} />
        <TextInput label={t.passwordAgain} type="password" autoComplete="new-password" value={again} error={differ ? t.passwordsDiffer : undefined} onChange={(e) => setAgain(e.target.value)} />
        {status.setup_needs_pin && <TextInput label={t.setup.pin} hint={t.setup.pinHelp} type="password" inputMode="numeric" maxLength={8} value={pin} onChange={(e) => setPin(e.target.value)} />}
        {footer(t.setup.create, !name || !username || !password || password !== again)}
      </Frame>
    );
  }

  if (s?.locked) {
    return (
      <Frame
        icon="lock"
        title={t.locked.title}
        help={t.locked.help(s.display_name)}
        onSubmit={() =>
          run(async () => {
            const out = await accessUnlock(password);
            setPassword("");
            if (out.status === "ok") onSession(out.session);
            else if (out.status === "disabled") onSession(null);
            else setError(message(out));
          })
        }
      >
        <TextInput label={t.password} type="password" autoComplete="current-password" autoFocus value={password} onChange={(e) => setPassword(e.target.value)} />
        {footer(t.enter, !password)}
        <TextButton onClick={() => run(async () => { await accessLogout(); onSession(null); })}>{t.locked.other}</TextButton>
      </Frame>
    );
  }

  if (s?.must_change_password) {
    return (
      <Frame
        icon="shield"
        title={t.change.title}
        help={t.change.help}
        onSubmit={() => run(async () => onSession(await accessChangePassword(current, password)))}
      >
        <TextInput label={t.change.current} type="password" autoComplete="current-password" autoFocus value={current} onChange={(e) => setCurrent(e.target.value)} />
        <TextInput label={t.change.new} hint={t.passwordHint} type="password" autoComplete="new-password" value={password} onChange={(e) => setPassword(e.target.value)} />
        <TextInput label={t.passwordAgain} type="password" autoComplete="new-password" value={again} error={differ ? t.passwordsDiffer : undefined} onChange={(e) => setAgain(e.target.value)} />
        {footer(t.change.save, !current || !password || password !== again)}
      </Frame>
    );
  }

  if (mode === "recover") {
    return (
      <Frame
        icon="shield"
        title={t.recovery.useTitle}
        help={t.forgotHelp}
        onSubmit={() =>
          run(async () => {
            const out = await accessRecover(username, code, password);
            setShownCode({ code: out.recovery_code, session: out.session });
          })
        }
      >
        <TextInput label={t.username} autoComplete="username" autoFocus value={username} onChange={(e) => setUsername(e.target.value)} />
        <TextInput label={t.recovery.code} value={code} onChange={(e) => setCode(e.target.value.toUpperCase())} />
        <TextInput label={t.recovery.newPassword} hint={t.passwordHint} type="password" autoComplete="new-password" value={password} onChange={(e) => setPassword(e.target.value)} />
        <TextInput label={t.passwordAgain} type="password" autoComplete="new-password" value={again} error={differ ? t.passwordsDiffer : undefined} onChange={(e) => setAgain(e.target.value)} />
        {footer(t.recovery.use, !username || !code || !password || password !== again)}
        <TextButton onClick={() => setMode("login")}>{t.recovery.back}</TextButton>
      </Frame>
    );
  }

  return (
    <Frame
      icon="user"
      title={t.login.title}
      help={t.login.help}
      onSubmit={() =>
        run(async () => {
          const out = await accessLogin(username, password);
          setPassword("");
          if (out.status === "ok") onSession(out.session);
          else setError(message(out));
        })
      }
    >
      <TextInput label={t.username} autoComplete="username" autoFocus value={username} onChange={(e) => setUsername(e.target.value)} />
      <TextInput label={t.password} type="password" autoComplete="current-password" value={password} onChange={(e) => setPassword(e.target.value)} />
      {footer(t.enter, !username || !password)}
      {mode === "forgot" ? (
        <Alert tone="info">
          <p>{t.forgotHelp}</p>
          <TextButton onClick={() => setMode("recover")}>{t.recovery.useTitle}</TextButton>
        </Alert>
      ) : (
        <TextButton onClick={() => setMode("forgot")}>{t.login.forgot}</TextButton>
      )}
    </Frame>
  );
}
