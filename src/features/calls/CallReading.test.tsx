import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { CallSummary, ReadingDetail, ReadingRow } from "../../lib/types";

vi.mock("../../lib/tauri", () => ({
  callReadingGet: vi.fn(),
  callReadingRetry: vi.fn(),
  callReadingConfirm: vi.fn(),
  toAppError: (e: unknown) => ({ code: "x", message: String(e) }),
}));

import * as api from "../../lib/tauri";
import { CallPanel, isBusy } from "./CallReading";

const row = (over: Partial<ReadingRow> = {}): ReadingRow => ({
  id: "read_1",
  name: "Convocatoria Ejemplo",
  funder: "Fundación Ficticia",
  year: 2027,
  status: "ready",
  note: null,
  created_at: "2026-10-03T10:00:00Z",
  finished_at: "2026-10-03T10:02:00Z",
  confirmed_at: null,
  files: [
    { document_id: "doc_1", name: "bases.pdf", pages: 12, role: "main" },
    { document_id: "doc_2", name: "guia.pdf", pages: 4, role: "guide" },
  ],
  ...over,
});

const summary: CallSummary = {
  document_kind: { words: "Convocatoria", class: "convocatoria" },
  title: "Convocatoria Ejemplo 2027",
  funder: "Fundación Ficticia",
  edition: null,
  objective: null,
  dates: [{ label: "Cierre de postulación", when: "23 de mayo de 2027", kind: "cierre", page: 3, file: "bases.pdf" }],
  amounts: [{ kind: "max_amount", label: null, value: "$250,000", page: 2, file: "bases.pdf" }],
  groups: [{ key: "who_can", items: [{ text: "Asociaciones civiles", applies_to: "con dos años de operación", requirement: "obligatorio", page: 2, file: "bases.pdf" }] }],
  conflicts: [{ field: "contrapartida", note: "Dicen 30 % y 20 %.", versions: [{ text: "30%", applies_to: null, requirement: null, page: 4, file: "bases.pdf" }] }],
  doubts: ["No se dice cuántos proyectos se apoyarán."],
  missing: ["entrega.medio_o_lugar"],
};

const detail = (r: ReadingRow, over: Partial<ReadingDetail> = {}): ReadingDetail => ({
  reading: r,
  card: null,
  brief_pending: false,
  differences: [],
  summary,
  quality: { pages: 12, pages_with_quotes: 9, quotes_verified_percent: 97, blocks_read: 8, blocks_total: 8 },
  ...over,
});

function show(readingId: string | null = "read_1") {
  const qc = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  render(
    <QueryClientProvider client={qc}>
      <CallPanel readingId={readingId} />
    </QueryClientProvider>,
  );
}

beforeEach(() => {
  vi.resetAllMocks();
});

describe("CallPanel", () => {
  it("is one line with the name and who calls, and the whole understanding opens in a window", async () => {
    vi.mocked(api.callReadingGet).mockResolvedValue(detail(row({ confirmed_at: "2026-10-03T11:00:00Z" })));
    show();
    expect(await screen.findByText("Convocatoria Ejemplo")).toBeInTheDocument();
    expect(screen.getByText("Confirmada")).toBeInTheDocument();
    await userEvent.click(screen.getByText("Ver resumen"));
    expect(await screen.findByText("Convocatoria Ejemplo 2027")).toBeInTheDocument();
    expect(screen.getByText("Asociaciones civiles")).toBeInTheDocument();
  });

  it("says the files are gone when the project has lost its call", () => {
    show(null);
    expect(screen.getByText(/ya no hay nada que leer/)).toBeInTheDocument();
  });
});

describe("isBusy", () => {
  it("a call being read shows no buttons to read it again and the screen keeps asking", () => {
    expect(isBusy(row({ status: "reading" }))).toBe(true);
    expect(isBusy(row({ status: "waiting", note: null }))).toBe(true);
    expect(isBusy(row({ status: "waiting", note: "offline" }))).toBe(false);
    expect(isBusy(row({ status: "ready" }))).toBe(false);
  });
});
