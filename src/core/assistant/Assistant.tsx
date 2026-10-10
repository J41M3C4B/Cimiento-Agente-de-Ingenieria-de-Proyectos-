import { useQuery } from "@tanstack/react-query";
import { createContext, useContext, useEffect, useRef, useState } from "react";
import type { ReactNode } from "react";
import { Icon } from "../../components/icons";
import { ManualText } from "../../components/ManualHelp";
import { QuarantineDialog } from "../../components/QuarantineDialog";
import { Alert, Bubble, Button, Composer, Disclosure, IconButton, TextButton } from "../../components/ui";
import { es } from "../../i18n/es-MX";
import { assistantAsk, manualEntry, toAppError } from "../../lib/tauri";
import { useSession } from "../access/session";
import type { AssistantOutcome, AssistantTurn, Decision, ManualSource, QuarantineReport } from "../../lib/types";

const t = es.assistant;

/** What any screen may do with the Assistant: open it, about a field and with a question already written. */
type AssistantApi = { open: (about?: { field?: string; question?: string }) => void; toggle: () => void; isOpen: boolean };

const AssistantContext = createContext<AssistantApi | null>(null);

/** The Assistant, or `null` outside of the app (a form drawn alone, in a test). */
export const useAssistant = () => useContext(AssistantContext);

type Message =
  | { from: "person"; text: string }
  | { from: "assistant"; outcome: Exclude<AssistantOutcome, { status: "quarantine" }> }
  | { from: "error"; text: string };

/** The turns that go with a question: what the person asked and what was answered, in words. */
function turns(messages: Message[]): AssistantTurn[] {
  return messages.flatMap((m): AssistantTurn[] => {
    if (m.from === "person") return [{ from: "person", text: m.text }];
    if (m.from !== "assistant") return [];
    const o = m.outcome;
    return [{ from: "assistant", text: o.status === "answered" ? o.text : `(${t.manualIntro} ${o.hits.map((h) => h.title).join("; ")})` }];
  });
}

/** One entry of the manual, opened in place. */
function ManualHit({ hit }: { hit: ManualSource }) {
  const entry = useQuery({ queryKey: ["manual", hit.id], queryFn: () => manualEntry(hit.id), staleTime: Infinity });
  return <Disclosure title={hit.title}>{entry.data ? <ManualText entry={entry.data} /> : null}</Disclosure>;
}

/** An answer of the AI: its text, the entries of the manual it used and, maybe, the place to go. */
function Answer({ text, sources, place, onGo }: { text: string; sources: ManualSource[]; place: string | null; onGo: (place: string) => void }) {
  return (
    <Bubble from="assistant">
      <p className="whitespace-pre-line">{text}</p>
      {sources.length > 0 && (
        <p className="mt-2 text-small text-ink-3">
          {t.sources} {sources.map((s) => s.title).join(", ")}
        </p>
      )}
      {place && (
        <Button size="sm" variant="secondary" className="mt-3" onClick={() => onGo(place)}>
          {t.goTo(t.places[place] ?? place)}
          <Icon name="next" size={14} />
        </Button>
      )}
    </Bubble>
  );
}

/** The failures worth trying again, and those someone fixes in «Inteligencia artificial». */
const RETRY = ["offline", "busy", "unavailable"];
const SETTINGS = ["not_configured", "key_rejected", "budget"];

/**
 * The AI could not answer: what happened and what to do, said plainly (a key, the connection, a busy service), and
 * meanwhile what the manual says. Whoever may change the settings gets the way there; the rest, whom to ask.
 */
function Unavailable({ reason, hits, busy, onRetry, onGo }: { reason: string; hits: ManualSource[]; busy: boolean; onRetry: () => void; onGo: (place: string) => void }) {
  const access = useSession();
  const admin = access?.can("settings") ?? false;
  const help = t.reasonHelp[reason];
  return (
    <div className="space-y-3">
      <Alert tone="warn">
        <p>{t.reasons[reason] ?? t.reasons.unavailable}</p>
        {help && <p className="mt-1 text-ink-2">{admin ? help.admin : help.other}</p>}
        {(RETRY.includes(reason) || (admin && SETTINGS.includes(reason))) && (
          <div className="mt-3 flex flex-wrap gap-2">
            {RETRY.includes(reason) && (
              <Button size="sm" variant="secondary" disabled={busy} onClick={onRetry}>
                {t.retry}
              </Button>
            )}
            {admin && SETTINGS.includes(reason) && (
              <Button size="sm" variant="secondary" onClick={() => onGo("ai")}>
                {t.toSettings}
              </Button>
            )}
          </div>
        )}
      </Alert>
      <p className="text-small text-ink-3">{t.meanwhile}</p>
      {hits.length === 0 ? <p className="text-ui text-ink-2">{t.noHits}</p> : hits.map((h) => <ManualHit key={h.id} hit={h} />)}
    </div>
  );
}

/**
 * The Assistant (ADR-034 §4): a panel over any screen. It knows the page where the person is and, when the «?» of a
 * field opened it, that field. Rust decides everything (who answers, what it reads, where it may send the person);
 * this only draws the conversation, which lives here and is gone when the program closes.
 */
export function AssistantProvider({ screen, onGo, children }: { screen: string; onGo: (place: string) => void; children: ReactNode }) {
  const [isOpen, setOpen] = useState(false);
  const [messages, setMessages] = useState<Message[]>([]);
  const [draft, setDraft] = useState("");
  const [field, setField] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [held, setHeld] = useState<{ report: QuarantineReport; question: string } | null>(null);
  const end = useRef<HTMLDivElement>(null);
  const box = useRef<HTMLTextAreaElement>(null);

  const api: AssistantApi = {
    isOpen,
    toggle: () => setOpen((o) => !o),
    open: (about) => {
      setOpen(true);
      if (about?.field !== undefined) setField(about.field);
      if (about?.question) setDraft(about.question);
      window.setTimeout(() => box.current?.focus(), 0);
    },
  };

  // the last message in view. In braces: a newer WebView2 returns a promise from scrollIntoView, and an effect that
  // returns something that is not a function brings the whole screen down
  useEffect(() => {
    end.current?.scrollIntoView?.({ block: "end" });
  }, [messages, busy]);

  const fieldEntry = useQuery({ queryKey: ["manual", field], queryFn: () => manualEntry(field!), enabled: field !== null, staleTime: Infinity });

  async function send(question: string, decision?: Decision) {
    setBusy(true);
    const before = messages;
    try {
      const out = await assistantAsk({ screen, field, question, history: turns(before) }, decision);
      if (out.status === "quarantine") {
        setHeld({ report: out.report, question });
        return;
      }
      setHeld(null);
      setDraft("");
      setMessages([...before, { from: "person", text: out.asked }, { from: "assistant", outcome: out }]);
    } catch (e) {
      setMessages([...before, { from: "person", text: question }, { from: "error", text: toAppError(e).message }]);
      setDraft("");
    } finally {
      setBusy(false);
    }
  }

  const submit = () => {
    if (!busy && draft.trim()) void send(draft.trim());
  };

  return (
    <AssistantContext.Provider value={api}>
      {children}
      {isOpen && (
        <aside aria-label={t.title} className="fixed inset-y-0 right-0 z-[110] flex w-full max-w-[420px] flex-col border-l border-line bg-card shadow-float">
          <header className="flex items-center justify-between gap-3 px-5 pb-3 pt-5">
            <h2 className="flex items-center gap-2 text-subtitle font-bold">
              <Icon name="sparkles" size={20} />
              {t.title}
            </h2>
            <span className="flex items-center gap-1">
              {messages.length > 0 && <TextButton onClick={() => setMessages([])}>{t.clear}</TextButton>}
              <IconButton icon="x" label={t.close} variant="plain" size="sm" onClick={() => setOpen(false)} />
            </span>
          </header>

          <div className="flex min-h-0 flex-1 flex-col gap-3 overflow-y-auto px-5 pb-4">
            {messages.length === 0 && <p className="text-ui text-ink-2">{t.intro}</p>}
            {messages.map((m, i) =>
              m.from === "person" ? (
                <Bubble key={i} from="person">
                  {m.text}
                </Bubble>
              ) : m.from === "error" ? (
                <Alert key={i} tone="error">
                  {m.text}
                </Alert>
              ) : m.outcome.status === "answered" ? (
                <Answer key={i} text={m.outcome.text} sources={m.outcome.sources} place={m.outcome.go_to} onGo={onGo} />
              ) : (
                <Unavailable key={i} reason={m.outcome.reason} hits={m.outcome.hits} busy={busy} onRetry={() => void send(m.outcome.asked)} onGo={onGo} />
              ),
            )}
            {busy && <p className="text-small text-ink-3">{t.thinking}</p>}
            <div ref={end} />
          </div>

          <form
            className="space-y-2 border-t border-line px-5 py-4"
            onSubmit={(e) => {
              e.preventDefault();
              submit();
            }}
          >
            {field && (
              <p className="flex items-center gap-2 text-small text-ink-2">
                {t.fromField(fieldEntry.data?.title ?? field)}
                <IconButton icon="x" label={t.close} variant="plain" size="sm" onClick={() => setField(null)} />
              </p>
            )}
            <Composer
              send={
                <Button type="submit" variant="primary" aria-label={t.send} title={t.send} disabled={busy || !draft.trim()} className="w-ctl shrink-0 !px-0">
                  <Icon name="send" size={18} />
                </Button>
              }
            >
              <label className="flex min-w-0 flex-1">
                <span className="sr-only">{t.label}</span>
                <textarea
                  ref={box}
                  rows={2}
                  value={draft}
                  placeholder={t.placeholder}
                  onChange={(e) => setDraft(e.target.value)}
                  onKeyDown={(e) => {
                    if (e.key === "Enter" && !e.shiftKey && !e.nativeEvent.isComposing) {
                      e.preventDefault();
                      submit();
                    }
                  }}
                />
              </label>
            </Composer>
          </form>
        </aside>
      )}
      {held && (
        // over the panel, which is over the windows of the screen
        <div className="fixed inset-0 z-[115]">
        <QuarantineDialog
          report={held.report}
          busy={busy}
          onRedact={() => void send(held.question, "redact")}
          onNotPersonal={() => void send(held.question, "not_personal")}
          onCancel={() => setHeld(null)}
        />
        </div>
      )}
    </AssistantContext.Provider>
  );
}
