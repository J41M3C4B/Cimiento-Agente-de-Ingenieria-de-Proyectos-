import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";

vi.mock("./api", () => ({
  hrPersonSave: vi.fn(),
  hrPersonReveal: vi.fn(),
  hrPersonDelete: vi.fn(),
  hrModalityCreate: vi.fn(),
  hrPositionSave: vi.fn(),
}));
vi.mock("../../lib/tauri", () => ({ toAppError: (e: unknown) => ({ code: "x", message: String(e) }) }));

import * as api from "./api";
import { PersonWizard, emptyPerson } from "./PersonWizard";
import type { ModalityInfo, PersonView, PositionRow, StaffChange } from "./types";

const rules = { relation: "employee", lft_benefits: true, imss: true, pay: "salary", needs_end_date: false, tax_data: true } as const;
const modalities: ModalityInfo[] = [
  { code: "indefinite", title: null, builtin: true, behaves_as: "indefinite", rules },
  { code: "volunteer", title: null, builtin: true, behaves_as: "volunteer", rules: { ...rules, relation: "volunteer", lft_benefits: false, imss: false, pay: "none", tax_data: false } },
];
const positions: PositionRow[] = [
  { id: "pos_1", title: "Cocina", area: "kitchen", duties: null, default_modality: "indefinite", default_schedule: "full_time", reference_pay_mxn: null, authorized_seats: null, reports_to: null, active: true, people: 0 },
];
const change = { overview: { people: [], positions, modalities, custom_fields: [], totals: {} }, profile: null } as unknown as StaffChange;

const stored: PersonView = {
  id: "per_1",
  data: { ...emptyPerson(), first_names: "Ana", position_id: "pos_1", modality: "indefinite", curp: null, rfc: null, nss: null, clabe: null },
  secrets: { curp: true, rfc: false, nss: false, clabe: false },
  masked: { curp: "HEGG••••••••••••04", rfc: null, nss: null, clabe: null },
  start_date_approx: true,
  progress: { personal: { filled: 1, total: 7 }, job: { filled: 0, total: 5 }, emergency: { filled: 0, total: 1 }, pay: { filled: 0, total: 6 }, percent: 5 },
  issues: [],
};

function wizard(view: PersonView | null) {
  const onSaved = vi.fn();
  render(<PersonWizard view={view} positions={positions} modalities={modalities} fields={[]} onSaved={onSaved} onChange={vi.fn()} onModalities={vi.fn()} onClose={vi.fn()} />);
  return onSaved;
}

describe("the record of a person in steps", () => {
  beforeEach(() => vi.clearAllMocks());

  it("saves with the minimum and takes the person to the step of a blocking problem", async () => {
    vi.mocked(api.hrPersonSave).mockResolvedValueOnce({ status: "invalid", issues: [{ code: "position_missing", field: "position_id", blocking: true }] });
    wizard(null);
    await userEvent.type(screen.getByLabelText(/Nombre\(s\)/), "Ana");
    await userEvent.click(screen.getByRole("button", { name: "Guardar" }));
    await waitFor(() => expect(screen.getAllByText("Falta elegir el puesto.").length).toBeGreaterThan(0));
    expect(screen.getByRole("combobox", { name: /^Modalidad/ })).toBeInTheDocument(); // on the job step now
    const sent = vi.mocked(api.hrPersonSave).mock.calls[0]!;
    expect(sent[0]).toBeNull();
    expect(sent[1].first_names).toBe("Ana");
    expect(sent[1].curp).toBe(""); // a new record has nothing stored to keep
  });

  it("picking a position suggests its usual modality and schedule", async () => {
    vi.mocked(api.hrPersonSave).mockResolvedValueOnce({ status: "saved", person: stored, change });
    wizard(null);
    await userEvent.type(screen.getByLabelText(/Nombre\(s\)/), "Ana");
    await userEvent.click(screen.getByRole("button", { name: "Siguiente" }));
    await userEvent.selectOptions(screen.getByRole("combobox", { name: /^Puesto/ }), "pos_1");
    await userEvent.click(screen.getByRole("button", { name: "Guardar" }));
    const sent = vi.mocked(api.hrPersonSave).mock.calls[0]![1];
    expect([sent.position_id, sent.modality, sent.schedule]).toEqual(["pos_1", "indefinite", "full_time"]);
  });

  it("a stored identifier shows covered, is shown only on purpose, and is sent back as «leave it»", async () => {
    vi.mocked(api.hrPersonReveal).mockResolvedValueOnce("HEGG560427MVZRRL04");
    vi.mocked(api.hrPersonSave).mockResolvedValueOnce({ status: "saved", person: stored, change });
    const onSaved = wizard(stored);
    expect(screen.getByText("HEGG••••••••••••04")).toBeInTheDocument();
    await userEvent.click(screen.getByRole("button", { name: "Mostrar" }));
    await screen.findByText("HEGG560427MVZRRL04");
    expect(api.hrPersonReveal).toHaveBeenCalledWith("per_1", "curp");
    await userEvent.click(screen.getByRole("button", { name: "Guardar" }));
    expect(vi.mocked(api.hrPersonSave).mock.calls[0]![1].curp).toBeNull();
    await waitFor(() => expect(onSaved).toHaveBeenCalled());
  });

  it("a volunteer has no pay step", async () => {
    wizard({ ...stored, data: { ...stored.data, modality: "volunteer" } });
    for (let i = 0; i < 3; i++) await userEvent.click(screen.getByRole("button", { name: "Siguiente" }));
    expect(screen.getByText(/no recibe pago: este paso no aplica/)).toBeInTheDocument();
  });
});
