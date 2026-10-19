import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { ConversationPhase, ConversationView, ProjectRow, ReadingDetail, StoredSummary, TurnView } from "../../lib/types";

vi.mock("../../lib/tauri", () => ({
  conversationStart: vi.fn(),
  conversationSend: vi.fn(),
  conversationRetry: vi.fn(),
  callReadingGet: vi.fn(),
  projectJob: vi.fn(),
  diagnosisSummaryGenerate: vi.fn(),
  diagnosisSummaryEdit: vi.fn(),
  diagnosisSummaryConfirm: vi.fn(),
  // an error the program sent keeps its code; anything else is a plain failure
  toAppError: (e: unknown) => (e && typeof e === "object" && "code" in e ? e : { code: "x", message: String(e) }),
}));

import * as api from "../../lib/tauri";
import { DiagnosisPanel } from "./DiagnosisPanel";

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

const turns: TurnView[] = [{ turn: 1, role: "assistant", kind: "root_proposal", level: null, text: "Quedó como usted la dijo.", options: [] }];

const stored = (confirmed: boolean): StoredSummary => ({
  origin: "ai_assumption",
  confirmed_at: confirmed ? "2026-10-04T10:00:00Z" : null,
  summary: {
    problem_statement: "No hay camioneta.",
    affected: { group: "Residentes", count: 20, description: "Personas mayores" },
    current_consequences: [],
    root_causes: [],
    reframed_need: "Transporte confiable",
    alternatives: [],
    suggested_indicators: [],
    open_questions: [],
  },
});

const view = (phase: ConversationPhase, summary: StoredSummary | null = null): ConversationView => ({
  project,
  call: { name: "Apoyos 2027", funder: "Fundación Ficticia", year: 2027 },
  turns,
  phase,
  why_level: 0,
  max_whys: 5,
  fit: null,
  root: null,
  summary,
  unsupported_figures: [],
  legacy: false,
});

const reading: ReadingDetail = {
  reading: {
    id: "read_1",
    name: "Convocatoria Ejemplo",
    funder: "Fundación Ficticia",
    year: 2027,
    status: "ready",
    note: null,
    created_at: "2026-10-03T10:00:00Z",
    finished_at: "2026-10-03T10:02:00Z",
    confirmed_at: "2026-10-03T10:03:00Z",
    files: [],
  },
  card: null,
  brief_pending: false,
  summary: null,
  quality: null,
  differences: [],
};

function show(v: ConversationView, props: { onContinue?: () => void; onView?: (v: ConversationView) => void } = {}) {
  const qc = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  render(
    <QueryClientProvider client={qc}>
      <DiagnosisPanel view={v} onView={props.onView ?? vi.fn()} onContinue={props.onContinue ?? vi.fn()} busy={false} panelOpen />
    </QueryClientProvider>,
  );
}

beforeEach(() => {
  vi.resetAllMocks();
  vi.mocked(api.callReadingGet).mockResolvedValue(reading);
  vi.mocked(api.projectJob).mockResolvedValue({ running: null, finished: null });
});

describe("DiagnosisPanel", () => {
  it("while the summary is being written the chat shows the assistant working, not just the button", async () => {
    vi.mocked(api.diagnosisSummaryGenerate).mockReturnValue(new Promise(() => undefined)); // never answers: it is working
    show(view("closed"));
    await userEvent.click(await screen.findByText("Armar el resumen"));
    expect(await screen.findByText("Repasando todo lo que me contó…")).toBeInTheDocument();
    expect(screen.getByText("Estamos armando el resumen…")).toBeInTheDocument();
  });

  it("when the person left and came back while the summary was being written, the work is still shown and nothing can be asked twice", async () => {
    vi.mocked(api.projectJob).mockResolvedValue({ running: "summary", finished: null });
    show(view("closed"));
    expect(await screen.findByText("Repasando todo lo que me contó…")).toBeInTheDocument();
    expect(screen.getByText("Estamos armando el resumen…").closest("button")).toBeDisabled();
    expect(api.diagnosisSummaryGenerate).not.toHaveBeenCalled();
  });

  it("a request the program refuses because it is already doing it is not shown as an error", async () => {
    vi.mocked(api.diagnosisSummaryGenerate).mockRejectedValue({ code: "already_running", message: "Ya lo estamos haciendo." });
    show(view("closed"));
    await userEvent.click(await screen.findByText("Armar el resumen"));
    await waitFor(() => expect(api.diagnosisSummaryGenerate).toHaveBeenCalled());
    expect(screen.queryByText("Ya lo estamos haciendo.")).not.toBeInTheDocument();
  });

  it("when the summary is ready the chat says so and offers to open it", async () => {
    show(view("closed", stored(false)));
    expect(await screen.findByText(/Ya quedó el resumen/)).toBeInTheDocument();
    expect(screen.getAllByText("Ver el resumen").length).toBeGreaterThan(0);
  });

  it("while they talk, the panel only says where the next step will be and the chat has no buttons to go on", async () => {
    show(view("awaiting_answer"));
    expect(await screen.findByText("Cuando terminemos la conversación, el siguiente paso aparecerá aquí.")).toBeInTheDocument();
    expect(screen.queryByText("Armar el resumen")).not.toBeInTheDocument();
    expect(screen.queryByText("Continuar al siguiente paso")).not.toBeInTheDocument();
  });

  it("when the conversation closes, the panel offers to make the summary", async () => {
    show(view("closed"));
    await userEvent.click(await screen.findByText("Armar el resumen"));
    await waitFor(() => expect(api.diagnosisSummaryGenerate).toHaveBeenCalledWith("proj_1"));
  });

  it("a summary that is not confirmed cannot be continued from; once confirmed the panel offers to continue", async () => {
    const onContinue = vi.fn();
    show(view("closed", stored(false)), { onContinue });
    expect(await screen.findByText("Falta que lo revise")).toBeInTheDocument();
    expect(screen.queryByText("Continuar al siguiente paso")).not.toBeInTheDocument();
  });

  it("a confirmed summary can be viewed and the project can go on", async () => {
    const onContinue = vi.fn();
    show(view("closed", stored(true)), { onContinue });
    expect(await screen.findByText("Confirmado")).toBeInTheDocument();
    await userEvent.click(screen.getByText("Continuar al siguiente paso"));
    expect(onContinue).toHaveBeenCalled();
  });
});
