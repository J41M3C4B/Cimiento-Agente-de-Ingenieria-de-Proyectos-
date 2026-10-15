import { useQuery, useQueryClient } from "@tanstack/react-query";
import { createContext, useContext, useEffect, useRef, useState } from "react";
import type { ReactNode } from "react";
import { Icon } from "../../components/icons";
import { ManualText } from "../../components/ManualHelp";
import { QuarantineDialog } from "../../components/QuarantineDialog";
import { Alert, Bubble, Button, Composer, Disclosure, IconButton, TextButton } from "../../components/ui";
import { es } from "../../i18n/es-MX";
import { assistantAsk, captureAsk, institutionFillAllFromDocuments, institutionFillFromDocuments, manualEntry, toAppError } from "../../lib/tauri";
import { useSession } from "../access/session";
import { formatDate } from "../../lib/format";
import { usePlaceName } from "../places";
import { OVERVIEW_KEY } from "../profile/gaps";
import { fillAllNotice, fillNotice } from "../profile/fromDocuments";
import type { FillNotice } from "../profile/fromDocuments";
import type { AssistantOutcome, AssistantTurn, CaptureOutcome, Decision, DocumentSource, Fill, ManualSource, QuarantineReport, RecordsList } from "../../lib/types";

const t = es.assistant;

/**
 * What any screen may do with the Assistant: open it, about a field and with a question already written, or to fill
 * a window for the person («Llenarlo por mí», IA4).
 */
type AssistantApi = { open: (about?: { field?: string; question?: string }) => void; fill: (target: Fill) => void; toggle: () => void; isOpen: boolean };

const AssistantContext = createContext<AssistantApi | null>(null);

/** The Assistant, or `null` outside of the app (a form drawn alone, in a test). */
export const useAssistant = () => useContext(AssistantContext);

type Message =
  | { from: "person"; text: string }
  | { from: "assistant"; outcome: Exclude<AssistantOutcome, { status: "quarantine" }> }
  | { from: "capture"; target: Fill; outcome: Exclude<CaptureOutcome, { status: "quarantine" }> }
  | { from: "documents"; target: Fill; notice: FillNotice }
  | { from: "error"; text: string };

/** The turns that go with a question: what the person asked and what was answered, in words. */
function turns(messages: Message[], name: (place: string) => string): AssistantTurn[] {
  return messages.flatMap((m): AssistantTurn[] => {
    if (m.from === "person") return [{ from: "person", text: m.text }];
    if (m.from === "capture") return m.outcome.status === "proposed" ? [{ from: "assistant", text: captured(m.outcome, name(m.outcome.place)).join(" ") }] : [];
    if (m.from === "documents") return [{ from: "assistant", text: m.notice.text }];
    if (m.from !== "assistant") return [];
    const o = m.outcome;
    if (o.status === "answered")
      return [{ from: "assistant", text: [o.text, ...(o.records ? listed(o.records) : [])].join(" "), documents: [...new Set(o.documents.map((d) => d.document_id))] }];
    return [{ from: "assistant", text: `(${t.manualIntro} ${o.hits.map((h) => h.title).join("; ")})` }];
  });
}

/** The documents of the Expediente by how they are, in the program's words: the AI never lists nor counts them. */
function listed(r: RecordsList): string[] {
  const dated = (items: RecordsList["expiring"]) => items.map((d) => t.recordsDated(d.name, formatDate(d.date) || d.date)).join(", ");
  return [
    ...(r.have.length > 0 ? [t.recordsHave(r.have.length, r.have.join(", "))] : []),
    ...(r.missing.length > 0 ? [t.recordsMissing(r.missing.length, r.missing.join(", "))] : []),
    ...(r.in_progress.length > 0 ? [t.recordsInProgress(r.in_progress.join(", "))] : []),
    ...(r.expiring.length > 0 ? [t.recordsExpiring(dated(r.expiring))] : []),
    ...(r.expired.length > 0 ? [t.recordsExpired(dated(r.expired))] : []),
  ];
}

/** One entry of the manual, opened in place. */
function ManualHit({ hit }: { hit: ManualSource }) {
  const entry = useQuery({ queryKey: ["manual", hit.id], queryFn: () => manualEntry(hit.id), staleTime: Infinity });
  return <Disclosure title={hit.title}>{entry.data ? <ManualText entry={entry.data} /> : null}</Disclosure>;
}

/** An answer of the AI: its text, the entries of the manual and the documents it used and, maybe, the place to go, a
 * window to fill with what the person tells, and one to fill with the documents. */
function Answer({
  text,
  sources,
  documents,
  records,
  place,
  fill,
  fromDocuments,
  busy,
  onGo,
  onFill,
  onFromDocuments,
}: {
  text: string;
  sources: ManualSource[];
  documents: DocumentSource[];
  records: RecordsList | null;
  place: string | null;
  fill: Fill | null;
  fromDocuments: Fill | null;
  busy: boolean;
  onGo: (place: string) => void;
  onFill: (target: Fill) => void;
  onFromDocuments: (target: Fill) => void;
}) {
  const name = usePlaceName();
  return (
    <Bubble from="assistant">
      <p className="whitespace-pre-line">{text}</p>
      {records &&
        listed(records).map((line, i) => (
          <p key={i} className="mt-2">
            {line}
          </p>
        ))}
      {sources.length > 0 && (
        <p className="mt-2 text-small text-ink-3">
          {t.sources} {sources.map((s) => s.title).join(", ")}
        </p>
      )}
      {documents.length > 0 && (
        <p className="mt-2 text-small text-ink-3">
          {t.documentSources} {documents.map((d) => t.documentPage(d.document, d.page)).join("; ")}
        </p>
      )}
      {(place || fill || fromDocuments) && (
        <div className="mt-3 flex flex-wrap gap-2">
          {place && (
            <Button size="sm" variant="secondary" onClick={() => onGo(place)}>
              {t.goTo(name(place))}
              <Icon name="next" size={14} />
            </Button>
          )}
          {fill && (
            <Button size="sm" variant="secondary" onClick={() => onFill(fill)}>
              <Icon name="sparkles" size={14} />
              {t.fillMe}
            </Button>
          )}
          {fromDocuments && (
            <Button size="sm" variant="secondary" disabled={busy} onClick={() => onFromDocuments(fromDocuments)}>
              <Icon name="sparkles" size={14} />
              {fromDocuments.form === "all" ? es.suggestions.fillAllFromDocuments : es.suggestions.fillFromDocuments}
            </Button>
          )}
        </div>
      )}
    </Bubble>
  );
}

/** What the Capturist left, in the program's words: what the code kept, what it did not take and what is still to
 * tell. Never the AI's words, which may promise what the code dropped. */
function captured(outcome: Extract<CaptureOutcome, { status: "proposed" }>, place: string): string[] {
  const n = outcome.fields.length;
  return [
    n > 0 ? t.leftWhich(n, place, outcome.left) : t.leftNone(place),
    ...(outcome.dropped > 0 ? [t.notTaken] : []),
    ...(outcome.still.length > 0 ? [t.still(outcome.still)] : []),
  ];
}

/** What the Capturist left and, when it left suggestions, the way to the window to review them. */
function Captured({ outcome, onGo }: { outcome: Extract<CaptureOutcome, { status: "proposed" }>; onGo: (place: string) => void }) {
  const name = usePlaceName();
  return (
    <Bubble from="assistant">
      {captured(outcome, name(outcome.place)).map((line, i) => (
        <p key={i} className={i > 0 ? "mt-2" : ""}>
          {line}
        </p>
      ))}
      {outcome.fields.length > 0 && (
        <Button size="sm" variant="secondary" className="mt-3" onClick={() => onGo(outcome.place)}>
          {t.review}
          <Icon name="next" size={14} />
        </Button>
      )}
    </Bubble>
  );
}

/** What «Llenar con mis documentos» left, in the program's words, and the way to the window to review it. */
function FromDocuments({ target, notice, onGo }: { target: Fill; notice: FillNotice; onGo: (place: string) => void }) {
  if (notice.tone === "error") return <Alert tone="warn">{notice.text}</Alert>;
  return (
    <Bubble from="assistant">
      <p>{notice.text}</p>
      {notice.tone === "ok" && (
        <Button size="sm" variant="secondary" className="mt-3" onClick={() => onGo(target.place)}>
          {t.review}
          <Icon name="next" size={14} />
        </Button>
      )}
    </Bubble>
  );
}

/** The failures worth trying again, and those someone fixes in «Ayuda automática». */
const RETRY = ["offline", "busy", "unavailable"];
const SETTINGS = ["not_configured", "key_rejected", "budget"];

/**
 * The AI could not answer: what happened and what to do, said plainly (a key, the connection, a busy service), and
 * meanwhile what the manual says (when there is a manual answer: filling a window has none). Whoever may change the
 * settings gets the way there; the rest, whom to ask.
 */
function Unavailable({ reason, hits, busy, onRetry, onGo }: { reason: string; hits?: ManualSource[]; busy: boolean; onRetry: () => void; onGo: (place: string) => void }) {
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
      {hits && (
        <>
          <p className="text-small text-ink-3">{t.meanwhile}</p>
          {hits.length === 0 ? <p className="text-ui text-ink-2">{t.noHits}</p> : hits.map((h) => <ManualHit key={h.id} hit={h} />)}
        </>
      )}
    </div>
  );
}

/**
 * The Assistant (ADR-034 §4): a panel over any screen. It knows the page where the person is and, when the «?» of a
 * field opened it, that field. Rust decides everything (who answers, what it reads, where it may send the person);
 * this only draws the conversation, which lives here and is gone when the program closes.
 *
 * While it fills a window («Llenarlo por mí»), what the person writes goes to the Capturist, which leaves suggestions
 * in that window; the person reviews them there.
 */
export function AssistantProvider({ screen, onGo, children }: { screen: string; onGo: (place: string) => void; children: ReactNode }) {
  const [isOpen, setOpen] = useState(false);
  const [messages, setMessages] = useState<Message[]>([]);
  const [draft, setDraft] = useState("");
  const [field, setField] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [filling, setFilling] = useState<Fill | null>(null);
  const [held, setHeld] = useState<{ report: QuarantineReport; question: string; target: Fill | null } | null>(null);
  const qc = useQueryClient();
  const placeName = usePlaceName();
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
    fill: (target) => {
      setOpen(true);
      setField(null);
      setFilling(target);
      window.setTimeout(() => box.current?.focus(), 0);
    },
  };

  // the last message in view. In braces: a newer WebView2 returns a promise from scrollIntoView, and an effect that
  // returns something that is not a function brings the whole screen down
  useEffect(() => {
    end.current?.scrollIntoView?.({ block: "end" });
  }, [messages, busy]);

  const fieldEntry = useQuery({ queryKey: ["manual", field], queryFn: () => manualEntry(field!), enabled: field !== null, staleTime: Infinity });

  /**
   * What the person tells, to the Capturist: the suggestions go to the window and the panel says how many. From an
   * answer («Llenarlo por mí»), what was asked is already in the conversation and is not shown again (`echo`).
   */
  async function capture(target: Fill, said: string, history: AssistantTurn[], decision?: Decision, echo = true) {
    setBusy(true);
    const before = messages;
    const asked: Message[] = echo ? [{ from: "person", text: said }] : [];
    try {
      const out = await captureAsk({ form: target.form, said, history }, decision);
      if (out.status === "quarantine") {
        setHeld({ report: out.report, question: said, target });
        return;
      }
      setHeld(null);
      setDraft("");
      setMessages([...before, ...(echo ? [{ from: "person" as const, text: out.asked }] : []), { from: "capture", target, outcome: out }]);
      if (out.status === "proposed") {
        // the window of that form (`formKey` of FormWindow) and the notice of «Mi institución»
        void qc.invalidateQueries({ queryKey: ["form", out.form] });
        void qc.invalidateQueries({ queryKey: OVERVIEW_KEY });
      }
    } catch (e) {
      setMessages([...before, ...asked, { from: "error", text: toAppError(e).message }]);
      setDraft("");
    } finally {
      setBusy(false);
    }
  }

  /**
   * «Llenar con mis documentos» under an answer (C3): the Reader of the Expediente leaves suggestions in that window
   * (or in all of them), each with its document and page; the panel says how many, in the program's words.
   */
  async function fromDocuments(target: Fill) {
    setBusy(true);
    const before = messages;
    try {
      const notice = target.form === "all" ? fillAllNotice(await institutionFillAllFromDocuments()) : fillNotice(await institutionFillFromDocuments(target.form));
      setMessages([...before, { from: "documents", target, notice }]);
      void qc.invalidateQueries({ queryKey: ["form"] });
      void qc.invalidateQueries({ queryKey: OVERVIEW_KEY });
    } catch (e) {
      setMessages([...before, { from: "error", text: toAppError(e).message }]);
    } finally {
      setBusy(false);
    }
  }

  /** «Llenarlo por mí» under an answer: what the person asked just before it is what the Capturist reads. */
  function fillFrom(at: number, target: Fill) {
    const asked = messages[at - 1];
    setFilling(target);
    if (asked?.from === "person") void capture(target, asked.text, turns(messages.slice(0, at - 1), placeName), undefined, false);
  }

  async function send(question: string, decision?: Decision) {
    if (filling) return capture(filling, question, turns(messages, placeName), decision);
    setBusy(true);
    const before = messages;
    try {
      const out = await assistantAsk({ screen, field, question, history: turns(before, placeName) }, decision);
      if (out.status === "quarantine") {
        setHeld({ report: out.report, question, target: null });
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
              ) : m.from === "documents" ? (
                <FromDocuments key={i} target={m.target} notice={m.notice} onGo={onGo} />
              ) : m.from === "capture" ? (
                m.outcome.status === "proposed" ? (
                  <Captured key={i} outcome={m.outcome} onGo={onGo} />
                ) : (
                  <Unavailable
                    key={i}
                    reason={m.outcome.reason}
                    busy={busy}
                    onRetry={() => void capture(m.target, m.outcome.asked, turns(messages.slice(0, i - 1), placeName), undefined, false)}
                    onGo={onGo}
                  />
                )
              ) : m.outcome.status === "answered" ? (
                <Answer
                  key={i}
                  text={m.outcome.text}
                  sources={m.outcome.sources}
                  documents={m.outcome.documents ?? []}
                  records={m.outcome.records ?? null}
                  place={m.outcome.go_to}
                  fill={m.outcome.fill}
                  fromDocuments={m.outcome.from_documents ?? null}
                  busy={busy}
                  onGo={onGo}
                  onFill={(f) => fillFrom(i, f)}
                  onFromDocuments={(f) => void fromDocuments(f)}
                />
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
            {filling && (
              <div className="space-y-1 text-small text-ink-2">
                <p className="flex items-center gap-2 font-semibold text-ink">
                  <Icon name="sparkles" size={14} />
                  {t.filling(placeName(filling.place))}
                  <TextButton onClick={() => setFilling(null)}>{t.stopFilling}</TextButton>
                </p>
                <p>{t.fillIntro(placeName(filling.place))}</p>
              </div>
            )}
            {field && !filling && (
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
                  placeholder={filling ? t.fillPlaceholder : t.placeholder}
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
          onRedact={() => void (held.target ? capture(held.target, held.question, turns(messages, placeName), "redact") : send(held.question, "redact"))}
          onNotPersonal={() => void (held.target ? capture(held.target, held.question, turns(messages, placeName), "not_personal") : send(held.question, "not_personal"))}
          onCancel={() => setHeld(null)}
        />
        </div>
      )}
    </AssistantContext.Provider>
  );
}
