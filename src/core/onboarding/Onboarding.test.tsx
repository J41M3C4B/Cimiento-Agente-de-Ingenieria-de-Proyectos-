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
// the map of the program (core/places.rs): the core; the frozen modules, Proyectos included, are not places (ADR-043)
vi.mock("../../lib/tauri", () => ({
  toAppError: (e: unknown) => ({ code: "x", message: String(e) }),
  devLoadFixture: vi.fn(),
  appPlaces: vi.fn(async () => ["home", "profile", "documents"].map((id) => ({ id, label: id }))),
}));
vi.mock("../access/session", () => ({ useSession: () => ({ can: () => false }) }));

import * as api from "./api";
import type { OnboardingStatus, StepStatus } from "./api";
import type { FieldSpec } from "../../lib/types";
import { OnboardingGate, resumeOnboarding } from "./OnboardingGate";

// the steps as Rust sends them (ADR-042): each one drawn from the catalog of forms
const field = (id: string, kind: FieldSpec["kind"] = "text"): FieldSpec => ({
  id, kind, options: [], required: true, essential: true, applies_when: { when: "always" }, sensitivity: "public", ai: "as_is", used_by: [], min: null, max: null,
});
const FIELDS: Record<string, FieldSpec[]> = {
  who: [field("institution.name"), field("institution.mission", "long_text")],
  attention: [field("institution.populations", "multi_select"), field("institution.modalities", "multi_select")],
  place: [field("institution.state"), field("institution.municipality")],
  constitution: [field("institution.legal_form"), field("institution.founded_year", "year"), field("institution.authorized_donee"), field("institution.cluni")],
  figures: [field("institution.capacity_total", "number"), field("institution.served_estimate", "number"), field("institution.staff_paid_estimate", "number"), field("institution.staff_volunteer_estimate", "number")],
};
const KEYS = Object.keys(FIELDS);

/** Every step, with what is missing in each (all its fields, unless said). */
function steps(missing: Record<string, string[]> = {}): StepStatus[] {
  return KEYS.map((key) => {
    const m = missing[key] ?? FIELDS[key]!.map((f) => f.id);
    return { key, missing: m, complete: m.length === 0, form: { id: `onboarding.${key}`, sections: [{ id: key, columns: 1, fields: FIELDS[key]! }] } };
  });
}
const done = Object.fromEntries(KEYS.map((k) => [k, []]));

function status(patch: Partial<OnboardingStatus> = {}): OnboardingStatus {
  return { done: false, ready: false, welcomed: true, can_postpone: false, setup: null, records: { served: 0, staff: 0 }, steps: steps(), values: {}, ...patch };
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
    expect(await screen.findByRole("heading", { name: "Le damos la bienvenida a SociAI" })).toBeInTheDocument();
    expect(screen.queryByText(/proyectos|asistente/i)).not.toBeInTheDocument();
    expect(screen.queryByText("La app")).not.toBeInTheDocument();
    // the drawing of the modules shows only what is a place: Documentos, never a frozen one
    expect(await screen.findByText("Documentos")).toBeInTheDocument();
    for (const frozen of ["Proyectos", "Personal", "Beneficiarios", "Finanzas", "Donantes"]) expect(screen.queryByText(frozen)).not.toBeInTheDocument();
  });

  it("the direction cannot skip the data and is told what is missing, in the words of each field", async () => {
    vi.mocked(api.onboardingStatus).mockResolvedValue(status());
    vi.mocked(api.onboardingSave).mockResolvedValue({ status: "saved", onboarding: status() });
    gate();
    expect(await screen.findByRole("heading", { name: "Su institución" })).toBeInTheDocument();
    expect(screen.queryByText("Lo hará la dirección")).not.toBeInTheDocument();
    await userEvent.click(screen.getByRole("button", { name: "Guardar y seguir" }));
    await waitFor(() => expect(screen.getByText(/Para seguir falta: Nombre de la institución, ¿A qué se dedica\?$/)).toBeInTheDocument());
    expect(screen.queryByText("La app")).not.toBeInTheDocument();
  });

  it("a step saves only its own fields, and goes on once Rust says it is complete", async () => {
    vi.mocked(api.onboardingStatus).mockResolvedValue(status({ values: { "institution.state": "jal" } }));
    vi.mocked(api.onboardingSave).mockResolvedValue({ status: "saved", onboarding: status({ steps: steps({ who: [] }) }) });
    gate();
    await userEvent.type(await screen.findByLabelText(/Nombre de la institución/), "Casa Ficticia");
    await userEvent.click(screen.getByRole("button", { name: "Guardar y seguir" }));
    await waitFor(() => expect(api.onboardingSave).toHaveBeenCalledWith("who", { "institution.name": "Casa Ficticia" }, undefined));
    expect(await screen.findByRole("heading", { name: "A quién atienden" })).toBeInTheDocument();
  });

  it("the administrator may leave the data to the direction and enter", async () => {
    vi.mocked(api.onboardingStatus).mockResolvedValue(status({ can_postpone: true, setup: { ai_ready: false, managers: 0 } }));
    gate();
    await userEvent.click(await screen.findByRole("button", { name: "Lo hará la dirección" }));
    expect(screen.getByText("La app")).toBeInTheDocument();
  });

  it("the administrator who left the data can take them up again from Inicio", async () => {
    vi.mocked(api.onboardingStatus).mockResolvedValue(status({ can_postpone: true, setup: { ai_ready: false, managers: 0 } }));
    gate();
    await userEvent.click(await screen.findByRole("button", { name: "Lo hará la dirección" }));
    expect(screen.getByText("La app")).toBeInTheDocument();
    act(() => resumeOnboarding());
    expect(await screen.findByRole("heading", { name: "Su institución" })).toBeInTheDocument();
    expect(screen.queryByText("La app")).not.toBeInTheDocument();
  });

  it("it opens on the first step that is missing, drawn from the catalog", async () => {
    vi.mocked(api.onboardingStatus).mockResolvedValue(status({ steps: steps({ who: [], attention: [] }) }));
    gate();
    expect(await screen.findByRole("heading", { name: "Dónde están" })).toBeInTheDocument();
    expect(screen.getByLabelText(/Municipio o alcaldía/)).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Su institución, Listo" })).toBeInTheDocument();
  });

  it("the people served the records answer are not asked, and it says so", async () => {
    const figures = steps({ ...done, figures: ["institution.capacity_total"] });
    figures[4]!.form.sections[0]!.fields = FIELDS.figures!.filter((f) => f.id !== "institution.served_estimate");
    vi.mocked(api.onboardingStatus).mockResolvedValue(status({ steps: figures, records: { served: 12, staff: 0 } }));
    gate();
    expect(await screen.findByRole("heading", { name: "Cuántas personas son" })).toBeInTheDocument();
    expect(screen.getByText(/Ya hay 12 personas registradas en «Beneficiarios»/)).toBeInTheDocument();
    expect(screen.queryByLabelText(/¿Cuántas personas atienden hoy\?/)).not.toBeInTheDocument();
    expect(screen.getByLabelText(/¿Cuántas personas trabajan con sueldo\?/)).toBeInTheDocument();
  });

  it("the setup tells what is ready from what is pending with words, not only with color", async () => {
    vi.mocked(api.onboardingStatus).mockResolvedValue(status({ can_postpone: true, setup: { ai_ready: true, managers: 0 } }));
    gate();
    expect(await screen.findByText("Llave puesta")).toBeInTheDocument();
    expect(screen.getByText("Lista")).toBeInTheDocument();
    expect(screen.getByText("Todavía no hay cuentas")).toBeInTheDocument();
    expect(screen.getAllByText("Pendiente")).toHaveLength(2);
  });

  it("a finished institution opens the app", async () => {
    vi.mocked(api.onboardingStatus).mockResolvedValue(status({ done: true }));
    gate();
    expect(await screen.findByText("La app")).toBeInTheDocument();
  });
});
