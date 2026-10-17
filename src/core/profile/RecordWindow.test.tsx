import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { Expediente, FieldSpec, RecordForm } from "../../lib/types";

const field = (id: string, kind: FieldSpec["kind"], options: string[] = []): FieldSpec => ({
  id,
  kind,
  options,
  required: false,
  essential: false,
  applies_when: { when: "always" },
  sensitivity: "internal",
  ai: "as_is",
  used_by: [],
  min: null,
  max: null,
});

// the window of a document of the catalog, as Rust describes it (core/institution/forms.rs RECORD_KNOWN)
const empty: RecordForm = {
  spec: { id: "institution.record", sections: [{ id: "record_state", columns: 1, fields: [field("institution.record.status", "select", ["have", "in_progress", "missing", "not_applicable"])] }] },
  values: {},
  missing: ["institution.record.status"],
  file: null,
  read: {},
};
// the same window once the file filled its date of issue (ADR-038)
const withFile: RecordForm = {
  spec: {
    id: "institution.record",
    sections: [
      { id: "record_state", columns: 1, fields: [field("institution.record.status", "select", ["have", "in_progress", "missing", "not_applicable"])] },
      { id: "record_dates", columns: 2, fields: [field("institution.record.issued_on", "date")] },
    ],
  },
  values: { "institution.record.status": "have", "institution.record.issued_on": "2026-02-06" },
  missing: [],
  file: { name: "aviso.docx", readable: true, from_picture: true, doubtful: [2] },
  read: { "institution.record.issued_on": { page: 1, computed: false } },
};
const expediente: Expediente = { records: [], summary: { expected: 12, have: 1, in_progress: 0, missing: 11, expiring: 0, expired: 0 } };

vi.mock("@tauri-apps/plugin-dialog", () => ({ open: vi.fn(async () => "C:\\Documentos\\aviso.docx") }));
vi.mock("../../lib/tauri", () => ({
  institutionRecordForm: vi.fn(async () => empty),
  institutionRecordSave: vi.fn(async () => ({ status: "saved", id: "rec_1", expediente })),
  institutionRecordAttach: vi.fn(),
  institutionRecordOpen: vi.fn(async () => false),
  institutionRecordDetach: vi.fn(),
  institutionRecords: vi.fn(async () => expediente),
  manualEntry: vi.fn(async (id: string) => ({ id, title: "Aviso de privacidad", paragraphs: ["El texto que explica a las personas qué datos suyos se piden."], used_by: [] })),
  toAppError: (e: unknown) => ({ code: "x", message: String(e) }),
}));

import { act } from "@testing-library/react";
import { open as pickFile } from "@tauri-apps/plugin-dialog";
import { institutionRecordAttach, institutionRecordForm, institutionRecordSave } from "../../lib/tauri";
import type { AttachOutcome } from "../../lib/types";
import { endedText, markEnded, markReading, resetRecordReadings, useRecordReadings } from "./recordReading";
import { RecordWindow, attachedNotice } from "./RecordWindow";

function draw(onDone = vi.fn()) {
  const qc = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  render(
    <QueryClientProvider client={qc}>
      <RecordWindow record={{ type: "privacy_notice", id: null }} onDone={onDone} onClose={() => {}} />
    </QueryClientProvider>,
  );
  return onDone;
}

const attached = (more: Partial<Extract<AttachOutcome, { status: "attached" }>> = {}): AttachOutcome => ({
  status: "attached",
  id: "rec_1",
  readable: true,
  from_picture: false,
  doubtful: [],
  covered: 0,
  filled: [],
  suggested: 0,
  looks_like: null,
  expediente,
  ...more,
});

/** The file is read in the background (ADR-038): the window asks for it, and Rust says later how it ended. */
async function upload(outcome: AttachOutcome) {
  vi.mocked(institutionRecordAttach).mockResolvedValueOnce({ status: "reading", key: "privacy_notice" });
  fireEvent.click(await screen.findByRole("button", { name: "Subir archivo" }));
  await waitFor(() => expect(institutionRecordAttach).toHaveBeenCalledWith("privacy_notice", null, "C:\\Documentos\\aviso.docx"));
  // while it is read: the window says so, and may be closed
  expect(await screen.findByText(/Leyendo el archivo… Puede cerrar esta ventana/)).toBeInTheDocument();
  act(() => {
    markEnded({ key: "privacy_notice", kind: "privacy_notice", outcome });
  });
}

describe("RecordWindow", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    resetRecordReadings();
    vi.mocked(institutionRecordForm).mockImplementation(async () => empty);
  });

  it("says what the document is, and a file chosen in the system is kept with the data of people covered, without asking", async () => {
    // once the file is on it, the window says what the file filled
    vi.mocked(institutionRecordForm).mockImplementation(async (_kind, id) => (id ? withFile : empty));
    draw();
    expect(await screen.findByText("El texto que explica a las personas qué datos suyos se piden.")).toBeInTheDocument();
    expect(screen.getByText("Sin archivo. Puede marcarlo aunque no lo suba.")).toBeInTheDocument();

    // a scan, read as pictures (OCR): its worn page was read with doubts, and the scanner covered an email
    await upload(attached({ from_picture: true, doubtful: [2], covered: 1, filled: ["institution.record.issued_on"], suggested: 2 }));
    expect(
      await screen.findByText(/pero la página 2 se leyó con dudas.*Llenamos «Fecha de emisión».*Dejamos 2 sugerencias en «Mi institución».*Tapamos 1 dato personal/),
    ).toBeInTheDocument();
    // the date the file filled shows in its field, saying where it came from
    expect(await screen.findByText("Leído del archivo, página 1.")).toBeInTheDocument();
    expect(screen.getByDisplayValue("2026-02-06")).toBeInTheDocument();
    // nothing is asked: one window of the system, one reading
    expect(screen.queryByRole("button", { name: "Tapar y seguir" })).not.toBeInTheDocument();
    expect(pickFile).toHaveBeenCalledTimes(1);
    expect(institutionRecordAttach).toHaveBeenCalledTimes(1);
    // with its file it is had: the state goes with the save
    expect(await screen.findByRole("radio", { name: "Lo tenemos" })).toBeChecked();
  });

  it("a file that looks like another document fills nothing and says so", async () => {
    draw();
    await upload(attached({ looks_like: "tax_status_certificate" }));
    expect(await screen.findByText(/Este archivo parece «Constancia de situación fiscal», no este documento/)).toBeInTheDocument();
  });

  it("a file it cannot take says why, and saving sends what the person marked", async () => {
    const onDone = draw();
    await upload({ status: "unreadable", reason: "too_many_pages" });
    expect(await screen.findByText(/más de 100 páginas escaneadas/)).toBeInTheDocument();

    fireEvent.click(screen.getByRole("radio", { name: "En trámite" }));
    fireEvent.click(screen.getByRole("button", { name: "Guardar" }));
    await waitFor(() => expect(onDone).toHaveBeenCalledWith("Listo, quedó guardado."));
    expect(vi.mocked(institutionRecordSave).mock.calls[0]).toEqual(["privacy_notice", null, { "institution.record.status": "in_progress" }, undefined]);
  });

  it("a document already being read is not read twice", async () => {
    vi.mocked(institutionRecordAttach).mockResolvedValueOnce({ status: "already_reading", key: "privacy_notice" });
    draw();
    fireEvent.click(await screen.findByRole("button", { name: "Subir archivo" }));
    expect(await screen.findByText("Ya estamos leyendo un archivo de este documento. Espere a que termine.")).toBeInTheDocument();
  });
});

describe("the readings of the Expediente in the background", () => {
  beforeEach(() => resetRecordReadings());

  it("keep which documents are read and how each one ended, said with the name of its document", () => {
    let seen: ReturnType<typeof useRecordReadings> | null = null;
    function Probe() {
      seen = useRecordReadings();
      return null;
    }
    render(<Probe />);
    act(() => markReading("articles_of_incorporation", "articles_of_incorporation"));
    expect(seen!.reading.has("articles_of_incorporation")).toBe(true);
    act(() => {
      markEnded({ key: "articles_of_incorporation", kind: "articles_of_incorporation", outcome: attached({ filled: ["institution.record.reference"] }) });
    });
    expect(seen!.reading.size).toBe(0);
    const last = seen!.ended[seen!.ended.length - 1]!;
    expect(endedText(last, attachedNotice)).toMatch(/^«Acta constitutiva»: Se guardó el archivo\. Llenamos «Folio o número»/);
    // an error ends it too, and says so: nothing ends in silence
    const failed = markEnded({ key: "power_of_attorney", kind: "power_of_attorney", error: { code: "internal", message: "Algo salió mal." } });
    expect(endedText(failed, attachedNotice)).toBe("«Poder del representante legal»: Algo salió mal.");
  });
});
