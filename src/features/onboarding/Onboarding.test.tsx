import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { act, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";

vi.mock("./api", async (orig) => ({
  ...(await orig<typeof import("./api")>()),
  onboardingStatus: vi.fn(),
  onboardingSave: vi.fn(),
  onboardingFinish: vi.fn(),
  onboardingWelcomeDone: vi.fn(),
}));
vi.mock("../../lib/tauri", () => ({ toAppError: (e: unknown) => ({ code: "x", message: String(e) }), devLoadFixture: vi.fn() }));
vi.mock("../access/session", () => ({ useSession: () => ({ can: () => false }) }));

import * as api from "./api";
import type { OnboardingStatus } from "./api";
import { OnboardingGate, resumeOnboarding } from "./OnboardingGate";

const KEYS = ["institution", "location", "people", "team", "money", "building"];

function status(patch: Partial<OnboardingStatus> = {}): OnboardingStatus {
  return {
    done: false,
    ready: false,
    welcomed: true,
    can_postpone: false,
    setup: null,
    records: { served: 0, staff: 0, fee_payers: 0 },
    steps: KEYS.map((key) => ({ key, missing: key === "institution" ? ["name", "mission"] : ["x"], complete: false })),
    data: {
      institution: { name: "", kind: "elderly_home", mission: null, legal_rfc: null, contact_phone: null, contact_email: null, legal_rep_name: null, state: null, municipality: null, founded_year: null, legal_form: null, authorized_donee: null, cluni: null },
      capacity_total: null, served_estimate: null, staff_paid_estimate: null, staff_volunteer_estimate: null, annual_budget_mxn: null, income: [],
      floors: null, built_m2: null, tenure: null, tenure_until: null, tenure_documented: null,
    },
    ...patch,
  };
}

function gate() {
  const qc = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  render(
    <QueryClientProvider client={qc}>
      <OnboardingGate>
        <p>La app</p>
      </OnboardingGate>
    </QueryClientProvider>,
  );
}

describe("the first start", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    try {
      sessionStorage.clear();
    } catch {
      // no storage in this environment
    }
  });

  it("a person who has not seen the welcome sees it first", async () => {
    vi.mocked(api.onboardingStatus).mockResolvedValue(status({ welcomed: false, done: true }));
    gate();
    expect(await screen.findByText("Le damos la bienvenida a Cimiento")).toBeInTheDocument();
    expect(screen.queryByText("La app")).not.toBeInTheDocument();
  });

  it("the direction cannot skip the data and is told what is missing", async () => {
    vi.mocked(api.onboardingStatus).mockResolvedValue(status());
    vi.mocked(api.onboardingSave).mockResolvedValue({ status: "saved", onboarding: status() });
    gate();
    expect(await screen.findByText("Datos de su institución")).toBeInTheDocument();
    expect(screen.queryByText("Dejarlos a la dirección y entrar")).not.toBeInTheDocument();
    await userEvent.click(screen.getByRole("button", { name: "Guardar y seguir" }));
    await waitFor(() => expect(screen.getByText(/Para seguir falta: El nombre, A qué se dedica\./)).toBeInTheDocument());
    expect(screen.queryByText("La app")).not.toBeInTheDocument();
  });

  it("the administrator may leave the data to the direction and enter", async () => {
    vi.mocked(api.onboardingStatus).mockResolvedValue(status({ can_postpone: true, setup: { ai_ready: false, managers: 0 } }));
    gate();
    await userEvent.click(await screen.findByRole("button", { name: "Dejarlos a la dirección y entrar" }));
    expect(screen.getByText("La app")).toBeInTheDocument();
  });

  it("the administrator who left the data can take them up again from Inicio", async () => {
    vi.mocked(api.onboardingStatus).mockResolvedValue(status({ can_postpone: true, setup: { ai_ready: false, managers: 0 } }));
    gate();
    await userEvent.click(await screen.findByRole("button", { name: "Dejarlos a la dirección y entrar" }));
    expect(screen.getByText("La app")).toBeInTheDocument();
    act(() => resumeOnboarding());
    expect(await screen.findByText("Datos de su institución")).toBeInTheDocument();
    expect(screen.queryByText("La app")).not.toBeInTheDocument();
  });

  it("a finished institution opens the app", async () => {
    vi.mocked(api.onboardingStatus).mockResolvedValue(status({ done: true }));
    gate();
    expect(await screen.findByText("La app")).toBeInTheDocument();
  });
});
