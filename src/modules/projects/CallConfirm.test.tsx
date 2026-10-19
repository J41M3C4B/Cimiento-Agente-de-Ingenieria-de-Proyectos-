import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { CallCard, CallSummary, ReadingDetail, ReadingRow } from "../../lib/types";

vi.mock("../../lib/tauri", () => ({
  callReadingGet: vi.fn(),
  callReadingRetry: vi.fn(),
  callReadingConfirm: vi.fn(),
  callBriefMake: vi.fn(),
  projectJob: vi.fn(),
  toAppError: (e: unknown) => ({ code: "x", message: String(e) }),
}));

import * as api from "../../lib/tauri";
import { CallConfirm } from "./CallConfirm";

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

// what the person reads first: short, in plain words (ADR-024)
const card: CallCard = {
  title: "Convocatoria Ejemplo 2027",
  funder: "Fundación Ficticia",
  edition: null,
  brief: "Es una convocatoria de la Fundación Ficticia. Da hasta $250,000 por proyecto.",
  lead: null,
  facts: [
    { kind: "max_amount", value: "$250,000", page: 2, file: "bases.pdf" },
    { kind: "closing", value: "23 de mayo de 2027", page: 3, file: "bases.pdf" },
  ],
  blocks: [{ key: "who_can", points: [{ text: "Asociaciones civiles", applies_to: "con dos años de operación", page: 2, file: "bases.pdf" }], more: 3 }],
  alerts: [
    { kind: "conflicts", count: 1, fields: [] },
    { kind: "missing", count: 1, fields: ["entrega.medio_o_lugar"] },
    { kind: "doubts", count: 1, fields: [] },
  ],
};

const detail = (r: ReadingRow, over: Partial<ReadingDetail> = {}): ReadingDetail => ({
  reading: r,
  card,
  brief_pending: false,
  differences: [],
  summary,
  quality: { pages: 12, pages_with_quotes: 9, quotes_verified_percent: 97, blocks_read: 8, blocks_total: 8 },
  ...over,
});

function show(readingId: string | null = "read_1", onContinue = vi.fn()) {
  const qc = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  render(
    <QueryClientProvider client={qc}>
      <CallConfirm readingId={readingId} onContinue={onContinue} />
    </QueryClientProvider>,
  );
  return onContinue;
}

beforeEach(() => {
  vi.resetAllMocks();
  vi.mocked(api.projectJob).mockResolvedValue({ running: null, finished: null });
});

describe("CallConfirm", () => {
  it("shows what the person said about the call and what each file is, in one calm column", async () => {
    vi.mocked(api.callReadingGet).mockResolvedValue(detail(row()));
    show();
    expect(await screen.findByText("Esto leímos de su convocatoria")).toBeInTheDocument();
    expect(screen.getByText("Convocatoria Ejemplo")).toBeInTheDocument();
    expect(screen.getByText("Fundación Ficticia · 2027")).toBeInTheDocument();
    expect(screen.getByText("bases.pdf")).toBeInTheDocument();
    expect(screen.getByText("12 páginas")).toBeInTheDocument();
    expect(screen.getByText("La convocatoria")).toBeInTheDocument();
    expect(screen.getByText("guia.pdf")).toBeInTheDocument();
    expect(screen.getByText("Guía o instructivo")).toBeInTheDocument();
  });

  it("says the files are gone when the project has lost its call", () => {
    show(null);
    expect(screen.getByText(/ya no hay nada que leer/)).toBeInTheDocument();
    expect(api.callReadingGet).not.toHaveBeenCalled();
  });

  it("shows the card first: what it is in a few words, the figures that matter, whether it fits and only the warnings that are true", async () => {
    vi.mocked(api.callReadingGet).mockResolvedValue(detail(row()));
    show();
    expect(await screen.findByText("En pocas palabras")).toBeInTheDocument();
    expect(screen.getByText(/Es una convocatoria de la Fundación Ficticia/)).toBeInTheDocument();
    expect(screen.getByText("Resumen hecho con ayuda automática. El documento es el que vale.")).toBeInTheDocument();
    // the figures, in the document's own words, with their page
    expect(screen.getByText("Monto máximo por proyecto")).toBeInTheDocument();
    expect(screen.getAllByText("$250,000").length).toBeGreaterThan(0);
    expect(screen.getByText("23 de mayo de 2027")).toBeInTheDocument();
    expect(screen.getAllByText("bases.pdf, página 2").length).toBeGreaterThan(0);
    // a few points, and how many more the detail has
    expect(screen.getByText("Quién puede participar")).toBeInTheDocument();
    expect(screen.getByText("Asociaciones civiles")).toBeInTheDocument();
    expect(screen.getByText("Aplica a: con dos años de operación")).toBeInTheDocument();
    expect(screen.getByText("y 3 más en el detalle")).toBeInTheDocument();
    // the warnings, in plain words
    expect(screen.getByText(/En un punto, los documentos dicen cosas distintas/)).toBeInTheDocument();
    expect(screen.getByText("Dónde se entrega la solicitud")).toBeInTheDocument();
    expect(screen.getByText(/Quedó una duda al leerla/)).toBeInTheDocument();
    // what the AI reads is not the main view: no list of topics, no reading telemetry
    expect(screen.queryByText("Documentos que piden siempre")).not.toBeInTheDocument();
    expect(screen.queryByText(/Se leyeron 8 de 8 partes/)).not.toBeInTheDocument();
    // the summary was already written: nobody is asked to write it again
    expect(api.callBriefMake).not.toHaveBeenCalled();
  });

  it("writes the «en pocas palabras» once when nobody has tried yet, and meanwhile the card says what the document says it is for", async () => {
    vi.mocked(api.callReadingGet).mockResolvedValue(detail(row(), { brief_pending: true, card: { ...card, brief: null, lead: "Apoyar a casas hogar y asilos en su alimentación." } }));
    vi.mocked(api.callBriefMake).mockResolvedValue(detail(row(), { card: { ...card, brief: "Resumen recién escrito." } }));
    show();
    expect(await screen.findByText("Resumen recién escrito.")).toBeInTheDocument();
    expect(api.callBriefMake).toHaveBeenCalledTimes(1);
    expect(api.callBriefMake).toHaveBeenCalledWith("read_1");
  });

  it("does not pay for the same summary twice when the program says it is already being written", async () => {
    vi.mocked(api.callReadingGet).mockResolvedValue(detail(row(), { brief_pending: true, card: { ...card, brief: null, lead: "Apoyar a casas hogar." } }));
    vi.mocked(api.projectJob).mockResolvedValue({ running: "brief", finished: null });
    show();
    expect(await screen.findByText("Apoyar a casas hogar.")).toBeInTheDocument();
    await waitFor(() => expect(api.projectJob).toHaveBeenCalledWith("call:read_1"));
    expect(api.callBriefMake).not.toHaveBeenCalled();
  });

  it("the whole structured reading is behind «Consultar el detalle», with how it was read", async () => {
    vi.mocked(api.callReadingGet).mockResolvedValue(detail(row()));
    show();
    await userEvent.click(await screen.findByText("Consultar el detalle"));
    expect(await screen.findByRole("dialog")).toBeInTheDocument();
    expect(screen.getByText("Detalle de la convocatoria")).toBeInTheDocument();
    expect(screen.getByText(/Tipo de documento:/)).toBeInTheDocument();
    expect(screen.getByText("Cómo se leyó")).toBeInTheDocument();
    expect(screen.getByText(/Se leyeron 8 de 8 partes/)).toBeInTheDocument();
  });

  it("warns, without blocking, when what the person wrote does not match what was read", async () => {
    vi.mocked(api.callReadingGet).mockResolvedValue(
      detail(row(), { differences: [{ field: "funder", said: "Fundación Alsea", read: "Nacional Monte de Piedad" }, { field: "year", said: "2025", read: "2026" }] }),
    );
    show();
    expect(await screen.findByText("Revise que sea la convocatoria correcta")).toBeInTheDocument();
    expect(screen.getByText(/Usted escribió que la convoca «Fundación Alsea», pero el documento menciona «Nacional Monte de Piedad»/)).toBeInTheDocument();
    expect(screen.getByText(/Usted escribió el año 2025, pero el documento menciona 2026/)).toBeInTheDocument();
    expect(screen.getByText("Sí, es esta convocatoria")).toBeEnabled();
  });

  it("warns, without blocking, when what was read does not look like a call", async () => {
    vi.mocked(api.callReadingGet).mockResolvedValue(detail(row(), { summary: { ...summary, document_kind: { words: "Aviso", class: "aviso" } } }));
    show();
    expect(await screen.findByText(/no parece ser una convocatoria/)).toBeInTheDocument();
    expect(screen.getByText("Quién puede participar")).toBeInTheDocument();
  });

  it("explains why a call is waiting and offers to read it again", async () => {
    vi.mocked(api.callReadingGet).mockResolvedValue(detail(row({ status: "waiting", note: "not_configured" }), { summary: null, quality: null, card: null }));
    vi.mocked(api.callReadingRetry).mockResolvedValue(true);
    show();
    expect(await screen.findByText(/falta activar la ayuda automática/)).toBeInTheDocument();
    expect(screen.queryByText("En pocas palabras")).not.toBeInTheDocument();
    await userEvent.click(screen.getByText("Leer otra vez"));
    await waitFor(() => expect(api.callReadingRetry).toHaveBeenCalledWith("read_1"));
  });

  it("the person confirms the call, and only then can the project go on", async () => {
    const onContinue = vi.fn();
    vi.mocked(api.callReadingGet).mockResolvedValue(detail(row()));
    vi.mocked(api.callReadingConfirm).mockResolvedValue(true);
    show("read_1", onContinue);
    // the topics are in view and there is no way to continue before confirming
    expect(await screen.findByText("Quién puede participar")).toBeInTheDocument();
    expect(screen.queryByText("Continuar a la conversación")).not.toBeInTheDocument();
    vi.mocked(api.callReadingGet).mockResolvedValue(detail(row({ confirmed_at: "2026-10-03T11:00:00Z" })));
    await userEvent.click(screen.getByText("Sí, es esta convocatoria"));
    await waitFor(() => expect(api.callReadingConfirm).toHaveBeenCalledWith("read_1"));
    await userEvent.click(await screen.findByText("Continuar a la conversación"));
    expect(onContinue).toHaveBeenCalled();
    expect(screen.getByText("Convocatoria confirmada.")).toBeInTheDocument();
  });

  it("a call that is still being read cannot be confirmed yet", async () => {
    vi.mocked(api.callReadingGet).mockResolvedValue(detail(row({ status: "reading" }), { summary: null, quality: null, card: null }));
    show();
    expect(await screen.findByText("Cuando termine de leerse podrá confirmarla.")).toBeInTheDocument();
    expect(screen.queryByText("Sí, es esta convocatoria")).not.toBeInTheDocument();
  });
});
