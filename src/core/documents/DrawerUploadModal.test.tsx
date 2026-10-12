import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import type { Expediente, InstitutionRecord } from "../../lib/types";

const record = (type: string, id: string | null, file = false, custom_name: string | null = null): InstitutionRecord => ({
  id,
  type,
  custom_name,
  status: file ? "have" : "missing",
  issued_on: null,
  valid_until: null,
  validity: null,
  days_left: null,
  file: file ? { name: "acta.pdf", readable: true, from_picture: false, doubtful: [] } : null,
  origin: null,
  confirmed_at: null,
});
const expediente: Expediente = {
  records: [record("articles_of_incorporation", "rec_1", true), record("tax_status_certificate", null), record("other", "rec_9", false, "Contrato de comodato")],
  summary: { expected: 2, have: 1, in_progress: 0, missing: 1, expiring: 0, expired: 0 },
};

vi.mock("../../lib/tauri", () => ({ institutionRecords: vi.fn(async () => expediente) }));

import { DrawerUploadModal } from "./DrawerUploadModal";

function draw() {
  const onPick = vi.fn();
  const qc = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  render(
    <QueryClientProvider client={qc}>
      <DrawerUploadModal onPick={onPick} onClose={() => {}} />
    </QueryClientProvider>,
  );
  return onPick;
}

describe("DrawerUploadModal, the tray (ADR-036)", () => {
  it("asks the drawer and which document of the Expediente it is, then hands over that document", async () => {
    const onPick = draw();
    expect(screen.getByText("¿A qué cajón va?")).toBeInTheDocument();
    expect(screen.getByRole("radio", { name: /Expediente de la institución/ })).toBeChecked();
    expect(screen.getByRole("button", { name: "Seguir" })).toBeDisabled();
    expect(await screen.findByRole("option", { name: "Acta constitutiva (ya tiene archivo)" })).toBeInTheDocument();
    fireEvent.change(screen.getByLabelText("¿Qué documento es?"), { target: { value: "tax_status_certificate" } });
    fireEvent.click(screen.getByRole("button", { name: "Seguir" }));
    expect(onPick).toHaveBeenCalledWith({ type: "tax_status_certificate", id: null, custom_name: null });
  });

  it("offers the documents the institution named and a new «Otro documento»", async () => {
    const onPick = draw();
    await screen.findByRole("option", { name: "Contrato de comodato" });
    fireEvent.change(screen.getByLabelText("¿Qué documento es?"), { target: { value: "other:rec_9" } });
    fireEvent.click(screen.getByRole("button", { name: "Seguir" }));
    expect(onPick).toHaveBeenLastCalledWith({ type: "other", id: "rec_9", custom_name: "Contrato de comodato" });
    fireEvent.change(screen.getByLabelText("¿Qué documento es?"), { target: { value: "other:new" } });
    fireEvent.click(screen.getByRole("button", { name: "Seguir" }));
    expect(onPick).toHaveBeenLastCalledWith({ type: "other", id: null });
  });
});
