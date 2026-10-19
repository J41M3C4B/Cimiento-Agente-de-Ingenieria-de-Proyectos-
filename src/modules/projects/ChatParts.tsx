import { useEffect, useState } from "react";
import { Avatar, Bubble } from "../../components/ui";
import { es } from "../../i18n/es-MX";
import type { TurnView } from "../../lib/types";

const t = es.conversation;

const reduceMotion = () =>
  typeof window !== "undefined" && typeof window.matchMedia === "function" && window.matchMedia("(prefers-reduced-motion: reduce)").matches;

/** The small mark of the assistant: the black «AI» avatar that comes before each of its messages. */
export function Mark({ active }: { active?: boolean }) {
  return (
    <span className={`mt-0.5 shrink-0 ${active ? "opacity-90" : ""}`}>
      <Avatar name="A I" tone="ink" />
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
        {shown < text.length && <span className="ml-0.5 inline-block h-[1em] w-[2px] translate-y-[2px] animate-pulse bg-ink-3" />}
      </span>
    </>
  );
}

export function Message({ turn, animate, onTyped }: { turn: TurnView; animate: boolean; onTyped: () => void }) {
  const mine = turn.role === "person";
  if (mine) {
    return (
      <Bubble from="person">
        <span className="sr-only">{t.you}</span>
        <span className="whitespace-pre-wrap">{turn.text}</span>
      </Bubble>
    );
  }
  return (
    <Bubble from="assistant" avatar={<Mark />}>
      <span className="sr-only">{t.assistant}</span>
      <span className="whitespace-pre-wrap">{animate ? <Typed text={turn.text} onDone={onTyped} /> : turn.text}</span>
    </Bubble>
  );
}

/** The assistant is working: its mark and the line that says what it is doing, changing every few seconds, with three dots. */
export function Thinking({ phrases }: { phrases: readonly string[] }) {
  const [i, setI] = useState(0);
  useEffect(() => {
    const id = window.setInterval(() => setI((n) => n + 1), 2400);
    return () => window.clearInterval(id);
  }, []);
  return (
    <div role="status">
      <Bubble from="assistant" avatar={<Mark active />}>
        <p className="flex items-center gap-2 text-ui font-semibold text-ink-2">
          <span key={i} className="anim-shine anim-rise">
            {phrases[i % phrases.length]}
          </span>
          <span aria-hidden="true" className="flex gap-1">
            {[0, 1, 2].map((d) => (
              <span key={d} className="anim-dot h-1.5 w-1.5 rounded-pill bg-ink-3" style={{ animationDelay: `${d * 0.16}s` }} />
            ))}
          </span>
        </p>
      </Bubble>
    </div>
  );
}

/** What the assistant says at a later step of the chat. */
export function Said({ children }: { children: React.ReactNode; active?: boolean }) {
  return (
    <Bubble from="assistant" avatar={<Mark />}>
      <span className="sr-only">{t.assistant}</span>
      <div>{children}</div>
    </Bubble>
  );
}

/** What the person answered by pressing a button, shown as their message. */
export function Mine({ text }: { text: string }) {
  return <Message turn={{ turn: -1, role: "person", kind: "why", level: null, text, options: [] }} animate={false} onTyped={() => undefined} />;
}
