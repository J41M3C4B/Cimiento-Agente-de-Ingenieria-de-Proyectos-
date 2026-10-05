import { useState } from "react";
import { Icon } from "../../components/icons";
import { Alert, Button, TextInput } from "../../components/ui";
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
    <div className="flex min-h-screen items-center justify-center bg-stone-50 p-6 text-[15px] text-stone-900">
      <form
        className="w-full max-w-md space-y-5 rounded-2xl border border-stone-200 bg-white p-9 shadow-lift"
        onSubmit={(e) => {
          e.preventDefault();
          if (pin) void enter();
        }}
      >
        <span className="flex h-14 w-14 items-center justify-center rounded-full bg-blue-50 text-blue-800">
          <Icon name="lock" size={26} />
        </span>
        <h1 className="text-[22px] font-semibold">{t.title}</h1>
        <p>{t.help}</p>
        <TextInput label={t.label} type="password" inputMode="numeric" autoComplete="off" maxLength={8} autoFocus value={pin} onChange={(e) => setPin(e.target.value)} />
        {message && <Alert tone="warn">{message}</Alert>}
        <Button type="submit" variant="primary" disabled={busy || !pin}>
          {t.enter}
        </Button>
      </form>
    </div>
  );
}
