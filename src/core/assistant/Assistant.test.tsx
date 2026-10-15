import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

vi.mock("../../lib/tauri", () => ({
  assistantAsk: vi.fn(),
  captureAsk: vi.fn(),
  institutionFillFromDocuments: vi.fn(),
  institutionFillAllFromDocuments: vi.fn(),
  appPlaces: vi.fn(async () => [
    { id: "profile.legal", label: "Datos legales y fiscales" },
    { id: "profile.institution", label: "Su institución" },
    { id: "security", label: "Seguridad" },
  ]),
  manualEntry: vi.fn(async (id: string) => ({ id, title: id === "howto.backup" ? "Hacer un respaldo" : "Folio ante la Junta", paragraphs: ["Qué poner: el folio."], used_by: [] })),
  toAppError: (e: unknown) => ({ code: "x", message: String(e) }),
}));

// as the WebView2 of Windows does today: scrolling returns a promise (an effect that returned it blanked the screen)
Element.prototype.scrollIntoView = function () {
  return Promise.resolve() as unknown as void;
};

import { assistantAsk, captureAsk, institutionFillAllFromDocuments, institutionFillFromDocuments } from "../../lib/tauri";
import { ManualPanel } from "../../components/ManualHelp";
import { AssistantProvider, useAssistant } from "./Assistant";

function Opener() {
  const a = useAssistant();
  return <button onClick={() => a?.open()}>abrir</button>;
}

/** A window of «Mi institución» closing, as FormWindow tells the Assistant. */
function Closer() {
  const a = useAssistant();
  return (
    <>
      <button onClick={() => a?.windowClosed("institution.identity")}>cerrar Su institución</button>
      <button onClick={() => a?.windowClosed("institution.legal")}>cerrar Datos legales</button>
    </>
  );
}

function draw(onGo = vi.fn()) {
  const qc = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  render(
    <QueryClientProvider client={qc}>
      <AssistantProvider screen="profile" onGo={onGo}>
        <Opener />
        <Closer />
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
      documents: [{ document_id: "doc_1", document: "Acta constitutiva", page: 3 }],
      fill: null,
      from_documents: null,
    });
    const onGo = draw();
    fireEvent.click(screen.getByText("abrir"));
    ask("¿Dónde pongo el folio?");
    expect(await screen.findByText("En Datos legales y fiscales.")).toBeInTheDocument();
    expect(screen.getByText(/Lo tomé del manual: Folio ante la Junta/)).toBeInTheDocument();
    expect(screen.getByText("Lo tomé de sus documentos: Acta constitutiva, página 3")).toBeInTheDocument();
    expect(vi.mocked(assistantAsk).mock.calls[0]![0]).toEqual({ screen: "profile", field: null, question: "¿Dónde pongo el folio?", history: [] });
    fireEvent.click(await screen.findByRole("button", { name: /Ir a Datos legales y fiscales/ }));
    expect(onGo).toHaveBeenCalledWith("profile.legal");
    // the next question carries the documents that answer was based on: «ese mismo» has a document
    vi.mocked(assistantAsk).mockResolvedValueOnce({ status: "answered", asked: "¿Y ese mismo?", text: "Sí.", go_to: null, sources: [], documents: [], fill: null, from_documents: null });
    ask("¿Y ese mismo?");
    await screen.findByText("Sí.");
    expect(lastCall()[0].history[1]).toEqual({ from: "assistant", text: "En Datos legales y fiscales.", documents: ["doc_1"] });
  });

  it("«¿Qué documentos tengo?»: the program writes the list by names under the answer, and it goes with the next question", async () => {
    vi.mocked(assistantAsk).mockResolvedValueOnce({
      status: "answered",
      asked: "¿Qué documentos tengo?",
      text: "Esto tiene en su Expediente.",
      go_to: "profile.records",
      sources: [],
      documents: [],
      fill: null,
      from_documents: null,
      records: { have: ["Acta constitutiva", "Poder notarial"], missing: ["CLUNI"], in_progress: [], expiring: [{ name: "Poder notarial", date: "2027-01-18" }], expired: [] },
    });
    draw();
    fireEvent.click(screen.getByText("abrir"));
    ask("¿Qué documentos tengo?");
    expect(await screen.findByText("Tiene 2: Acta constitutiva, Poder notarial.")).toBeInTheDocument();
    expect(screen.getByText("Le falta 1: CLUNI.")).toBeInTheDocument();
    expect(screen.getByText("Por vencer: Poder notarial (18 de enero de 2027).")).toBeInTheDocument();
    vi.mocked(assistantAsk).mockResolvedValueOnce({ status: "answered", asked: "¿Y la CLUNI?", text: "Súbala en el Expediente.", go_to: null, sources: [], documents: [], fill: null, from_documents: null });
    ask("¿Y la CLUNI?");
    await screen.findByText("Súbala en el Expediente.");
    expect(lastCall()[0].history[1].text).toContain("Le falta 1: CLUNI.");
  });

  it("«Preguntar más» opens it about the field, and without the AI the manual answers", async () => {
    vi.mocked(assistantAsk).mockResolvedValueOnce({ status: "manual", asked: "¿Cómo hago un respaldo?", reason: "offline", hits: [{ id: "howto.backup", title: "Hacer un respaldo" }] });
    draw();
    fireEvent.click(await screen.findByRole("button", { name: "Preguntar más" }));
    expect(screen.getByLabelText("Su pregunta")).toHaveValue("¿Qué pongo aquí y para qué sirve?");
    expect(await screen.findByText(/Sobre «Folio ante la Junta»/)).toBeInTheDocument();
    ask("¿Cómo hago un respaldo?");
    expect(await screen.findByText(/No hay conexión a internet/)).toBeInTheDocument();
    expect(screen.getByText(/Revise su conexión/)).toBeInTheDocument();
    expect(screen.getByText("Hacer un respaldo")).toBeInTheDocument();
    expect(lastCall()[0].field).toBe("institution.junta_folio");
  });

  it("says why the AI did not answer and what to do: try again when it is busy, ask the administrator for a key", async () => {
    vi.mocked(assistantAsk)
      .mockResolvedValueOnce({ status: "manual", asked: "¿Qué me falta?", reason: "busy", hits: [] })
      .mockResolvedValueOnce({ status: "answered", asked: "¿Qué me falta?", text: "Le falta el patronato.", go_to: null, sources: [], documents: [], fill: null, from_documents: null })
      .mockResolvedValueOnce({ status: "manual", asked: "¿Y ahora?", reason: "not_configured", hits: [] });
    draw();
    fireEvent.click(screen.getByText("abrir"));
    ask("¿Qué me falta?");
    expect(await screen.findByText("El servicio de ayuda automática está saturado en este momento.")).toBeInTheDocument();
    expect(screen.getByText("Intente de nuevo en unos minutos.")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Intentar de nuevo" }));
    expect(await screen.findByText("Le falta el patronato.")).toBeInTheDocument();
    expect(lastCall()[0].question).toBe("¿Qué me falta?");

    // without a key: whoever may not change the settings is told whom to ask, and there is nothing to retry
    ask("¿Y ahora?");
    expect(await screen.findByText(/falta su llave/)).toBeInTheDocument();
    expect(screen.getByText("Pídale a la persona administradora que la configure.")).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "Ir a «Ayuda automática»" })).toBeNull();
    expect(screen.getAllByRole("button", { name: "Intentar de nuevo" })).toHaveLength(1);
  });

  it("a question with data of a person waits for the person's decision", async () => {
    vi.mocked(assistantAsk)
      .mockResolvedValueOnce({ status: "quarantine", report: { fields: [{ path: "question", counts: { curp: 1 }, blocking: true, redacted_preview: "¿Dónde pongo [CURP]?" }], counts: { curp: 1 }, has_blocking: true } })
      .mockResolvedValueOnce({ status: "answered", asked: "¿Dónde pongo [CURP]?", text: "En la ficha de la persona.", go_to: null, sources: [], documents: [], fill: null, from_documents: null });
    draw();
    fireEvent.click(screen.getByText("abrir"));
    ask("¿Dónde pongo LOPM800101MDFRZN09?");
    fireEvent.click(await screen.findByRole("button", { name: "Tapar y seguir" }));
    await waitFor(() => expect(lastCall()[1]).toBe("redact"));
    expect(await screen.findByText("¿Dónde pongo [CURP]?")).toBeInTheDocument();
    expect(screen.queryByText(/LOPM800101/)).toBeNull();
  });

  it("«Llenarlo por mí» under an answer: the Capturist reads what was asked and leaves suggestions in the window", async () => {
    const asked = "Atendemos a 38 señoras, ¿dónde lo pongo?";
    const fill = { form: "institution.identity", place: "profile.institution" };
    vi.mocked(assistantAsk).mockResolvedValueOnce({ status: "answered", asked, text: "En «Su institución».", go_to: "profile.institution", sources: [], documents: [], fill, from_documents: null });
    vi.mocked(captureAsk)
      .mockResolvedValueOnce({ status: "proposed", asked, form: fill.form, place: fill.place, fields: ["institution.populations", "institution.sex_served"], left: ["¿A quién atienden?", "¿Atienden a mujeres, a hombres o a ambos?"], still: ["¿A qué se dedica?"], dropped: 1 })
      .mockResolvedValueOnce({ status: "unavailable", asked: "Somos I.A.P.", reason: "offline" });
    const onGo = draw();
    fireEvent.click(screen.getByText("abrir"));
    ask(asked);
    fireEvent.click(await screen.findByRole("button", { name: "Llenarlo por mí" }));
    // the program's words: what the code kept, what it did not take and what is still to tell
    expect(await screen.findByText("Dejé 2 sugerencias en «Su institución»: «¿A quién atienden?» y «¿Atienden a mujeres, a hombres o a ambos?».")).toBeInTheDocument();
    expect(screen.getByText(/Lo que habría tenido que calcular o suponer no lo dejé/)).toBeInTheDocument();
    expect(screen.getByText("Todavía puede contarme: «¿A qué se dedica?».")).toBeInTheDocument();
    expect(vi.mocked(captureAsk).mock.calls[0]![0]).toEqual({ form: "institution.identity", said: asked, history: [] });
    expect(screen.getAllByText(asked)).toHaveLength(1);
    fireEvent.click(screen.getByRole("button", { name: "Revisarlas" }));
    expect(onGo).toHaveBeenCalledWith("profile.institution");

    // while it fills the window, what the person writes goes to the Capturist; without the AI it says why, with no manual
    expect(screen.getByText("Llenando «Su institución»")).toBeInTheDocument();
    ask("Somos I.A.P.");
    expect(await screen.findByText(/No hay conexión a internet/)).toBeInTheDocument();
    expect(screen.queryByText("Mientras tanto, le respondo con el manual:")).toBeNull();
    const last = vi.mocked(captureAsk).mock.calls[1]![0];
    expect(last.said).toBe("Somos I.A.P.");
    expect(last.history.map((h) => h.text)).toEqual([asked, "En «Su institución».", expect.stringMatching(/^Dejé 2 sugerencias en «Su institución»: .*Todavía puede contarme: «¿A qué se dedica\?»\.$/)]);

    // back to questions
    fireEvent.click(screen.getByRole("button", { name: "Dejar de llenar" }));
    vi.mocked(assistantAsk).mockResolvedValueOnce({ status: "answered", asked: "¿Y luego?", text: "Guarde la ventana.", go_to: null, sources: [], documents: [], fill: null, from_documents: null });
    ask("¿Y luego?");
    expect(await screen.findByText("Guarde la ventana.")).toBeInTheDocument();
    expect(captureAsk).toHaveBeenCalledTimes(2);
  });

  it("closing the window it fills ends «Llenarlo por mí»: the next question goes to the Assistant", async () => {
    const asked = "Somos una I.A.P.";
    const fill = { form: "institution.identity", place: "profile.institution" };
    vi.mocked(assistantAsk).mockResolvedValueOnce({ status: "answered", asked, text: "Va en «Su institución».", go_to: null, sources: [], documents: [], fill, from_documents: null });
    vi.mocked(captureAsk).mockResolvedValueOnce({ status: "proposed", asked, form: fill.form, place: fill.place, fields: ["institution.populations"], left: ["¿A quién atienden?"], still: [], dropped: 0 });
    draw();
    fireEvent.click(screen.getByText("abrir"));
    ask(asked);
    fireEvent.click(await screen.findByRole("button", { name: "Llenarlo por mí" }));
    await screen.findByText(/Dejé 1 sugerencia/);
    // another window closing changes nothing
    fireEvent.click(screen.getByText("cerrar Datos legales"));
    expect(screen.getByText("Llenando «Su institución»")).toBeInTheDocument();
    // the person saved or closed the window it was filling: filling ends with no click on «Dejar de llenar»
    fireEvent.click(screen.getByText("cerrar Su institución"));
    expect(screen.queryByText("Llenando «Su institución»")).toBeNull();
    vi.mocked(assistantAsk).mockResolvedValueOnce({ status: "answered", asked: "¿Cómo hago un respaldo?", text: "En Seguridad.", go_to: null, sources: [], documents: [], fill: null, from_documents: null });
    ask("¿Cómo hago un respaldo?");
    expect(await screen.findByText("En Seguridad.")).toBeInTheDocument();
    expect(captureAsk).toHaveBeenCalledTimes(1);
  });

  it("«Llenar con mis documentos» under an answer: the Reader fills that window, and the program says how many it left", async () => {
    const legal = { form: "institution.legal", place: "profile.legal" };
    vi.mocked(assistantAsk)
      .mockResolvedValueOnce({ status: "answered", asked: "Llénalo con mi acta", text: "Use el botón.", go_to: null, sources: [], documents: [], fill: null, from_documents: legal })
      .mockResolvedValueOnce({ status: "answered", asked: "Y lo demás", text: "Puede llenar todo.", go_to: null, sources: [], documents: [], fill: null, from_documents: { form: "all", place: "profile" } });
    vi.mocked(institutionFillFromDocuments).mockResolvedValueOnce({ status: "proposed", form: legal.form, place: legal.place, fields: ["institution.legal_name", "institution.purpose"], dropped: 0, note: "" });
    vi.mocked(institutionFillAllFromDocuments).mockResolvedValueOnce([{ status: "nothing", form: legal.form, place: legal.place, reason: "no_documents" }]);
    const onGo = draw();
    fireEvent.click(screen.getByText("abrir"));
    ask("Llénalo con mi acta");
    fireEvent.click(await screen.findByRole("button", { name: "Llenar con mis documentos" }));
    await waitFor(() => expect(institutionFillFromDocuments).toHaveBeenCalledWith("institution.legal"));
    fireEvent.click(await screen.findByRole("button", { name: "Revisarlas" }));
    expect(onGo).toHaveBeenCalledWith("profile.legal");
    ask("Y lo demás");
    fireEvent.click(await screen.findByRole("button", { name: "Llenar todo con mis documentos" }));
    await waitFor(() => expect(institutionFillAllFromDocuments).toHaveBeenCalledTimes(1));
    expect(await screen.findByText(/no tiene documentos|Expediente/)).toBeInTheDocument();
    expect(captureAsk).not.toHaveBeenCalled();
  });
});
