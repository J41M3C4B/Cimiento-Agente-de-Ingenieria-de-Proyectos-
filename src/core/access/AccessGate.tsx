import { useQuery, useQueryClient } from "@tanstack/react-query";
import { useCallback, useEffect, useMemo, useRef } from "react";
import type { ReactNode } from "react";
import { es } from "../../i18n/es-MX";
import { accessLock, accessLogout, accessStatus } from "./api";
import { AccessScreen } from "./AccessScreen";
import { onAccessLost, SessionContext } from "./session";
import type { SessionApi } from "./session";
import type { AccessStatus, SessionView } from "./types";

const KEY = ["access-status"] as const;

/**
 * Nothing of the app shows until a person is inside (ADR-028). After a while without use the screen locks (Rust
 * keeps the lock; this counts the time). If Rust says the session is not usable, the entry screen comes back.
 */
export function AccessGate({ children }: { children: ReactNode }) {
  const qc = useQueryClient();
  const status = useQuery({ queryKey: KEY, queryFn: accessStatus });
  const refresh = useCallback(() => void qc.invalidateQueries({ queryKey: KEY }), [qc]);
  // what the previous person saw goes away with them
  const forget = useCallback(() => qc.removeQueries({ predicate: (q) => q.queryKey[0] !== KEY[0] }), [qc]);
  const session = status.data?.session ?? null;
  const usable = session !== null && !session.locked && !session.must_change_password;

  useEffect(() => onAccessLost(refresh), [refresh]);

  // the idle clock: any use of the mouse or the keyboard starts it again
  const last = useRef(Date.now());
  const minutes = status.data?.idle_minutes ?? 15;
  useEffect(() => {
    if (!usable) return;
    const touch = () => (last.current = Date.now());
    const events = ["pointerdown", "keydown", "wheel", "pointermove"] as const;
    events.forEach((e) => window.addEventListener(e, touch, { passive: true }));
    last.current = Date.now();
    const timer = window.setInterval(() => {
      if (Date.now() - last.current > minutes * 60_000) void accessLock().then(refresh, refresh);
    }, 15_000);
    return () => {
      events.forEach((e) => window.removeEventListener(e, touch));
      window.clearInterval(timer);
    };
  }, [usable, minutes, refresh]);

  const api = useMemo<SessionApi | null>(
    () =>
      session && {
        session,
        can: (p) => session.permissions.includes(p),
        lock: () => void accessLock().then(refresh, refresh),
        logout: () =>
          void accessLogout().then(() => {
            forget();
            refresh();
          }, refresh),
      },
    [session, forget, refresh],
  );

  if (status.isLoading || !status.data) return <p className="p-6 text-heading">{es.common.loading}</p>;
  if (!usable || !api) {
    const show: AccessStatus = status.data;
    return (
      <AccessScreen
        key={`${show.needs_setup}-${session?.user_id ?? "none"}-${session?.locked}`}
        status={show}
        onSession={(s: SessionView | null) => {
          qc.setQueryData<AccessStatus>(KEY, (old) => old && { ...old, needs_setup: false, setup_needs_pin: false, session: s });
          if (!s) forget();
        }}
      />
    );
  }
  return <SessionContext.Provider value={api}>{children}</SessionContext.Provider>;
}
