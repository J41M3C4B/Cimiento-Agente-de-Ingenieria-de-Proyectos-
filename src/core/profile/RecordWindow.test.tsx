import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { Expediente, FieldSpec, RecordForm } from "../../lib/types";

const field = (id: string, kind: FieldSpec["kind"], options: string[] = []): FieldSpec => ({
  id,
  kind,
  options,
  required: false,
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
};
const expediente: Expediente = { records: [], summary: { expected: 12, have: 1, in_progress: 0, missing: 11, expiring: 0, expired: 0 } };
const report = { fields: [{ path: "page_1", counts: { email: 1 }, blocking: false, redacted_preview: "Dudas con [CORREO OCULTO]" }], counts: { email: 1 }, has_blocking: false };

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

import { open as pickFile } from "@tauri-apps/plugin-dialog";
import { institutionRecordAttach, institutionRecordSave } from "../../lib/tauri";
import { RecordWindow } from "./RecordWindow";

function draw(onDone = vi.fn()) {
  const qc = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  render(
    <QueryClientProvider client={qc}>
      <RecordWindow record={{ type: "privacy_notice", id: null }} onDone={onDone} onClose={() => {}} />
    </QueryClientProvider>,
  );
  return onDone;
}

describe("RecordWindow", () => {
  beforeEach(() => vi.clearAllMocks());

  it("says what the document is, and a file chosen in the system goes through the scanner before it is kept", async () => {
    vi.mocked(institutionRecordAttach)
      .mockResolvedValueOnce({ status: "quarantine", report })
      // a scan, read as pictures (OCR): its worn page was read with doubts
      .mockResolvedValueOnce({ status: "attached", id: "rec_1", readable: true, from_picture: true, doubtful: [2], expediente });
    draw();
    expect(await screen.findByText("El texto que explica a las personas qué datos suyos se piden.")).toBeInTheDocument();
    expect(screen.getByText("Sin archivo. Puede marcarlo aunque no lo suba.")).toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "Subir archivo" }));
    await waitFor(() => expect(institutionRecordAttach).toHaveBeenCalledWith("privacy_notice", null, "C:\\Documentos\\aviso.docx", undefined));
    fireEvent.click(await screen.findByRole("button", { name: "Tapar y seguir" }));
    // the same file again, with the decision: the window of the system is not opened twice
    await waitFor(() => expect(institutionRecordAttach).toHaveBeenLastCalledWith("privacy_notice", null, "C:\\Documentos\\aviso.docx", "redact"));
    expect(pickFile).toHaveBeenCalledTimes(1);
    expect(await screen.findByText(/pero la página 2 se leyó con dudas/)).toBeInTheDocument();
    // with its file it is had: the state goes with the save
    expect(await screen.findByRole("radio", { name: "Lo tenemos" })).toBeChecked();
  });

  it("a file it cannot take says why, and saving sends what the person marked", async () => {
    vi.mocked(institutionRecordAttach).mockResolvedValueOnce({ status: "unreadable", reason: "roster" });
    const onDone = draw();
    fireEvent.click(await screen.findByRole("button", { name: "Subir archivo" }));
    expect(await screen.findByText(/es una lista de personas/)).toBeInTheDocument();

    fireEvent.click(screen.getByRole("radio", { name: "En trámite" }));
    fireEvent.click(screen.getByRole("button", { name: "Guardar" }));
    await waitFor(() => expect(onDone).toHaveBeenCalledWith("Listo, quedó guardado."));
    expect(vi.mocked(institutionRecordSave).mock.calls[0]).toEqual(["privacy_notice", null, { "institution.record.status": "in_progress" }, undefined]);
  });
});
