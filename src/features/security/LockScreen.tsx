import { useState } from "react";
import { Icon } from "../../components/icons";
import { Alert, Button, Card, TextInput } from "../../components/ui";
import { es } from "../../i18n/es-MX";
import { pinVerify, toAppError } from "../../lib/tauri";

const t = es.lock;

/** Asks for the PIN before anything else is shown. It protects the screen; the data is encrypted anyway. */
export function LockScreen({ onUnlock }: { onUnlock: () => void }) {
  const [pin, setPin] = useState("");
  const [message, setMessage] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  async function enter() {
    setBusy(true);
    try {
      const out = await pinVerify(pin);
      if (out.status === "ok") return onUnlock();
      setMessage(out.status === "locked" ? t.locked(out.wait_secs) : t.wrong);
      setPin("");
    } catch (e) {
      setMessage(toAppError(e).message);
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className="flex min-h-screen items-center justify-center bg-canvas p-4 text-body text-ink">
      <Card className="w-full max-w-md">
        <form
          className="space-y-6"
          onSubmit={(e) => {
            e.preventDefault();
            if (pin) void enter();
          }}
        >
          <div>
            <span className="grid h-ctl w-ctl place-items-center rounded-field bg-ink text-on-ink" title={es.app.name}>
              <Icon name="logo" size={22} />
            </span>
          </div>
          <div className="space-y-1.5">
            <h1 className="flex items-center gap-2 text-subtitle font-bold leading-tight tracking-tight">
              <Icon name="lock" size={22} className="shrink-0 text-ink-2" />
              {t.title}
            </h1>
            <p className="text-ui text-ink-2">{t.help}</p>
          </div>
          <TextInput label={t.label} type="password" inputMode="numeric" autoComplete="off" maxLength={8} autoFocus value={pin} onChange={(e) => setPin(e.target.value)} />
          {message && <Alert tone="warn">{message}</Alert>}
          <Button type="submit" variant="primary" className="w-full" disabled={busy || !pin}>
            {t.enter}
          </Button>
        </form>
      </Card>
    </div>
  );
}
