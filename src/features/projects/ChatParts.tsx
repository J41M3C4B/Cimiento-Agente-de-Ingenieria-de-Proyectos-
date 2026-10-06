import { useEffect, useState } from "react";
import { Icon } from "../../components/icons";
import { es } from "../../i18n/es-MX";
import type { TurnView } from "../../lib/types";

const t = es.conversation;

const reduceMotion = () =>
  typeof window !== "undefined" && typeof window.matchMedia === "function" && window.matchMedia("(prefers-reduced-motion: reduce)").matches;

/** The round mark of the assistant. It breathes while it works. */
export function Mark({ active }: { active?: boolean }) {
  return (
    <span
      aria-hidden="true"
      className={`msg-mark flex h-8 w-8 shrink-0 items-center justify-center rounded-full bg-gradient-to-br from-navy-700 to-navy-950 text-brass-300 shadow-card ${active ? "anim-breathe" : ""}`}
    >
      <Icon name="sparkles" size={15} />
    </span>
  );
}

/** A message that arrives is written out little by little, like someone typing; one that was already there is not. */
export function Typed({ text, onDone }: { text: string; onDone: () => void }) {
  const [shown, setShown] = useState(reduceMotion() ? text.length : 0);
  useEffect(() => {
    if (shown >= text.length) {
      onDone();
      return;
    }
    // about a second and a half for any length, a little faster for the longest
    const step = Math.max(2, Math.ceil(text.length / 85));
    const id = window.setTimeout(() => setShown((n) => Math.min(text.length, n + step)), 18);
    return () => window.clearTimeout(id);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [shown, text]);
  return (
    <>
      <span className="sr-only">{text}</span>
      <span aria-hidden="true">
        {text.slice(0, shown)}
        {shown < text.length && <span className="ml-0.5 inline-block h-[1em] w-[2px] translate-y-[2px] animate-pulse bg-stone-700" />}
      </span>
    </>
  );
}

export function Message({ turn, animate, onTyped }: { turn: TurnView; animate: boolean; onTyped: () => void }) {
  const mine = turn.role === "person";
  if (mine) {
    return (
      <div className="anim-rise flex flex-col items-end gap-1">
        <span className="text-[12px] font-medium text-stone-600">{t.you}</span>
        <div className="max-w-[82%] whitespace-pre-wrap rounded-2xl rounded-tr-md bg-navy-800 px-4 py-2.5 text-[15px] leading-relaxed text-white shadow-card">{turn.text}</div>
      </div>
    );
  }
  return (
    <div className="msg-assistant anim-rise flex gap-3">
      <Mark active={animate} />
      <div className="min-w-0 flex-1 space-y-1">
        <p className="msg-name text-[12px] font-semibold text-stone-600">{t.assistant}</p>
        <p className="max-w-[64ch] whitespace-pre-wrap rounded-2xl rounded-tl-md border border-stone-200 bg-stone-50 px-4 py-3 text-[15px] leading-relaxed text-stone-900 shadow-card">
          {animate ? <Typed text={turn.text} onDone={onTyped} /> : turn.text}
        </p>
      </div>
    </div>
  );
}

/** The assistant is working: its mark breathes and the line says what it is doing, changing every few seconds. */
export function Thinking({ phrases }: { phrases: readonly string[] }) {
  const [i, setI] = useState(0);
  useEffect(() => {
    const id = window.setInterval(() => setI((n) => n + 1), 2400);
    return () => window.clearInterval(id);
  }, []);
  return (
    <div role="status" className="anim-rise flex items-center gap-3">
      <Mark active />
      <p className="flex items-center gap-2 text-[14px] font-medium">
        <span key={i} className="anim-shine anim-rise">
          {phrases[i % phrases.length]}
        </span>
        <span aria-hidden="true" className="flex gap-1">
          {[0, 1, 2].map((d) => (
            <span key={d} className="anim-dot h-1.5 w-1.5 rounded-full bg-stone-500" style={{ animationDelay: `${d * 0.16}s` }} />
          ))}
        </span>
      </p>
    </div>
  );
}

/** What the assistant says at a later step of the chat, as plain text with its mark. */
export function Said({ children, active }: { children: React.ReactNode; active?: boolean }) {
  return (
    <div className="msg-assistant anim-rise flex gap-3">
      <Mark active={active} />
      <div className="min-w-0 flex-1 space-y-1">
        <p className="msg-name text-[12px] font-semibold text-stone-600">{t.assistant}</p>
        <div className="max-w-[64ch] rounded-2xl rounded-tl-md border border-stone-200 bg-stone-50 px-4 py-3 text-[15px] leading-relaxed text-stone-900 shadow-card">{children}</div>
      </div>
    </div>
  );
}

/** What the person answered by pressing a button, shown as their message. */
export function Mine({ text }: { text: string }) {
  return <Message turn={{ turn: -1, role: "person", kind: "why", level: null, text, options: [] }} animate={false} onTyped={() => undefined} />;
}
