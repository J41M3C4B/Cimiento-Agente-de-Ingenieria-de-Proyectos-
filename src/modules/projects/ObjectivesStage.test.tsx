import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { ConversationView, NeedRow, NeedsView, ProjectRow, ReadingDetail } from "../../lib/types";

vi.mock("../../lib/tauri", () => ({
  conversationStart: vi.fn(),
  conversationSend: vi.fn(),
  conversationRetry: vi.fn(),
  callReadingGet: vi.fn(),
  projectJob: vi.fn(),
  needsGet: vi.fn(),
  needsPropose: vi.fn(),
  needAdd: vi.fn(),
  needSelect: vi.fn(),
  needRate: vi.fn(),
  toAppError: (e: unknown) => ({ code: "x", message: String(e) }),
}));

import * as api from "../../lib/tauri";
import { ObjectivesStage } from "./ObjectivesStage";

const project: ProjectRow = {
  id: "proj_1",
  institution_id: "i",
  profile_id: "p",
  title: "Apoyos 2027",
  initial_request: null,
  stage: "PRIORITIZATION",
  needs_review: false,
  created_at: "2026-10-03T10:00:00Z",
  kind: "call",
  call_reading_id: "read_1",
  color: null,
  donor_kind: null,
};

const conversation: ConversationView = {
  project,
  call: { name: "Apoyos 2027", funder: "Fundación Ficticia", year: 2027 },
  turns: [{ turn: 1, role: "assistant", kind: "root_proposal", level: null, text: "Quedó como usted la dijo.", options: [] }],
  phase: "closed",
  why_level: 0,
  max_whys: 5,
  fit: null,
  root: { text: "Falta un plan", confirmed: true },
  summary: null,
  unsupported_figures: [],
  legacy: false,
};

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

const need =(id: string, title: string, over: Partial<NeedRow> = {}): NeedRow => ({
  id,
  title,
  description: `Cómo ataca la causa: ${title}`,
  scores: null,
  total_score: null,
  selected: false,
  origin: "ai_assumption",
  confirmed: false,
  ...over,
});

const needsView = (needs: NeedRow[]): NeedsView => ({ project, needs, ranking: [], beneficiaries_suggestion: null });

const three = [need("n1", "Mantenimiento preventivo"), need("n2", "Capacitación del personal"), need("n3", "Fondo de reparaciones")];

function show(onContinue = vi.fn()) {
  const qc = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  render(
    <QueryClientProvider client={qc}>
      <ObjectivesStage view={conversation} onView={vi.fn()} onContinue={onContinue} busy={false} panelOpen />
    </QueryClientProvider>,
  );
  return onContinue;
}

beforeEach(() => {
  vi.resetAllMocks();
  vi.mocked(api.callReadingGet).mockResolvedValue(reading);
  vi.mocked(api.projectJob).mockResolvedValue({ running: null, finished: null });
});

describe("ObjectivesStage", () => {
  it("does not ask for the objectives again when the program is already writing them (the person left and came back)", async () => {
    vi.mocked(api.needsGet).mockResolvedValue(needsView([]));
    vi.mocked(api.projectJob).mockResolvedValue({ running: "needs", finished: null });
    show();
    await waitFor(() => expect(api.projectJob).toHaveBeenCalledWith("proj_1"));
    expect(await screen.findByRole("status")).toBeInTheDocument();
    expect(api.needsPropose).not.toHaveBeenCalled();
  });

  it("does not try again by itself when the AI already failed while the person was away", async () => {
    vi.mocked(api.needsGet).mockResolvedValue(needsView([]));
    vi.mocked(api.projectJob).mockResolvedValue({ running: null, finished: { kind: "needs", ai: "offline" } });
    show();
    expect(await screen.findByText(/No pudimos proponer objetivos todavía/)).toBeInTheDocument();
    expect(api.needsPropose).not.toHaveBeenCalled();
  });

  it("proposes the objectives by itself when the person arrives, in the assistant's order, with the first recommended", async () => {
    vi.mocked(api.needsGet).mockResolvedValue(needsView([]));
    vi.mocked(api.needsPropose).mockResolvedValue({ view: needsView(three), ai: "used" });
    show();
    expect(await screen.findByText("Mantenimiento preventivo")).toBeInTheDocument();
    expect(api.needsPropose).toHaveBeenCalledTimes(1);
    const titles = screen.getAllByRole("heading", { level: 3 }).map((h) => h.textContent);
    expect(titles).toEqual(["Mantenimiento preventivo", "Capacitación del personal", "Fondo de reparaciones"]);
    expect(screen.getAllByText("Recomendado")).toHaveLength(1);
    // nobody is asked to rate anything
    expect(screen.queryByText(/Calificar/)).not.toBeInTheDocument();
  });

  it("does not ask again when the objectives are already there", async () => {
    vi.mocked(api.needsGet).mockResolvedValue(needsView(three));
    show();
    expect(await screen.findByText("Capacitación del personal")).toBeInTheDocument();
    expect(api.needsPropose).not.toHaveBeenCalled();
  });

  it("going on is only offered in the panel once an objective is chosen, and choosing takes one click and no rating", async () => {
    const onContinue = vi.fn();
    vi.mocked(api.needsGet).mockResolvedValue(needsView(three));
    vi.mocked(api.needSelect).mockResolvedValue(needsView([three[0]!, { ...three[1]!, selected: true }, three[2]!]));
    show(onContinue);
    expect(await screen.findByText("Elija un objetivo en la conversación para poder continuar.")).toBeInTheDocument();
    expect(screen.queryByText("Continuar al siguiente paso")).not.toBeInTheDocument();

    await userEvent.click((await screen.findAllByText("Elegir este objetivo"))[1]!);
    await waitFor(() => expect(api.needSelect).toHaveBeenCalledWith("proj_1", "n2"));
    expect(api.needRate).not.toHaveBeenCalled();
    await userEvent.click(await screen.findByText("Continuar al siguiente paso"));
    expect(onContinue).toHaveBeenCalled();
    expect(screen.getByText(/el objetivo del proyecto será «Capacitación del personal»/)).toBeInTheDocument();
  });

  it("an objective written in the chat box is added and chosen in one go", async () => {
    const mine = need("n4", "Un huerto para la cocina", { origin: "user", description: null, confirmed: true });
    vi.mocked(api.needsGet).mockResolvedValue(needsView(three));
    vi.mocked(api.needAdd).mockResolvedValue({ status: "saved", view: needsView([...three, mine]) });
    vi.mocked(api.needSelect).mockResolvedValue(needsView([...three, { ...mine, selected: true }]));
    show();
    await screen.findByText("Capacitación del personal");
    await userEvent.type(screen.getByPlaceholderText("¿Prefiere otro objetivo? Escríbalo aquí."), "Un huerto para la cocina{Enter}");
    await waitFor(() => expect(api.needAdd).toHaveBeenCalledWith("proj_1", "Un huerto para la cocina", "", undefined));
    await waitFor(() => expect(api.needSelect).toHaveBeenCalledWith("proj_1", "n4"));
    expect(await screen.findByText("Continuar al siguiente paso")).toBeInTheDocument();
  });

  it("says so and offers to try again when the assistant could not propose", async () => {
    vi.mocked(api.needsGet).mockResolvedValue(needsView([]));
    vi.mocked(api.needsPropose).mockResolvedValue({ view: needsView([]), ai: "offline" });
    show();
    expect(await screen.findByText(/No pudimos proponer objetivos todavía/)).toBeInTheDocument();
    expect(screen.getByText("Intentar otra vez")).toBeInTheDocument();
  });
});
