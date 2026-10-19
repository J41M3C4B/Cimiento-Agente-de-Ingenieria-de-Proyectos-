import { createContext, useContext } from "react";
import type { Permission, SessionView } from "./types";

/** The person inside the app, for the screens that adapt to what they may do. Rust decides; this only shows. */
export interface SessionApi {
  session: SessionView;
  can: (p: Permission) => boolean;
  lock: () => void;
  logout: () => void;
}

export const SessionContext = createContext<SessionApi | null>(null);

export function useSession(): SessionApi | null {
  return useContext(SessionContext);
}

/** The codes with which Rust says the session is not usable: the gate shows the entry screen again. */
export const ACCESS_LOST = new Set(["not_signed_in", "session_locked", "must_change_password"]);
const EVENT = "cimiento:access-lost";

export function notifyIfAccessLost(error: unknown) {
  const code = error && typeof error === "object" && "code" in error ? String((error as { code: unknown }).code) : "";
  if (ACCESS_LOST.has(code)) window.dispatchEvent(new Event(EVENT));
}

export function onAccessLost(handler: () => void): () => void {
  window.addEventListener(EVENT, handler);
  return () => window.removeEventListener(EVENT, handler);
}
