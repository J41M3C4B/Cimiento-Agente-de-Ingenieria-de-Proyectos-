import { useQuery, useQueryClient } from "@tanstack/react-query";
import { useEffect, useState } from "react";
import type { ReactNode } from "react";
import { Alert, Button } from "../../components/ui";
import { es } from "../../i18n/es-MX";
import { devLoadFixture, toAppError } from "../../lib/tauri";
import { useSession } from "../access/session";
import { ONBOARDING_KEY, onboardingStatus, onboardingWelcomeDone } from "./api";
import type { OnboardingStatus } from "./api";
import { Setup, Welcome } from "./Welcome";
import { Wizard } from "./Wizard";

const o = es.onboarding;
const LATER = "cimiento.onboarding.later";
const RESUME = "cimiento:onboarding-resume";

/** From Inicio: the administrator who left the data to the direction takes them up again (the data steps open). */
export function resumeOnboarding() {
  try {
    sessionStorage.removeItem(LATER);
  } catch {
    // without storage the gate still reopens through the event
  }
  window.dispatchEvent(new Event(RESUME));
}

function readLater(): boolean {
  try {
    return sessionStorage.getItem(LATER) === "1";
  } catch {
    return false;
  }
}

/**
 * What a person sees after entering and before the app (ADR-031): the welcome, once per account; then, while the
 * institution has not finished its data, the steps to fill them. The administrator sees the setup first and may
 * leave the data to the direction; the direction and accounting cannot skip them. Rust decides what is missing.
 */
export function OnboardingGate({ children }: { children: ReactNode }) {
  const qc = useQueryClient();
  const access = useSession();
  const status = useQuery({ queryKey: ONBOARDING_KEY, queryFn: onboardingStatus });
  const [later, setLater] = useState(readLater);
  const [filling, setFilling] = useState(false);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const put = (s: OnboardingStatus) => qc.setQueryData(ONBOARDING_KEY, s);

  useEffect(() => {
    const again = () => {
      setLater(false);
      setFilling(true);
    };
    window.addEventListener(RESUME, again);
    return () => window.removeEventListener(RESUME, again);
  }, []);

  async function welcomed() {
    setBusy(true);
    try {
      await onboardingWelcomeDone();
      await status.refetch();
    } catch (e) {
      setError(toAppError(e).message);
    } finally {
      setBusy(false);
    }
  }

  function postpone() {
    try {
      sessionStorage.setItem(LATER, "1");
    } catch {
      // without storage it lasts until the window closes anyway
    }
    setLater(true);
  }

  async function finished() {
    // what the steps wrote shows everywhere at once
    await Promise.all([qc.invalidateQueries({ queryKey: ["profile"] }), qc.invalidateQueries({ queryKey: ["facilities"] })]);
  }

  async function example(name: "asilo" | "casa-hogar") {
    setBusy(true);
    setError(null);
    try {
      await devLoadFixture(name);
      await Promise.all([status.refetch(), finished()]);
    } catch (e) {
      setError(toAppError(e).message);
    } finally {
      setBusy(false);
    }
  }

  if (status.isLoading) return <p className="p-6 text-heading">{es.common.loading}</p>;
  if (status.isError || !status.data) return <Alert tone="error">{toAppError(status.error).message}</Alert>;
  const s = status.data;

  if (!s.welcomed) return <Welcome onDone={welcomed} busy={busy} />;
  if (s.done || (s.can_postpone && later)) return <>{children}</>;
  if (s.can_postpone && !filling) {
    const examples = import.meta.env.DEV && access?.can("settings") && (
      <p className="flex flex-wrap items-center justify-center gap-x-3 gap-y-1 text-small text-ink-3">
        {o.setup.examples}
        <Button size="sm" variant="plain" disabled={busy} onClick={() => example("asilo")}>
          {es.profile.devAsilo}
        </Button>
        <Button size="sm" variant="plain" disabled={busy} onClick={() => example("casa-hogar")}>
          {es.profile.devCasaHogar}
        </Button>
      </p>
    );
    return <Setup status={s} onFill={() => setFilling(true)} onLater={postpone} examples={examples} error={error} />;
  }
  return <Wizard status={s} onStatus={put} onFinished={finished} onBack={s.can_postpone ? () => setFilling(false) : undefined} />;
}
