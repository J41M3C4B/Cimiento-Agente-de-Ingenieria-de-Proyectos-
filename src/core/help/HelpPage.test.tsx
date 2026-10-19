import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { fireEvent, render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";

vi.mock("../../lib/tauri", () => ({ manualSearch: vi.fn(), manualEntry: vi.fn() }));

import { manualEntry, manualSearch } from "../../lib/tauri";
import { HelpPage } from "./HelpPage";

function show() {
  const qc = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  render(
    <QueryClientProvider client={qc}>
      <HelpPage />
    </QueryClientProvider>,
  );
}

describe("the search in the manual, in Ayuda", () => {
  beforeEach(() => vi.clearAllMocks());

  it("finds what answers the doubt and opens it, with no AI", async () => {
    vi.mocked(manualSearch).mockResolvedValue([{ id: "howto.backup", title: "Cambiar de computadora o hacer un respaldo", snippet: "En «Seguridad»…" }]);
    vi.mocked(manualEntry).mockResolvedValue({ id: "howto.backup", title: "Cambiar de computadora o hacer un respaldo", paragraphs: ["En «Seguridad» cree un respaldo."], used_by: [] });
    show();
    fireEvent.change(screen.getByRole("searchbox", { name: "Buscar en el manual" }), { target: { value: "respaldo" } });
    const hit = await screen.findByRole("button", { name: "Cambiar de computadora o hacer un respaldo" });
    expect(manualSearch).toHaveBeenCalledWith("respaldo");
    fireEvent.click(hit);
    expect(await screen.findByText("En «Seguridad» cree un respaldo.")).toBeInTheDocument();
  });

  it("says so when nothing is found, and does not search a word too short", async () => {
    vi.mocked(manualSearch).mockResolvedValue([]);
    show();
    const box = screen.getByRole("searchbox", { name: "Buscar en el manual" });
    fireEvent.change(box, { target: { value: "ab" } });
    await new Promise((r) => setTimeout(r, 350));
    expect(manualSearch).not.toHaveBeenCalled();
    fireEvent.change(box, { target: { value: "zanahoria" } });
    expect(await screen.findByText("No encontramos nada con esas palabras. Pruebe con otras.")).toBeInTheDocument();
  });
});
