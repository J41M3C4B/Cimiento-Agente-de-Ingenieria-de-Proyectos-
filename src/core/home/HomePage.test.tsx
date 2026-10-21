import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

// the pieces under Inicio are not what is tried here: only what Inicio itself offers
vi.mock("./GlobalFigures", () => ({ GlobalFigures: ({ aside }: { aside: React.ReactNode }) => <div>{aside}</div> }));
vi.mock("./InstitutionCard", () => ({ InstitutionCard: () => <p>tarjeta de la institución</p> }));
vi.mock("../onboarding/api", () => ({ ONBOARDING_KEY: ["onboarding"], onboardingStatus: vi.fn(async () => ({ done: true, setup: null, records: { staff: 0, served: 0 } })) }));
vi.mock("../onboarding/OnboardingGate", () => ({ resumeOnboarding: vi.fn() }));
vi.mock("../access/session", () => ({ useSession: () => ({ session: { display_name: "Prueba" }, can: () => false }) }));
vi.mock("../places", () => ({ useIsPlace: () => () => false, usePlaceName: () => () => "" }));
vi.mock("../profile/gaps", () => ({ useOverview: () => ({ isSuccess: false }) }));
vi.mock("../../modules/facilities/api", () => ({ facilitiesOverview: vi.fn() }));
vi.mock("../../modules/facilities/FacilitiesTab", () => ({ FACILITIES_KEY: ["facilities"] }));
vi.mock("../../lib/tauri", () => ({ projectList: vi.fn() }));

import { projectList } from "../../lib/tauri";
import { HomePage } from "./HomePage";

// DIS-4: Proyectos is frozen, so Inicio offers nothing of it and does not even ask for it
describe("Inicio without Proyectos", () => {
  it("has no «Empezar un proyecto nuevo», no folder of projects in progress and never asks for the projects", async () => {
    render(
      <QueryClientProvider client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}>
        <HomePage onGoProfile={() => {}} onGoPlace={() => {}} onGo={() => {}} onGoAi={() => {}} />
      </QueryClientProvider>,
    );
    expect(await screen.findByText("Por hacer")).toBeInTheDocument();
    expect(screen.getByText("tarjeta de la institución")).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: /proyecto/i })).toBeNull();
    expect(screen.queryByText("En curso")).toBeNull();
    expect(projectList).not.toHaveBeenCalled();
  });
});
