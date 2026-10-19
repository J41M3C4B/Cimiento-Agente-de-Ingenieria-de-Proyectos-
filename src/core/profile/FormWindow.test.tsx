import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { FieldSpec, FormView } from "../../lib/types";

const field = (id: string, kind: FieldSpec["kind"], options: string[] = []): FieldSpec => ({
  id,
  kind,
  options,
  required: false,
  essential: false,
  applies_when: { when: "always" },
  sensitivity: "public",
  ai: "as_is",
  used_by: [],
  min: null,
  max: null,
});

// the window of «Su institución» with two suggestions of the AI waiting (core/profile/forms.rs and capturist.rs)
const view: FormView = {
  spec: {
    id: "institution.identity",
    sections: [{ id: "who", columns: 1, fields: [field("institution.name", "text"), field("institution.populations", "multi_select", ["childhood", "older_adults"]), field("institution.age_min", "number")] }],
  },
  values: { "institution.name": "Asilo Ficticio" },
  origins: {},
  missing: [],
  proposals: [
    { id: "prop_1", form_id: "institution.identity", field_id: "institution.populations", value: ["older_adults"], origin: "ai_assumption", said: "atendemos a 38 señoras", document: null, page: null },
    { id: "prop_2", form_id: "institution.identity", field_id: "institution.age_min", value: 60, origin: "ai_assumption", said: "de 60 años en adelante", document: null, page: null },
  ],
  fill: { form: "institution.identity", place: "profile.institution" },
};

vi.mock("../../lib/tauri", () => ({
  formGet: vi.fn(async () => view),
  formSave: vi.fn(async () => ({ status: "saved", profile: {} })),
  proposalsReject: vi.fn(async () => 1),
  institutionFillFromDocuments: vi.fn(),
  toAppError: (e: unknown) => ({ code: "x", message: String(e) }),
}));

import { formSave, institutionFillFromDocuments, proposalsReject } from "../../lib/tauri";
import { fillAllNotice } from "./fromDocuments";
import { FormWindow } from "./FormWindow";

function draw() {
  const qc = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  render(
    <QueryClientProvider client={qc}>
      <FormWindow id="institution.identity" title="Su institución" onSaved={() => {}} onClose={() => {}} />
    </QueryClientProvider>,
  );
}

describe("FormWindow with suggestions of the AI", () => {
  beforeEach(() => vi.clearAllMocks());

  it("shows each suggestion under its field; «Usar» only fills the field and «Guardar» saves it with the ones used", async () => {
    draw();
    expect(await screen.findByText(/Hay 2 sugerencias en esta ventana/)).toBeInTheDocument();
    expect(screen.getAllByText("Sugerencia de la IA:")[0]!.closest("p")).toHaveTextContent("Sugerencia de la IA: Personas mayores (60 años o más)");
    expect(screen.getByText("Usted dijo: «atendemos a 38 señoras»")).toBeInTheDocument();

    fireEvent.click(screen.getAllByRole("button", { name: "Usar" })[0]!);
    expect(screen.getByRole("checkbox", { name: "Personas mayores (60 años o más)" })).toBeChecked();
    expect(formSave).not.toHaveBeenCalled();
    expect(screen.getByText(/Hay 1 sugerencia en esta ventana/)).toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "Guardar" }));
    await waitFor(() => expect(formSave).toHaveBeenCalled());
    const [id, values, decision, used] = vi.mocked(formSave).mock.calls[0]!;
    expect([id, values["institution.populations"], decision, used]).toEqual(["institution.identity", ["older_adults"], undefined, ["prop_1"]]);
    expect(values["institution.age_min"]).toBeUndefined();
  });

  it("«Descartar» drops a suggestion without saving anything", async () => {
    draw();
    fireEvent.click((await screen.findAllByRole("button", { name: "Descartar" }))[1]!);
    await waitFor(() => expect(proposalsReject).toHaveBeenCalledWith(["prop_2"]));
    fireEvent.click(screen.getByRole("button", { name: "Descartar todas" }));
    await waitFor(() => expect(proposalsReject).toHaveBeenLastCalledWith(["prop_1", "prop_2"]));
    expect(formSave).not.toHaveBeenCalled();
  });
});

describe("FormWindow with a suggestion read from a document (IA6)", () => {
  it("says which document and page it was read from, not that the AI or the person said it", async () => {
    view.proposals = [
      { id: "prop_3", form_id: "institution.identity", field_id: "institution.age_min", value: 60, origin: "document", said: null, document: "Reglamento interno", page: 1 },
    ];
    draw();
    expect(await screen.findByText("Lo leímos en «Reglamento interno», página 1.")).toBeInTheDocument();
    expect(screen.getByText("Lo dice un documento:")).toBeInTheDocument();
    expect(screen.queryByText("Sugerencia de la IA:")).not.toBeInTheDocument();
  });
});

describe("«Llenar con mis documentos» (C2)", () => {
  beforeEach(() => vi.clearAllMocks());

  it("asks the Reader for this window and says how many suggestions it left, or why none", async () => {
    vi.mocked(institutionFillFromDocuments)
      .mockResolvedValueOnce({ status: "proposed", form: "institution.identity", place: "profile.institution", fields: ["institution.mission", "institution.age_min"], dropped: 1, note: "" })
      .mockResolvedValueOnce({ status: "nothing", form: "institution.identity", place: "profile.institution", reason: "no_documents" })
      .mockResolvedValueOnce({ status: "unavailable", form: "institution.identity", place: "profile.institution", reason: "not_configured" });
    draw();
    const button = await screen.findByRole("button", { name: "Llenar con mis documentos" });
    fireEvent.click(button);
    await waitFor(() => expect(institutionFillFromDocuments).toHaveBeenCalledWith("institution.identity"));
    expect(await screen.findByText("Dejé 2 sugerencias de sus documentos: revíselas abajo.")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Llenar con mis documentos" }));
    expect(await screen.findByText(/Su Expediente todavía no tiene documentos con texto/)).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Llenar con mis documentos" }));
    expect(await screen.findByText(/La ayuda automática todavía no está configurada/)).toBeInTheDocument();
    // nothing is saved by it: the suggestions wait under their fields
    expect(formSave).not.toHaveBeenCalled();
  });

  it("«Llenar todo» adds up the four windows", () => {
    const proposed = (n: number) => ({ status: "proposed" as const, form: "f", place: "p", fields: Array.from({ length: n }, (_, i) => `x${i}`), dropped: 0, note: "" });
    expect(fillAllNotice([proposed(3), proposed(0), proposed(2), { status: "nothing", form: "f", place: "p", reason: "all_written" }]).text).toBe(
      "Dejé 5 sugerencias de sus documentos en 2 ventanas de «Mi institución». Ábralas para revisarlas.",
    );
    expect(fillAllNotice([{ status: "nothing", form: "f", place: "p", reason: "no_documents" }]).text).toMatch(/todavía no tiene documentos/);
    expect(fillAllNotice([proposed(1), { status: "unavailable", form: "f", place: "p", reason: "budget" }]).tone).toBe("error");
  });
});
