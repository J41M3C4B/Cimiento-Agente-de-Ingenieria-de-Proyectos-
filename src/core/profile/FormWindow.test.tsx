import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { FieldSpec, FormView } from "../../lib/types";

const field = (id: string, kind: FieldSpec["kind"], options: string[] = []): FieldSpec => ({
  id,
  kind,
  options,
  required: false,
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
    { id: "prop_1", form_id: "institution.identity", field_id: "institution.populations", value: ["older_adults"], origin: "ai_assumption", said: "atendemos a 38 señoras" },
    { id: "prop_2", form_id: "institution.identity", field_id: "institution.age_min", value: 60, origin: "ai_assumption", said: "de 60 años en adelante" },
  ],
  fill: { form: "institution.identity", place: "profile.institution" },
};

vi.mock("../../lib/tauri", () => ({
  formGet: vi.fn(async () => view),
  formSave: vi.fn(async () => ({ status: "saved", profile: {} })),
  proposalsReject: vi.fn(async () => 1),
  toAppError: (e: unknown) => ({ code: "x", message: String(e) }),
}));

import { formSave, proposalsReject } from "../../lib/tauri";
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
    expect(await screen.findByText(/La IA dejó 2 sugerencias en esta ventana/)).toBeInTheDocument();
    expect(screen.getAllByText("Sugerencia de la IA:")[0]!.closest("p")).toHaveTextContent("Sugerencia de la IA: Personas mayores (60 años o más)");
    expect(screen.getByText("Usted dijo: «atendemos a 38 señoras»")).toBeInTheDocument();

    fireEvent.click(screen.getAllByRole("button", { name: "Usar" })[0]!);
    expect(screen.getByRole("checkbox", { name: "Personas mayores (60 años o más)" })).toBeChecked();
    expect(formSave).not.toHaveBeenCalled();
    expect(screen.getByText(/La IA dejó 1 sugerencia en esta ventana/)).toBeInTheDocument();

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
