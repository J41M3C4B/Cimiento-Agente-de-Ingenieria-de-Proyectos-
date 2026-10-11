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
