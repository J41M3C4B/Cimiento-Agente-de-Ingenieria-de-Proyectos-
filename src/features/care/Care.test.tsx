import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";

vi.mock("./api", () => ({
  carePersonSave: vi.fn(),
  carePersonDelete: vi.fn(),
  carePersonReveal: vi.fn(),
}));
vi.mock("../../lib/tauri", () => ({ toAppError: (e: unknown) => ({ code: "x", message: String(e) }) }));

import * as api from "./api";
import { BeneficiaryWizard } from "./BeneficiaryWizard";
import type { CareFlavor } from "./types";

function wizard(flavor: CareFlavor) {
  render(<BeneficiaryWizard view={null} flavor={flavor} groups={[]} fields={[]} onSaved={vi.fn()} onChange={vi.fn()} onClose={vi.fn()} />);
}

describe("the record of a person served", () => {
  beforeEach(() => vi.clearAllMocks());

  it("a new record starts with the approximate age and sends it instead of a date", async () => {
    vi.mocked(api.carePersonSave).mockResolvedValue({ status: "invalid", issues: [] });
    wizard("elderly_home");
    await userEvent.type(screen.getByLabelText(/^Nombre\(s\)/), "Luz");
    await userEvent.type(screen.getByLabelText(/^Edad aproximada/), "84");
    await userEvent.click(screen.getByRole("button", { name: "Guardar" }));
    await waitFor(() => expect(api.carePersonSave).toHaveBeenCalled());
    const sent = vi.mocked(api.carePersonSave).mock.calls[0]![1];
    expect([sent.first_names, sent.approx_age, sent.birth_date]).toEqual(["Luz", 84, null]);
  });

  it("an elderly home asks for schooling and a children's home for school and the legal situation", async () => {
    wizard("elderly_home");
    expect(screen.getByLabelText(/Último grado de estudios/)).toBeInTheDocument();
    expect(screen.queryByLabelText(/Grado que cursa/)).not.toBeInTheDocument();
    for (let i = 0; i < 3; i++) await userEvent.click(screen.getByRole("button", { name: "Siguiente" }));
    expect(screen.queryByLabelText(/Situación legal/)).not.toBeInTheDocument();
  });

  it("a children's home shows its own data", async () => {
    wizard("children_home");
    expect(screen.getByLabelText(/Grado que cursa/)).toBeInTheDocument();
    for (let i = 0; i < 3; i++) await userEvent.click(screen.getByRole("button", { name: "Siguiente" }));
    expect(screen.getByRole("combobox", { name: /^Situación legal/ })).toBeInTheDocument();
  });

  it("health goes only as categories", async () => {
    wizard("elderly_home");
    for (let i = 0; i < 2; i++) await userEvent.click(screen.getByRole("button", { name: "Siguiente" }));
    expect(screen.getByText(/no escriba diagnósticos, medicinas ni nombres de médicos/)).toBeInTheDocument();
    expect(screen.getByText("Diabetes")).toBeInTheDocument();
  });
});
