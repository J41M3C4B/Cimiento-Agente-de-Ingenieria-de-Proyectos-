import { useQueryClient } from "@tanstack/react-query";
import { useEffect, useRef, useState } from "react";
import type { ReactNode } from "react";
import { QuarantineDialog } from "../../components/QuarantineDialog";
import { Icon } from "../../components/icons";
import { Alert, Button, Segments } from "../../components/ui";
import { es } from "../../i18n/es-MX";
import { conversationRetry, conversationSend, conversationStart, toAppError } from "../../lib/tauri";
import type { AnswerOutcome, ConversationView, Decision, QuarantineReport } from "../../lib/types";
import { Message, Thinking } from "./ChatParts";

const t = es.conversation;

/** Where the conversation is, in words: the opening, which «why» of how many, the root cause, or done. */
export function progressLabel(view: ConversationView): string {
  if (view.phase === "closed") return t.progressDone;
  if (view.phase === "root_proposed") return t.progressRoot;
  if (view.why_level > 0) return t.progressWhy(view.why_level, view.max_whys);
  return t.progressOpening;
}

/**
 * The guided conversation of the diagnosis, as a working conversation: the assistant's messages are plain text with
 * its mark, the person's are a quiet block on the right, and while the AI works it shows what it is doing and then
 * writes its answer out. The AI writes every message; the code decides the steps. If the AI fails, what the person
 * wrote stays and «Intentar otra vez» asks again (ADR-017).
 */
/** What the person can still write once the conversation is closed (a later step may take it, e.g. their own objective). */
export type LaterWriting = {
  placeholder: string;
  hint: string;
  /** Without it the box stays disabled and only says where to go on. */
  send?: (text: string, decision?: Decision) => Promise<{ status: "saved" } | { status: "quarantine"; report: QuarantineReport }>;
};

export function ConversationChat({
  view,
  onView,
  disabled,
  tail,
  tailKey,
  later,
  backgroundTurn = false,
}: {
  view: ConversationView;
  onView: (v: ConversationView) => void;
  disabled: boolean;
  /** What the assistant does after the conversation, as part of the same chat (the objectives, for example). */
  tail?: ReactNode;
  /** Changes when the tail changes what it shows, so the chat scrolls to it. */
  tailKey?: string;
  later?: LaterWriting;
  /** A turn of this conversation is being written by the AI and was started elsewhere (before the person left and came back). */
  backgroundTurn?: boolean;
}) {
  const project = view.project.id;
  const qc = useQueryClient();
  const [text, setText] = useState("");
  const [working, setWorking] = useState(false);
  const [sending, setSending] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [pending, setPending] = useState<{ text: string; report: QuarantineReport; later: boolean } | null>(null);
  const [correcting, setCorrecting] = useState(false);
  const [typed, setTyped] = useState<Set<number>>(new Set());
  const started = useRef(false);
  // what was already there when the person arrived is not written out again
  const arrived = useRef(view.turns.reduce((m, x) => Math.max(m, x.turn), 0));
  const box = useRef<HTMLTextAreaElement>(null);
  const end = useRef<HTMLDivElement>(null);

  async function run(fn: () => Promise<AnswerOutcome>, sent?: string) {
    setWorking(true);
    setError(null);
    setSending(sent ?? null);
    try {
      const out = await fn();
      if (out.status === "quarantine") {
        setPending({ text: sent ?? "", report: out.report, later: false });
        return;
      }
      setPending(null);
      setNotice(out.ai === "used" || out.ai === "skipped" ? null : es.aiNotice[out.ai]);
      setText("");
      setCorrecting(false);
      onView(out.view);
    } catch (e) {
      const err = toAppError(e);
      // the AI is already writing this message (it started before the person left): wait for it, it is not a failure
      if (err.code === "already_running") void qc.invalidateQueries({ queryKey: ["project-job", project] });
      else setError(err.message);
    } finally {
      setWorking(false);
      setSending(null);
    }
  }

  async function runLater(msg: string, decision?: Decision) {
    if (!later?.send) return;
    setWorking(true);
    setError(null);
    setSending(msg);
    try {
      const out = await later.send(msg, decision);
      if (out.status === "quarantine") {
        setPending({ text: msg, report: out.report, later: true });
        return;
      }
      setPending(null);
      setText("");
    } catch (e) {
      setError(toAppError(e).message);
    } finally {
      setWorking(false);
      setSending(null);
    }
  }

  const closed = view.phase === "closed";
  const afterwards = closed && later?.send !== undefined;
  const send = (msg: string, decision?: Decision) => run(() => conversationSend(project, msg, false, decision), msg);
  const confirmRoot = () => run(() => conversationSend(project, "", true, undefined));
  const retry = () => run(() => (view.phase === "needs_opening" ? conversationStart(project) : conversationRetry(project)));

  // the first question is asked as soon as the person arrives (once: asking again changes nothing and costs nothing)
  useEffect(() => {
    if (view.phase === "needs_opening" && !started.current && !disabled && !view.legacy) {
      started.current = true;
      void run(() => conversationStart(project));
    }
    // `disabled` is a dependency because the parent holds the chat back until it knows whether the AI is already
    // writing this message (the person left and came back): then it starts only if nothing is running.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [view.phase, disabled]);

  useEffect(() => {
    end.current?.scrollIntoView?.({ block: "end", behavior: "smooth" });
  }, [view.turns.length, sending, working, typed.size, tailKey]);

  const last = view.turns[view.turns.length - 1];
  const asking = view.phase === "awaiting_answer" || view.phase === "root_proposed";
  const canWrite = (asking || afterwards) && !working && !disabled;
  const submit = (msg: string, decision?: Decision) => (afterwards ? runLater(msg, decision) : send(msg, decision));
  const owed = (view.phase === "awaiting_ai" || view.phase === "needs_opening") && !working && !disabled;
  // while the latest message is being written out, the answers wait
  const writing = last?.role === "assistant" && last.turn > arrived.current && !typed.has(last.turn);
  const quick = canWrite && !writing && last?.role === "assistant" ? last.options : [];
  const proposing = view.phase === "root_proposed" && canWrite;

  return (
    <section className="flex h-full min-h-0 flex-col" aria-label={t.title}>
      <header className="shrink-0 px-6 pt-5">
        <div className="mx-auto flex w-full max-w-[720px] items-center gap-4">
          <p className="shrink-0 text-[13px] font-medium text-stone-500">{progressLabel(view)}</p>
          <div className="w-40 max-w-full">
            <Segments
              total={view.max_whys + 1}
              filled={view.phase === "closed" || view.phase === "root_proposed" ? view.max_whys + 1 : view.why_level + 1}
              label={progressLabel(view)}
            />
          </div>
        </div>
      </header>

      <div role="log" aria-live="polite" className="min-h-0 flex-1 overflow-y-auto">
        <div className="mx-auto w-full max-w-[720px] space-y-7 px-6 pb-8 pt-6">
          {view.fit && view.fit.fit !== "fits" && (
            <Alert tone="warn">
              <p className="font-semibold">{view.fit.fit === "mismatch" ? t.fitMismatch : t.fitPartial}</p>
              {view.fit.note && <p>{view.fit.note}</p>}
            </Alert>
          )}
          {view.turns.map((turn) => (
            <Message
              key={turn.turn}
              turn={turn}
              animate={turn.role === "assistant" && turn.turn > arrived.current && !typed.has(turn.turn)}
              onTyped={() => setTyped((s) => new Set(s).add(turn.turn))}
            />
          ))}
          {sending && (
            <Message turn={{ turn: -1, role: "person", kind: "why", level: null, text: sending, options: [] }} animate={false} onTyped={() => undefined} />
          )}
          {(working || backgroundTurn) && <Thinking phrases={view.turns.length === 0 ? t.thinkingFirst : t.thinking} />}

          {notice && <Alert tone="warn">{notice}</Alert>}
          {error && <Alert tone="error">{error}</Alert>}
          {view.legacy && !view.summary && <Alert tone="info">{t.legacy}</Alert>}
          {owed && !view.legacy && (
            <div>
              <Button variant="primary" onClick={retry}>
                {t.retry}
              </Button>
            </div>
          )}
          {closed && !writing && (
            <p role="status" className="anim-rise flex items-center gap-3 text-[13px] font-medium text-stone-600">
              <span aria-hidden="true" className="h-px flex-1 bg-stone-200" />
              {t.ended}
              <span aria-hidden="true" className="h-px flex-1 bg-stone-200" />
            </p>
          )}
          {closed && !writing && tail}
          <div ref={end} />
        </div>
      </div>

      {/* the box is always here, like in any chat: when there is nothing to write it says where to go on */}
      <footer className="shrink-0 px-6 pb-5 pt-2">
        <div className="mx-auto w-full max-w-[720px] space-y-3">
          {quick.length > 0 && (
            <div className="flex flex-wrap gap-2" aria-label={t.quickReplies}>
              {quick.map((option, i) => (
                <button
                  key={option}
                  type="button"
                  onClick={() => {
                    if (proposing && i === 0) void confirmRoot();
                    else if (proposing) {
                      setCorrecting(true);
                      box.current?.focus();
                    } else void send(option);
                  }}
                  className={`rounded-full px-4 py-2 text-[14px] font-medium transition-colors ${
                    proposing && i === 0
                      ? "bg-stone-900 text-white hover:bg-stone-700"
                      : "bg-white text-stone-900 shadow-card hover:bg-stone-50"
                  }`}
                >
                  {option}
                </button>
              ))}
            </div>
          )}
          <form
            className="flex items-end gap-2 rounded-[24px] bg-white p-2.5 pl-3 shadow-float"
            onSubmit={(e) => {
              e.preventDefault();
              if (canWrite && text.trim()) void submit(text);
            }}
          >
            <label className="flex-1">
              <span className="sr-only">{t.answerLabel}</span>
              <textarea
                ref={box}
                rows={2}
                value={text}
                disabled={!canWrite}
                placeholder={later && closed ? later.placeholder : closed ? t.placeholderClosed : correcting ? t.placeholderCorrect : t.placeholder}
                onChange={(e) => setText(e.target.value)}
                onKeyDown={(e) => {
                  if (e.key === "Enter" && !e.shiftKey && !e.nativeEvent.isComposing) {
                    e.preventDefault();
                    if (canWrite && text.trim()) void submit(text);
                  }
                }}
                className="w-full resize-none rounded-lg border-0 bg-transparent px-2 py-2 text-[15px] text-stone-900 outline-none placeholder:text-stone-500 focus-visible:outline-none disabled:opacity-60"
              />
            </label>
            <Button type="submit" variant="primary" aria-label={t.send} title={t.send} disabled={!canWrite || !text.trim()} className="!h-10 !min-h-0 !w-10 shrink-0 !rounded-full !p-0">
              <Icon name="send" size={16} />
            </Button>
          </form>
          <p className="px-2 text-center text-[12px] text-stone-500">{later && closed ? later.hint : closed ? t.hintClosed : t.hint}</p>
        </div>
      </footer>

      {pending && (
        <QuarantineDialog
          report={pending.report}
          busy={working}
          onRedact={() => (pending.later ? runLater(pending.text, "redact") : send(pending.text, "redact"))}
          onNotPersonal={() => (pending.later ? runLater(pending.text, "not_personal") : send(pending.text, "not_personal"))}
          onCancel={() => setPending(null)}
        />
      )}
    </section>
  );
}
