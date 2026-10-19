import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { CallCard, CallSummary, ReadingDetail, ReadingRow } from "../../lib/types";

vi.mock("../../lib/tauri", () => ({
  callReadingGet: vi.fn(),
  callBriefMake: vi.fn(),
  projectJob: vi.fn(),
  toAppError: (e: unknown) => ({ code: "x", message: String(e) }),
}));

import * as api from "../../lib/tauri";
import { CallIndex } from "./CallIndex";

const row: ReadingRow = {
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
};

const summary: CallSummary = {
  document_kind: { words: "Convocatoria", class: "convocatoria" },
  title: "Convocatoria Ejemplo 2027",
  funder: "Fundación Ficticia",
  edition: "2027",
  objective: "Fortalecer a organizaciones que atienden a la primera infancia.",
  dates: [{ label: "Cierre de postulación", when: "23 de mayo de 2027", kind: "cierre", page: 3, file: "bases.pdf" }],
  amounts: [{ kind: "max_amount", label: null, value: "$250,000", page: 2, file: "bases.pdf" }],
  groups: [{ key: "who_can", items: [{ text: "Asociaciones civiles", applies_to: null, requirement: "obligatorio", page: 2, file: "bases.pdf" }] }],
  conflicts: [],
  doubts: [],
  missing: [],
};

const card: CallCard = {
  title: "Convocatoria Ejemplo 2027",
  funder: "Fundación Ficticia",
  edition: "2027",
  brief: "Apoya a organizaciones de primera infancia. Da hasta $250,000.",
  lead: null,
  facts: [
    { kind: "max_amount", value: "$250,000", page: 2, file: "bases.pdf" },
    { kind: "closing", value: "23 de mayo de 2027", page: 3, file: "bases.pdf" },
  ],
  blocks: [
    {
      key: "who_can",
      points: [
        { text: "Asociaciones civiles", applies_to: null, page: 2, file: "bases.pdf" },
        { text: "Con dos años de operación", applies_to: null, page: 2, file: "bases.pdf" },
        { text: "Con RFC vigente", applies_to: null, page: 2, file: "bases.pdf" },
      ],
      more: 0,
    },
  ],
  alerts: [],
};

const detail = (over: Partial<ReadingDetail> = {}): ReadingDetail => ({ reading: row, card, brief_pending: false, differences: [], summary, quality: null, ...over });

function show() {
  const qc = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  render(
    <QueryClientProvider client={qc}>
      <CallIndex readingId="read_1" />
    </QueryClientProvider>,
  );
}

beforeEach(() => {
  vi.resetAllMocks();
  vi.mocked(api.projectJob).mockResolvedValue({ running: null, finished: null });
});

describe("CallIndex", () => {
  it("shows the card, compact: what it is, the figures that matter and a couple of points, with no list of topics", async () => {
    vi.mocked(api.callReadingGet).mockResolvedValue(detail());
    show();
    expect(await screen.findByText("En pocas palabras")).toBeInTheDocument();
    expect(screen.getByText("Monto máximo por proyecto")).toBeInTheDocument();
    expect(screen.getByText("23 de mayo de 2027")).toBeInTheDocument();
    expect(screen.getByText("Quién puede participar")).toBeInTheDocument();
    // two points in the panel, and the third is counted, not shown
    expect(screen.getByText("Asociaciones civiles")).toBeInTheDocument();
    expect(screen.getByText("Con dos años de operación")).toBeInTheDocument();
    expect(screen.queryByText("Con RFC vigente")).not.toBeInTheDocument();
    expect(screen.getByText("y 1 más en el detalle")).toBeInTheDocument();
    expect(screen.queryByText("Dinero y topes")).not.toBeInTheDocument();
  });

  it("opens the whole structured reading, to consult, in a window", async () => {
    vi.mocked(api.callReadingGet).mockResolvedValue(detail());
    show();
    await userEvent.click(await screen.findByText("Consultar el detalle"));
    expect(await screen.findByRole("dialog")).toBeInTheDocument();
    expect(screen.getByText("Detalle de la convocatoria")).toBeInTheDocument();
  });

  it("says so while the call is still being read", async () => {
    vi.mocked(api.callReadingGet).mockResolvedValue(detail({ reading: { ...row, status: "reading" }, summary: null, card: null }));
    show();
    expect(await screen.findByText("La convocatoria todavía se está leyendo.")).toBeInTheDocument();
    expect(screen.queryByText("Consultar el detalle")).not.toBeInTheDocument();
  });
});
