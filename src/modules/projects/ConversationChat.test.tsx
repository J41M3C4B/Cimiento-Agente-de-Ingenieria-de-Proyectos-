import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { ConversationPhase, ConversationView, ProjectRow, TurnView } from "../../lib/types";

vi.mock("../../lib/tauri", () => ({
  conversationStart: vi.fn(),
  conversationSend: vi.fn(),
  conversationRetry: vi.fn(),
  toAppError: (e: unknown) => ({ code: "x", message: String(e) }),
}));

import * as api from "../../lib/tauri";
import { ConversationChat, progressLabel } from "./ConversationChat";

const project: ProjectRow = {
  id: "proj_1",
  institution_id: "i",
  profile_id: "p",
  title: "Apoyos 2027",
  initial_request: null,
  stage: "DIAGNOSIS",
  needs_review: false,
  created_at: "2026-10-03T10:00:00Z",
  kind: "call",
  call_reading_id: "read_1",
  color: null,
  donor_kind: null,
};

const ai = (turn: number, text: string, extra: Partial<TurnView> = {}): TurnView => ({ turn, role: "assistant", kind: "why", level: null, text, options: [], ...extra });
const me = (turn: number, text: string): TurnView => ({ turn, role: "person", kind: "why", level: 1, text, options: [] });

const view = (phase: ConversationPhase, turns: TurnView[], over: Partial<ConversationView> = {}): ConversationView => ({
  project,
  call: { name: "Apoyos 2027", funder: "Fundación Ficticia", year: 2027 },
  turns,
  phase,
  why_level: 0,
  max_whys: 5,
  fit: null,
  root: null,
  summary: null,
  unsupported_figures: [],
  legacy: false,
  ...over,
});

function show(v: ConversationView, onView = vi.fn()) {
  const qc = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  render(
    <QueryClientProvider client={qc}>
      <ConversationChat view={v} onView={onView} disabled={false} />
    </QueryClientProvider>,
  );
  return onView;
}

beforeEach(() => {
  vi.resetAllMocks();
});

describe("progressLabel", () => {
  it("says where the conversation is in plain words", () => {
    expect(progressLabel(view("awaiting_answer", [ai(1, "Hola", { kind: "opening" })]))).toBe("Para empezar");
    expect(progressLabel(view("awaiting_answer", [], { why_level: 2 }))).toBe("Porqué 2 de 5");
    expect(progressLabel(view("root_proposed", []))).toBe("La causa de fondo");
    expect(progressLabel(view("closed", []))).toBe("Causa de fondo confirmada");
  });
});

describe("ConversationChat", () => {
  it("asks for the first question by itself when the person arrives, and shows it as a message", async () => {
    const opened = view("awaiting_answer", [ai(1, "Hola, ¿qué proyecto tienen en mente?", { kind: "opening" })]);
    vi.mocked(api.conversationStart).mockResolvedValue({ status: "saved", view: opened, ai: "used" });
    const onView = show(view("needs_opening", []));
    await waitFor(() => expect(api.conversationStart).toHaveBeenCalledWith("proj_1"));
    await waitFor(() => expect(onView).toHaveBeenCalledWith(opened));
  });

  it("shows the messages of both and how far along it is (the call is beside the chat, not in it)", () => {
    show(view("awaiting_answer", [ai(1, "¿Qué proyecto tienen en mente?", { kind: "opening" }), me(2, "Arreglar la cocina")], { why_level: 1 }));
    expect(screen.queryByText(/Convocatoria: Apoyos 2027/)).not.toBeInTheDocument();
    expect(screen.getByText("¿Qué proyecto tienen en mente?")).toBeInTheDocument();
    expect(screen.getByText("Arreglar la cocina")).toBeInTheDocument();
    expect(screen.getByText("Porqué 1 de 5")).toBeInTheDocument();
  });

  it("sends what the person writes with Enter and keeps the line break for Shift+Enter", async () => {
    const next = view("awaiting_answer", [ai(1, "¿Y por qué?")]);
    vi.mocked(api.conversationSend).mockResolvedValue({ status: "saved", view: next, ai: "used" });
    const onView = show(view("awaiting_answer", [ai(1, "¿Qué proyecto tienen en mente?", { kind: "opening" })]));
    const box = screen.getByPlaceholderText("Escriba aquí su respuesta…");
    await userEvent.type(box, "Arreglar la cocina{Shift>}{Enter}{/Shift}y los baños");
    expect(api.conversationSend).not.toHaveBeenCalled();
    await userEvent.type(box, "{Enter}");
    await waitFor(() => expect(api.conversationSend).toHaveBeenCalledWith("proj_1", "Arreglar la cocina\ny los baños", false, undefined));
    await waitFor(() => expect(onView).toHaveBeenCalledWith(next));
  });

  it("offers the closed options as quick replies and sends the one that is chosen", async () => {
    vi.mocked(api.conversationSend).mockResolvedValue({ status: "saved", view: view("awaiting_answer", []), ai: "used" });
    show(view("awaiting_answer", [ai(1, "Elija lo que más se parezca:", { options: ["Falta dinero", "No lo sé todavía"] })]));
    await userEvent.click(screen.getByText("Falta dinero"));
    await waitFor(() => expect(api.conversationSend).toHaveBeenCalledWith("proj_1", "Falta dinero", false, undefined));
  });

  it("the root cause is only confirmed by an explicit «sí»; «no exactamente» asks what is different", async () => {
    vi.mocked(api.conversationSend).mockResolvedValue({ status: "saved", view: view("closed", []), ai: "skipped" });
    show(view("root_proposed", [ai(1, "Entonces la causa de fondo parece ser: «Falta un plan». ¿Es así?", { kind: "root_proposal", options: ["Sí, es esa", "No exactamente"] })]));
    await userEvent.click(screen.getByText("No exactamente"));
    expect(api.conversationSend).not.toHaveBeenCalled();
    expect(screen.getByPlaceholderText("¿Qué es distinto? Cuéntenos con sus palabras.")).toBeInTheDocument();
    await userEvent.click(screen.getByText("Sí, es esa"));
    await waitFor(() => expect(api.conversationSend).toHaveBeenCalledWith("proj_1", "", true, undefined));
  });

  it("when the AI failed it says so, keeps what was written and offers to try again", async () => {
    vi.mocked(api.conversationRetry).mockResolvedValue({ status: "saved", view: view("awaiting_answer", [ai(3, "¿Y por qué?")]), ai: "used" });
    show(view("awaiting_ai", [ai(1, "Hola", { kind: "opening" }), me(2, "Arreglar la cocina")]));
    expect(screen.getByText("Arreglar la cocina")).toBeInTheDocument();
    await userEvent.click(screen.getByText("Intentar otra vez"));
    await waitFor(() => expect(api.conversationRetry).toHaveBeenCalledWith("proj_1"));
  });

  it("says why the AI could not answer, in plain words", async () => {
    vi.mocked(api.conversationSend).mockResolvedValue({ status: "saved", view: view("awaiting_ai", [me(2, "Arreglar la cocina")]), ai: "offline" });
    show(view("awaiting_answer", [ai(1, "¿Qué proyecto tienen en mente?", { kind: "opening" })]));
    await userEvent.type(screen.getByPlaceholderText("Escriba aquí su respuesta…"), "Arreglar la cocina{Enter}");
    expect(await screen.findByText(/Lo que escribió quedó guardado/)).toBeInTheDocument();
  });

  it("warns, without blocking, when the idea does not fit what the call funds", () => {
    show(view("awaiting_answer", [ai(1, "¿Por qué?")], { fit: { fit: "mismatch", note: "La convocatoria apoya alimentación." } }));
    expect(screen.getByText(/no parece encajar con lo que apoya la convocatoria/)).toBeInTheDocument();
    expect(screen.getByText("La convocatoria apoya alimentación.")).toBeInTheDocument();
    expect(screen.getByPlaceholderText("Escriba aquí su respuesta…")).toBeEnabled();
  });

  it("a project from the earlier method says it needs a new one instead of an empty chat", () => {
    show(view("needs_opening", [], { legacy: true }));
    expect(screen.getByText(/se empezó con el método anterior/)).toBeInTheDocument();
    expect(api.conversationStart).not.toHaveBeenCalled();
  });

  it("when the conversation is closed the box stays, disabled, and says where to go on", () => {
    show(view("closed", [ai(1, "Quedó como usted la dijo.", { kind: "root_proposal" })], { root: { text: "Falta un plan", confirmed: true } }));
    expect(screen.queryByPlaceholderText("Escriba aquí su respuesta…")).not.toBeInTheDocument();
    expect(screen.getByPlaceholderText("La conversación terminó. Siga desde el panel de la derecha.")).toBeDisabled();
    expect(screen.getByText("Causa de fondo confirmada")).toBeInTheDocument();
    expect(screen.getByText("Terminamos la conversación")).toBeInTheDocument();
  });
});
