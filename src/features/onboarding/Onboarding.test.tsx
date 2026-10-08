import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";

vi.mock("../../lib/tauri", () => ({
  profileSave: vi.fn(),
  devLoadFixture: vi.fn(),
  toAppError: (e: unknown) => ({ code: "x", message: e instanceof Error ? e.message : String(e) }),
}));

import * as api from "../../lib/tauri";
import type { ProfileView } from "../../lib/types";
import { Onboarding } from "./Onboarding";

function show() {
  const qc = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  render(
    <QueryClientProvider client={qc}>
      <Onboarding />
    </QueryClientProvider>,
  );
  return qc;
}

beforeEach(() => vi.resetAllMocks());

describe("Onboarding", () => {
  it("shows the brand on the left and the first form on the right", () => {
    show();
    expect(screen.getByRole("img", { name: "SociAI" })).toBeInTheDocument();
    expect(screen.getByText("Paso 1 de 3", { selector: "p" })).toBeInTheDocument();
    expect(screen.getByLabelText(/Nombre de la institución/)).toBeInTheDocument();
  });

  it("does not go on without the name", async () => {
    show();
    await userEvent.click(screen.getByRole("button", { name: /Continuar/ }));
    expect(await screen.findByText("Este dato nos falta: el nombre de la institución.")).toBeInTheDocument();
    expect(screen.getByText("Paso 1 de 3", { selector: "p" })).toBeInTheDocument();
  });

  it("walks the three steps, saves the profile once and hands it to the app", async () => {
    const profile = { input: { institution: { name: "Casa Esperanza" } } } as unknown as ProfileView;
    vi.mocked(api.profileSave).mockResolvedValue({ status: "saved", profile });
    const qc = show();
    await userEvent.type(screen.getByLabelText(/Nombre de la institución/), "Casa Esperanza");
    await userEvent.click(screen.getByRole("button", { name: /Continuar/ }));
    await userEvent.type(await screen.findByLabelText("Teléfono de la institución"), "5512345678");
    await userEvent.click(screen.getByRole("button", { name: /Continuar/ }));
    await userEvent.type(await screen.findByLabelText(/Capacidad total/), "40");
    await userEvent.click(screen.getByRole("button", { name: /Empezar/ }));

    await waitFor(() => expect(api.profileSave).toHaveBeenCalledTimes(1));
    const input = vi.mocked(api.profileSave).mock.calls[0]![0];
    expect(input.institution.name).toBe("Casa Esperanza");
    expect(input.institution.contact_phone).toBe("5512345678");
    expect(input.capacity_total).toBe(40);
    await waitFor(() => expect(qc.getQueryData(["profile"])).toBe(profile));
  });
});
