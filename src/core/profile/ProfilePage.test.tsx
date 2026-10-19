import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { fireEvent, render, screen } from "@testing-library/react";
import { useState } from "react";
import { describe, expect, it, vi } from "vitest";

vi.mock("../../lib/tauri", () => ({
  profileGet: vi.fn(async () => null),
  institutionOverview: vi.fn(() => new Promise(() => {})),
  institutionLists: vi.fn(async () => ({ board: [], accounts: [] })),
  institutionRecords: vi.fn(() => new Promise(() => {})),
  devLoadFixture: vi.fn(),
  appPlaces: vi.fn(async () => [{ id: "profile.legal", label: "Datos legales y fiscales" }]),
  toAppError: (e: unknown) => ({ code: "x", message: String(e) }),
}));
// the window itself is not what is tried here: only when it opens and closes
vi.mock("./FormWindow", () => ({
  FormWindow: ({ title, onClose }: { title: string; onClose: () => void }) => (
    <div role="dialog" aria-label={title}>
      <button onClick={onClose}>Cerrar ventana</button>
    </div>
  ),
}));

import { institutionOverview, institutionRecords, profileGet } from "../../lib/tauri";
import { ProfilePage } from "./ProfilePage";
import type { Where } from "./gaps";

/** As App holds it: the page, and the window the Assistant asked for. */
function Harness() {
  const [page, setPage] = useState<"home" | "profile">("home");
  const [request, setRequest] = useState<{ where: Where; n: number } | null>(null);
  return (
    <>
      <button onClick={() => setPage("home")}>Inicio</button>
      <button onClick={() => setPage("profile")}>Mi institución</button>
      <button
        onClick={() => {
          setPage("profile");
          setRequest({ where: "legal", n: Date.now() });
        }}
      >
        Ir a Datos legales y fiscales
      </button>
      {page === "profile" && <ProfilePage onGo={() => {}} openWindow={request} onWindowOpened={() => setRequest(null)} />}
    </>
  );
}

describe("ProfilePage", () => {
  it("opens the window the Assistant asked for once; coming back later does not open it again", () => {
    render(
      <QueryClientProvider client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}>
        <Harness />
      </QueryClientProvider>,
    );
    fireEvent.click(screen.getByText("Ir a Datos legales y fiscales"));
    expect(screen.getByRole("dialog", { name: "Datos legales y fiscales" })).toBeInTheDocument();
    fireEvent.click(screen.getByText("Cerrar ventana"));
    expect(screen.queryByRole("dialog")).toBeNull();

    // to Inicio and back: the window stays closed
    fireEvent.click(screen.getByText("Inicio"));
    fireEvent.click(screen.getByText("Mi institución"));
    expect(screen.queryByRole("dialog")).toBeNull();
  });
});

describe("the tiles of «Mi institución»", () => {
  it("with Personal and Beneficiarios frozen, two tiles show the quick figures and open their window; the others are not drawn (ADR-043)", async () => {
    vi.mocked(profileGet).mockResolvedValue({ input: { institution: { name: "Asilo Ficticio" }, capacity_total: 25, served_estimate: 18, staff_paid_estimate: 9, staff_volunteer_estimate: 6 }, issues: [] } as unknown as Awaited<ReturnType<typeof profileGet>>);
    vi.mocked(institutionOverview).mockResolvedValue({
      people: { value: 18, approx: false },
      staff: { value: 15, approx: false },
      capacity: 25,
      spaces: 0,
      balance_annual_mxn: null,
      completion: { gaps: [], percent: 100 },
      suggestions: [],
    } as unknown as Awaited<ReturnType<typeof institutionOverview>>);
    const onGo = vi.fn();
    render(
      <QueryClientProvider client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}>
        <ProfilePage onGo={onGo} />
      </QueryClientProvider>,
    );
    fireEvent.click(await screen.findByRole("button", { name: /Personas que atienden/ }));
    expect(screen.getByRole("dialog", { name: "Capacidad y cifras rápidas" })).toBeInTheDocument();
    fireEvent.click(screen.getByText("Cerrar ventana"));
    expect(screen.getByRole("button", { name: /Personal/ })).toHaveTextContent("15");
    expect(screen.queryByRole("button", { name: /Instalaciones/ })).toBeNull();
    expect(screen.queryByRole("button", { name: /Finanzas/ })).toBeNull();
    expect(onGo).not.toHaveBeenCalled();
  });

  // ERP-2: a document they have on paper counts without its file, and the list says it has none
  it("a document they have without its file says «Sin archivo» in the list", async () => {
    vi.mocked(profileGet).mockResolvedValue({ input: { institution: { name: "Red Ficticia" }, capacity_total: null, served_estimate: null, staff_paid_estimate: 2, staff_volunteer_estimate: 15 }, issues: [] } as unknown as Awaited<ReturnType<typeof profileGet>>);
    const row = (type: string, status: string, file: unknown = null) => ({ id: type, type, custom_name: null, status, issued_on: null, valid_until: null, validity: null, days_left: null, file, origin: null, confirmed_at: null });
    vi.mocked(institutionRecords).mockResolvedValue({
      records: [row("articles_of_incorporation", "have"), row("tax_status_certificate", "have", { name: "constancia.pdf", readable: true, from_picture: false, doubtful: [] }), row("privacy_notice", "missing")],
      summary: { expected: 3, have: 2, in_progress: 0, missing: 1, expiring: 0, expired: 0 },
    } as unknown as Awaited<ReturnType<typeof institutionRecords>>);
    render(
      <QueryClientProvider client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}>
        <ProfilePage onGo={() => {}} />
      </QueryClientProvider>,
    );
    expect(await screen.findByText("constancia.pdf")).toBeInTheDocument();
    expect(screen.getAllByText("Sin archivo")).toHaveLength(1);
  });
});
