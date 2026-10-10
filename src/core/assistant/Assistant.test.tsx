import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

vi.mock("../../lib/tauri", () => ({
  assistantAsk: vi.fn(),
  manualEntry: vi.fn(async (id: string) => ({ id, title: id === "howto.backup" ? "Hacer un respaldo" : "Folio ante la Junta", paragraphs: ["Qué poner: el folio."], used_by: [] })),
  toAppError: (e: unknown) => ({ code: "x", message: String(e) }),
}));

// as the WebView2 of Windows does today: scrolling returns a promise (an effect that returned it blanked the screen)
Element.prototype.scrollIntoView = function () {
  return Promise.resolve() as unknown as void;
};

import { assistantAsk } from "../../lib/tauri";
import { ManualPanel } from "../../components/ManualHelp";
import { AssistantProvider, useAssistant } from "./Assistant";

function Opener() {
  const a = useAssistant();
  return <button onClick={() => a?.open()}>abrir</button>;
}

function draw(onGo = vi.fn()) {
  const qc = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  render(
    <QueryClientProvider client={qc}>
      <AssistantProvider screen="profile" onGo={onGo}>
        <Opener />
        <ManualPanel id="institution.junta_folio" panelId="p" />
      </AssistantProvider>
    </QueryClientProvider>,
  );
  return onGo;
}

const lastCall = () => {
  const calls = vi.mocked(assistantAsk).mock.calls;
  return calls[calls.length - 1]!;
};

const ask = (text: string) => {
  fireEvent.change(screen.getByLabelText("Su pregunta"), { target: { value: text } });
  fireEvent.click(screen.getByRole("button", { name: "Preguntar" }));
};

describe("Assistant", () => {
  it("answers with its sources and takes the person where it says", async () => {
    vi.mocked(assistantAsk).mockResolvedValueOnce({
      status: "answered",
      asked: "¿Dónde pongo el folio?",
      text: "En Datos legales y fiscales.",
      go_to: "profile.legal",
      sources: [{ id: "institution.junta_folio", title: "Folio ante la Junta" }],
    });
    const onGo = draw();
    fireEvent.click(screen.getByText("abrir"));
    ask("¿Dónde pongo el folio?");
    expect(await screen.findByText("En Datos legales y fiscales.")).toBeInTheDocument();
    expect(screen.getByText(/Lo tomé del manual: Folio ante la Junta/)).toBeInTheDocument();
    expect(vi.mocked(assistantAsk).mock.calls[0]![0]).toEqual({ screen: "profile", field: null, question: "¿Dónde pongo el folio?", history: [] });
    fireEvent.click(screen.getByRole("button", { name: /Ir a Datos legales y fiscales/ }));
    expect(onGo).toHaveBeenCalledWith("profile.legal");
  });

  it("«Preguntar más» opens it about the field, and without the AI the manual answers", async () => {
    vi.mocked(assistantAsk).mockResolvedValueOnce({ status: "manual", asked: "¿Cómo hago un respaldo?", reason: "offline", hits: [{ id: "howto.backup", title: "Hacer un respaldo" }] });
    draw();
    fireEvent.click(await screen.findByRole("button", { name: "Preguntar más" }));
    expect(screen.getByLabelText("Su pregunta")).toHaveValue("¿Qué pongo aquí y para qué sirve?");
    expect(await screen.findByText(/Sobre «Folio ante la Junta»/)).toBeInTheDocument();
    ask("¿Cómo hago un respaldo?");
    expect(await screen.findByText(/No hay conexión con la ayuda automática/)).toBeInTheDocument();
    expect(screen.getByText("Hacer un respaldo")).toBeInTheDocument();
    expect(lastCall()[0].field).toBe("institution.junta_folio");
  });

  it("a question with data of a person waits for the person's decision", async () => {
    vi.mocked(assistantAsk)
      .mockResolvedValueOnce({ status: "quarantine", report: { fields: [{ path: "question", counts: { curp: 1 }, blocking: true, redacted_preview: "¿Dónde pongo [CURP]?" }], counts: { curp: 1 }, has_blocking: true } })
      .mockResolvedValueOnce({ status: "answered", asked: "¿Dónde pongo [CURP]?", text: "En la ficha de la persona.", go_to: null, sources: [] });
    draw();
    fireEvent.click(screen.getByText("abrir"));
    ask("¿Dónde pongo LOPM800101MDFRZN09?");
    fireEvent.click(await screen.findByRole("button", { name: "Tapar y seguir" }));
    await waitFor(() => expect(lastCall()[1]).toBe("redact"));
    expect(await screen.findByText("¿Dónde pongo [CURP]?")).toBeInTheDocument();
    expect(screen.queryByText(/LOPM800101/)).toBeNull();
  });
});
