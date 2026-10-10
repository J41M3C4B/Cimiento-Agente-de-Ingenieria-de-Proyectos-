import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";

vi.mock("../../lib/tauri", () => ({ profileGet: vi.fn() }));
vi.mock("../access/api", () => ({ accessTeam: vi.fn() }));
vi.mock("../profile/gaps", () => ({ useFillGaps: vi.fn() }));

import { profileGet } from "../../lib/tauri";
import { accessTeam } from "../access/api";
import { useFillGaps } from "../profile/gaps";
import { InstitutionCard } from "./InstitutionCard";

const profile = (name: string) => ({ input: { institution: { name } } }) as unknown as Awaited<ReturnType<typeof profileGet>>;
// the percent comes from Rust (core/overview.rs), already never 100 while something is missing
const gaps = (n: number, percent: number) => ({ ready: true, percent, gaps: Array.from({ length: n }, (_, i) => ({ code: `g${i}`, text: "x", where: "institution" as const })) });

function show(onOpen = vi.fn()) {
  const qc = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  render(
    <QueryClientProvider client={qc}>
      <InstitutionCard onOpen={onOpen} />
    </QueryClientProvider>,
  );
  return onOpen;
}

describe("the dark card of the institution, in Inicio", () => {
  beforeEach(() => vi.clearAllMocks());

  it("while the data is not complete it shows how far they are, with a bar", async () => {
    vi.mocked(profileGet).mockResolvedValue(profile("Casa Hogar Luz"));
    vi.mocked(accessTeam).mockResolvedValue([{ display_name: "Jaime Caballero", role: "admin" }]);
    vi.mocked(useFillGaps).mockReturnValue(gaps(2, 50));
    show();
    await waitFor(() => expect(screen.getByText("Casa Hogar Luz")).toBeInTheDocument());
    expect(await screen.findByRole("progressbar")).toHaveAttribute("aria-valuenow", "50");
    expect(screen.getByText("50 % de sus datos · Faltan 2 datos por llenar")).toBeInTheDocument();
    expect(screen.queryByRole("group", { name: "Quienes tienen acceso" })).not.toBeInTheDocument();
  });

  it("once complete it keeps only the name and the people who have access", async () => {
    vi.mocked(profileGet).mockResolvedValue(profile("Casa Hogar Luz"));
    vi.mocked(accessTeam).mockResolvedValue([
      { display_name: "Jaime Caballero", role: "admin" },
      { display_name: "Rosa Hernández", role: "manager" },
    ]);
    vi.mocked(useFillGaps).mockReturnValue(gaps(0, 100));
    show();
    const group = await screen.findByRole("group", { name: "Quienes tienen acceso" });
    expect(group.querySelectorAll(".avatar")).toHaveLength(2);
    expect(screen.queryByRole("progressbar")).not.toBeInTheDocument();
    expect(screen.queryByText("Datos completos")).not.toBeInTheDocument();
  });

  it("shows how far the data are as Rust counts it, and the whole card opens Mi institución", async () => {
    vi.mocked(profileGet).mockResolvedValue(profile("Casa Hogar Luz"));
    vi.mocked(accessTeam).mockResolvedValue([]);
    vi.mocked(useFillGaps).mockReturnValue(gaps(1, 99));
    const onOpen = show();
    expect(await screen.findByRole("progressbar")).toHaveAttribute("aria-valuenow", "99");
    await userEvent.click(await screen.findByRole("button", { name: /Casa Hogar Luz/ }));
    expect(onOpen).toHaveBeenCalled();
  });
});
