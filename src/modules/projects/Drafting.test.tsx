import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { BudgetItemView, CallRequirements, ConversationView, DraftingView, ProjectRow, ReviewView, SectionView } from "../../lib/types";

vi.mock("../../lib/tauri", () => ({
  conversationStart: vi.fn(),
  conversationSend: vi.fn(),
  conversationRetry: vi.fn(),
  callReadingGet: vi.fn(),
  draftingGet: vi.fn(),
  draftingSetAsks: vi.fn(),
  draftingPrepare: vi.fn(),
  sectionsDraftAll: vi.fn(),
  sectionsConfirmAll: vi.fn(),
  sectionDraft: vi.fn(),
  sectionSave: vi.fn(),
  sectionConfirm: vi.fn(),
  budgetSaveItem: vi.fn(),
  budgetDeleteItem: vi.fn(),
  budgetConfirm: vi.fn(),
  scheduleSaveActivity: vi.fn(),
  scheduleDeleteActivity: vi.fn(),
  scheduleConfirm: vi.fn(),
  reviewGet: vi.fn(),
  guideExport: vi.fn(),
  toAppError: (e: unknown) => ({ code: "x", message: e instanceof Error ? e.message : String(e) }),
}));

import { formatMxn, toNumber } from "../../lib/format";
import * as api from "../../lib/tauri";
import { DraftingStage } from "./DraftingStage";
import { missingParts } from "./ProjectIndex";
import { ReadyStage } from "./ReadyStage";
import { ReviewStage } from "./ReviewStage";

const project: ProjectRow = {
  id: "proj_1",
  institution_id: "i",
  profile_id: "p",
  title: "Apoyos 2027",
  initial_request: null,
  stage: "DRAFTING",
  needs_review: false,
  created_at: "2026-10-03T10:00:00Z",
  kind: "call",
  call_reading_id: "read_1",
  color: null,
  donor_kind: null,
};

const requirements: CallRequirements = {
  max_amount_mxn: { value: 250000, page: 2, file: "bases.pdf" },
  min_amount_mxn: null,
  foreign_currency: false,
  cofunding_percent: { value: 30, page: 4, file: "bases.pdf" },
  admin_cap_percent: null,
  max_duration_months: { value: 12, page: 3, file: "bases.pdf" },
  closing_date: { value: "2027-05-23", page: 4, file: "bases.pdf" },
  required_docs: [],
  conditional_docs: [],
  optional_docs: [],
  formats: [],
  evaluation_criteria: [],
  project_requirements: [],
  fundable: [],
  not_fundable: [],
  indicators: [],
  how_to_deliver: [],
  contact: [],
};

const section = (over: Partial<SectionView> = {}): SectionView => ({
  key: "what",
  title: "Qué se va a hacer",
  guidance: "La idea del proyecto y su objetivo.",
  kind: "text",
  required: true,
  source: "base",
  content: "",
  status: "empty",
  open_points: [],
  unsupported_figures: [],
  plain: null,
  ...over,
});

const view = (over: Partial<DraftingView> = {}): DraftingView => ({
  project,
  asks_for_proposal: false,
  asks_confirmed: false,
  requirements,
  sections: [section()],
  budget: { items: [], totals: { subtotal: 0, vat: 0, total: 0, requested: 0, institution: 0, other: 0, administrative_requested: 0, counterpart_percent: 0, administrative_percent: 0 }, confirmed: false, missing_prices: 0 },
  schedule: { activities: [], duration_months: 0, confirmed: false },
  objective: "Un sistema de mantenimiento preventivo",
  plan_ready: true,
  ...over,
});

function show(ui: React.ReactElement) {
  const qc = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  render(<QueryClientProvider client={qc}>{ui}</QueryClientProvider>);
}

beforeEach(() => {
  vi.resetAllMocks();
});

describe("numbers and money", () => {
  it("reads the numbers a person writes and shows pesos the way they read them", () => {
    expect(toNumber("1,500.50")).toBe(1500.5);
    expect(toNumber(" $ 1 500 ")).toBe(1500);
    expect(Number.isNaN(toNumber(""))).toBe(true);
    expect(Number.isNaN(toNumber("mil"))).toBe(true);
    expect(formatMxn(250000)).toBe("$250,000.00");
  });
});

const proposed = (over: Partial<BudgetItemView> = {}): BudgetItemView => ({
  id: "bud_1",
  category: "material",
  description: "Tubería",
  quantity: 2,
  unit: "pieza",
  unit_price_mxn: 0,
  vat_included: false,
  funded_by: "requested",
  administrative: false,
  origin: "ai_assumption",
  line: { subtotal: 0, vat: 0, total: 0 },
  ...over,
});

const totals = { subtotal: 0, vat: 0, total: 0, requested: 0, institution: 0, other: 0, administrative_requested: 0, counterpart_percent: 0, administrative_percent: 0 };

const conversation: ConversationView = {
  project,
  call: { name: "Apoyos 2027", funder: "Fundación Ficticia", year: 2027 },
  turns: [{ turn: 1, role: "assistant", kind: "root_proposal", level: null, text: "Quedó como usted la dijo.", options: [] }],
  phase: "closed",
  why_level: 0,
  max_whys: 5,
  fit: null,
  root: { text: "Falta un plan", confirmed: true },
  summary: null,
  unsupported_figures: [],
  legacy: false,
};

function stage(onContinue = vi.fn()) {
  show(<DraftingStage view={conversation} onView={vi.fn()} onContinue={onContinue} busy={false} panelOpen />);
  return onContinue;
}

/** Opens a part of the project from the panel's index, in its window. */
const openPart = async (name: string) => userEvent.click(await screen.findByRole("button", { name: new RegExp(`^${name}`) }));

describe("DraftingStage", () => {
  it("says it will build the project, prepares the draft by itself once, and fills the panel", async () => {
    const prepared = view({ plan_ready: true, budget: { items: [proposed()], totals, confirmed: false, missing_prices: 1 } });
    vi.mocked(api.draftingGet).mockResolvedValue(view({ plan_ready: false }));
    vi.mocked(api.draftingPrepare).mockResolvedValue({ view: prepared, ai: "used" });
    stage();
    await waitFor(() => expect(api.draftingPrepare).toHaveBeenCalledWith("proj_1"));
    expect(await screen.findByText(/Ya dejé listo un borrador: 1 partida en el presupuesto y 0 actividades/)).toBeInTheDocument();
    expect(api.draftingPrepare).toHaveBeenCalledTimes(1);
    expect(screen.getByText(/Ahora armaré y redactaré el proyecto/)).toBeInTheDocument();
    // the panel shows each part in one line, and the budget opens in a window with its proposed lines
    expect(screen.getByRole("button", { name: /^Presupuesto.*1 partida/ })).toBeInTheDocument();
    await openPart("Presupuesto");
    expect(await screen.findByText("Tubería")).toBeInTheDocument();
    expect(screen.getByText("Propuesta")).toBeInTheDocument();
  });

  it("does not prepare it again when it was already prepared, and the way on is closed while parts are missing", async () => {
    vi.mocked(api.draftingGet).mockResolvedValue(view());
    stage();
    expect(await screen.findByText("Faltan 3 partes por confirmar.")).toBeInTheDocument();
    expect(screen.getByText("Pasar a la revisión")).toBeDisabled();
    expect(api.draftingPrepare).not.toHaveBeenCalled();
  });

  it("if the assistant could not prepare it, says why and lets the person try again", async () => {
    vi.mocked(api.draftingGet).mockResolvedValue(view({ plan_ready: false }));
    vi.mocked(api.draftingPrepare).mockResolvedValue({ view: view({ plan_ready: false }), ai: "offline" });
    stage();
    expect(await screen.findByText(/No pudimos conectarnos/)).toBeInTheDocument();
    vi.mocked(api.draftingPrepare).mockResolvedValue({ view: view(), ai: "used" });
    await userEvent.click(screen.getByText("Intentar otra vez"));
    await waitFor(() => expect(api.draftingPrepare).toHaveBeenCalledTimes(2));
  });

  it("counts only what has to be confirmed and lets the person on, from the panel, when nothing is missing", async () => {
    const done = view({
      sections: [section({ status: "confirmed", content: "Texto." }), section({ key: "extra", required: false, status: "empty" })],
      budget: { ...view().budget, confirmed: true },
      schedule: { activities: [], duration_months: 0, confirmed: true },
    });
    expect(missingParts(done)).toBe(0);
    vi.mocked(api.draftingGet).mockResolvedValue(done);
    const onContinue = stage();
    expect(await screen.findByText("Todo listo")).toBeInTheDocument();
    await userEvent.click(screen.getByText("Pasar a la revisión"));
    expect(onContinue).toHaveBeenCalled();
  });

  it("asks in the chat whether the call wants a proposal of its own, shows the answer as the person's message, and then asks how to write the texts", async () => {
    vi.mocked(api.draftingGet).mockResolvedValue(view({ asks_for_proposal: true }));
    vi.mocked(api.draftingSetAsks).mockResolvedValue(view({ asks_for_proposal: false, asks_confirmed: true }));
    stage();
    expect(await screen.findByText(/la convocatoria dice qué debe incluir la propuesta/)).toBeInTheDocument();
    // nothing about the texts is asked before this is answered
    expect(screen.queryByText("¿Cómo quiere avanzar con los textos?")).not.toBeInTheDocument();
    await userEvent.click(screen.getByText("No la pide"));
    await waitFor(() => expect(api.draftingSetAsks).toHaveBeenCalledWith("proj_1", false));
    expect(await screen.findByText("¿Cómo quiere avanzar con los textos?")).toBeInTheDocument();
    expect(screen.getByText("Borrador completo")).toBeInTheDocument();
    expect(screen.getByText("Solo una guía breve")).toBeInTheDocument();
    expect(screen.getByText("Los escribo yo")).toBeInTheDocument();
  });

  it("writes the texts only if the person asks: one call for all of them, and then they are confirmed together in the texts window", async () => {
    const drafted = view({
      asks_confirmed: true,
      sections: [
        section({ status: "draft_ai", content: "La institución cambiará las tuberías.", plain: "Cuente qué se va a hacer." }),
        section({ key: "why", title: "Por qué hace falta", status: "draft_ai", content: "Porque el agua sale con óxido." }),
      ],
    });
    vi.mocked(api.draftingGet).mockResolvedValue(view({ asks_confirmed: true, sections: [section(), section({ key: "why", title: "Por qué hace falta" })] }));
    vi.mocked(api.sectionsDraftAll).mockResolvedValue({ view: drafted, ai: "used" });
    vi.mocked(api.sectionsConfirmAll).mockResolvedValue({ ...drafted, sections: drafted.sections.map((s) => ({ ...s, status: "confirmed" as const })) });
    stage();
    // nothing is written until the person chooses how
    await screen.findByText("¿Cómo quiere avanzar con los textos?");
    expect(api.sectionsDraftAll).not.toHaveBeenCalled();
    await userEvent.click(screen.getByText("Borrador completo"));
    await waitFor(() => expect(api.sectionsDraftAll).toHaveBeenCalledWith("proj_1", "full"));
    expect(await screen.findByText(/Listo, dejé 2 textos preparados/)).toBeInTheDocument();
    // the choice is gone from the chat once it was made
    expect(screen.queryByText("Solo una guía breve")).not.toBeInTheDocument();
    await openPart("Textos");
    expect(await screen.findByText("Qué se va a hacer")).toBeInTheDocument();
    await userEvent.click(screen.getByText("Confirmar 2 listas"));
    await waitFor(() => expect(api.sectionsConfirmAll).toHaveBeenCalledWith("proj_1"));
    expect((await screen.findAllByText("Confirmada")).length).toBe(2);
  });

  it("offers a short guide instead, or lets the person write by themselves", async () => {
    vi.mocked(api.draftingGet).mockResolvedValue(view({ asks_confirmed: true }));
    vi.mocked(api.sectionsDraftAll).mockResolvedValue({ view: view({ asks_confirmed: true, sections: [section({ status: "draft_ai", content: "- Qué incluir" })] }), ai: "used" });
    stage();
    await userEvent.click(await screen.findByText("Solo una guía breve"));
    await waitFor(() => expect(api.sectionsDraftAll).toHaveBeenCalledWith("proj_1", "guide"));
  });

  it("«Los escribo yo» calls nothing: the texts are the person's and the assistant says where to find them", async () => {
    vi.mocked(api.draftingGet).mockResolvedValue(view({ asks_confirmed: true }));
    stage();
    await userEvent.click(await screen.findByText("Los escribo yo"));
    expect(await screen.findByText(/Muy bien, los textos son suyos/)).toBeInTheDocument();
    expect(api.sectionsDraftAll).not.toHaveBeenCalled();
  });

  it("a section opens to explain in plain words what it asks, apart from what the call says", async () => {
    vi.mocked(api.draftingGet).mockResolvedValue(
      view({
        sections: [
          section({
            title: "Visita de seguimiento",
            guidance: "El proyecto deberá contemplar una visita de seguimiento por parte de Fundación Alsea.",
            plain: "La Fundación visitará el proyecto para ver el uso de los recursos. Basta con aceptarlo.",
            status: "draft_ai",
            content: "La institución acepta la visita.",
          }),
        ],
      }),
    );
    stage();
    await openPart("Textos");
    await userEvent.click(await screen.findByText("Visita de seguimiento"));
    expect(screen.getByText("En sencillo")).toBeInTheDocument();
    expect(screen.getByText(/Basta con aceptarlo/)).toBeInTheDocument();
    expect(screen.getByText(/El proyecto deberá contemplar una visita de seguimiento/)).toBeInTheDocument();
  });

  it("the person corrects a draft and only then confirms it", async () => {
    const drafted = view({ sections: [section({ status: "draft_ai", content: "La institución cambiará las tuberías.", open_points: ["Quién hará la obra"] })] });
    vi.mocked(api.draftingGet).mockResolvedValue(drafted);
    vi.mocked(api.sectionConfirm).mockResolvedValue(view({ sections: [section({ status: "confirmed", content: "La institución cambiará las tuberías." })] }));
    stage();
    await openPart("Textos");
    await userEvent.click(await screen.findByText("Qué se va a hacer"));
    expect(screen.getByDisplayValue("La institución cambiará las tuberías.")).toBeInTheDocument();
    expect(screen.getByText("Quién hará la obra")).toBeInTheDocument();
    expect(screen.getAllByText("Borrador de la ayuda automática").length).toBeGreaterThan(0);
    await userEvent.click(screen.getByText("Confirmar esta sección"));
    await waitFor(() => expect(api.sectionConfirm).toHaveBeenCalledWith("proj_1", "what"));
  });

  it("warns about figures nobody gave and does not let a changed text be confirmed before it is saved", async () => {
    vi.mocked(api.draftingGet).mockResolvedValue(view({ sections: [section({ status: "draft_ai", content: "Se atenderá a 412 personas.", unsupported_figures: ["412"] })] }));
    stage();
    await openPart("Textos");
    await userEvent.click(await screen.findByText("Qué se va a hacer"));
    expect(screen.getByText(/estas cifras no las dio usted ni constan en el proyecto: 412/)).toBeInTheDocument();
    await userEvent.type(screen.getByLabelText("Qué se va a hacer"), " Corregido.");
    expect(screen.getByText("Confirmar esta sección")).toBeDisabled();
    expect(screen.getByText("Guardar cambios")).toBeEnabled();
  });

  it("the budget lines come proposed: the person writes only the cost, right in the table, and it cannot be confirmed without them", async () => {
    vi.mocked(api.draftingGet).mockResolvedValue(view({ budget: { items: [proposed()], totals, confirmed: false, missing_prices: 1 } }));
    const priced = proposed({ unit_price_mxn: 1000, origin: "user", line: { subtotal: 2000, vat: 320, total: 2320 } });
    vi.mocked(api.budgetSaveItem).mockResolvedValue({
      status: "saved",
      view: view({ budget: { items: [priced], totals: { ...totals, subtotal: 2000, vat: 320, total: 2320, requested: 2320 }, confirmed: false, missing_prices: 0 } }),
    });
    stage();
    await openPart("Presupuesto");
    expect(await screen.findByText("Falta el costo de 1 partida.")).toBeInTheDocument();
    expect(screen.getByText("Confirmar el presupuesto")).toBeDisabled();
    await userEvent.type(screen.getByLabelText(/Costo por unidad: Tubería/), "1,000");
    await userEvent.tab();
    await waitFor(() => expect(api.budgetSaveItem).toHaveBeenCalled());
    expect(vi.mocked(api.budgetSaveItem).mock.calls[0][1]).toMatchObject({ id: "bud_1", description: "Tubería", quantity: 2, unit_price_mxn: 1000 });
    expect(await screen.findAllByText("$2,320.00")).not.toHaveLength(0);
    expect(screen.queryByText("Propuesta")).not.toBeInTheDocument();
    expect(screen.getByText("Confirmar el presupuesto")).toBeEnabled();
  });

  it("sends a new budget line with the numbers as numbers", async () => {
    vi.mocked(api.draftingGet).mockResolvedValue(view());
    const saved = view({
      budget: {
        confirmed: false,
        missing_prices: 0,
        items: [proposed({ unit_price_mxn: 1000, origin: "user", line: { subtotal: 2000, vat: 320, total: 2320 } })],
        totals: { ...totals, subtotal: 2000, vat: 320, total: 2320, requested: 2320 },
      },
    });
    vi.mocked(api.budgetSaveItem).mockResolvedValue({ status: "saved", view: saved });
    stage();
    await openPart("Presupuesto");
    await userEvent.click((await screen.findAllByText("Agregar partida"))[0]);
    await userEvent.type(screen.getByLabelText(/¿Qué se va a comprar o pagar\?/), "Tubería");
    await userEvent.type(screen.getByLabelText(/Tipo de gasto/), "material");
    const quantity = screen.getByLabelText("Cantidad");
    await userEvent.clear(quantity);
    await userEvent.type(quantity, "2");
    await userEvent.type(screen.getByLabelText(/Precio por unidad/), "1,000");
    await userEvent.click(screen.getByText("Guardar partida"));
    await waitFor(() => expect(api.budgetSaveItem).toHaveBeenCalled());
    expect(vi.mocked(api.budgetSaveItem).mock.calls[0][1]).toMatchObject({ id: null, description: "Tubería", category: "material", quantity: 2, unit_price_mxn: 1000, vat_included: false, funded_by: "requested" });
    expect((await screen.findAllByText("$2,320.00")).length).toBeGreaterThan(0);
  });

  it("the schedule comes proposed too, and the person adjusts the months", async () => {
    vi.mocked(api.draftingGet).mockResolvedValue(
      view({ schedule: { activities: [{ id: "act_1", title: "Compra de material", start_month: 1, end_month: 2, origin: "ai_assumption" }], duration_months: 2, confirmed: false } }),
    );
    stage();
    expect(await screen.findByRole("button", { name: /^Cronograma.*1 actividad · 2 meses/ })).toBeInTheDocument();
    await openPart("Cronograma");
    expect(await screen.findAllByText("Compra de material")).not.toHaveLength(0);
    expect(screen.getByText("Del mes 1 al mes 2")).toBeInTheDocument();
    expect(screen.getByText("El proyecto dura 2 meses.")).toBeInTheDocument();
  });

  it("the chat box stays, disabled, and sends the person to the panel", async () => {
    vi.mocked(api.draftingGet).mockResolvedValue(view());
    stage();
    expect(await screen.findByPlaceholderText("Revise el borrador en el panel de la derecha.")).toBeDisabled();
  });
});

describe("ReviewStage", () => {
  const review = (checks: ReviewView["report"]["checks"]): ReviewView => ({
    project: { ...project, stage: "REVIEW" },
    report: { checks, errors: checks.filter((c) => c.level === "error").length, warnings: checks.filter((c) => c.level === "warn").length },
  });

  it("shows what to fix, what to look at and what to keep in mind, and closes the way on while there are errors", async () => {
    vi.mocked(api.reviewGet).mockResolvedValue(
      review([
        { code: "amount_over_max", level: "error", args: [], target: "budget", text: "Lo que pide ($300,000.00) es más de lo que permite la convocatoria ($250,000.00)." },
        { code: "call_closed", level: "warn", args: [], target: "call", text: "La fecha de cierre de la convocatoria (2027-05-23) ya pasó." },
        { code: "docs_to_gather", level: "info", args: [], target: null, text: "La convocatoria pide 3 documentos obligatorios." },
      ]),
    );
    const onBack = vi.fn();
    show(<ReviewStage view={conversation} onView={vi.fn()} onContinue={vi.fn()} onBack={onBack} busy={false} panelOpen />);
    expect(await screen.findByText(/Lo que pide \(\$300,000.00\) es más de lo que permite/)).toBeInTheDocument();
    expect(screen.getByText("Hay que corregir")).toBeInTheDocument();
    expect(screen.getByText("Conviene revisar")).toBeInTheDocument();
    expect(screen.getByText("Para tener en cuenta")).toBeInTheDocument();
    expect(screen.getByText("Se corrige en la redacción.")).toBeInTheDocument();
    expect(screen.getByText("Pasar al último paso")).toBeDisabled();
    // the assistant says it in the chat, and the details stay in the panel
    expect(await screen.findByText(/Encontré 1 cosa que hay que corregir antes de seguir/)).toBeInTheDocument();
    await userEvent.click(screen.getByText("Regresar a la redacción"));
    expect(onBack).toHaveBeenCalled();
  });

  it("with no errors the project goes on, and warnings never block", async () => {
    vi.mocked(api.reviewGet).mockResolvedValue(review([{ code: "fit_partial", level: "warn", args: [], target: "call", text: "La idea solo encaja en parte." }]));
    const onContinue = vi.fn();
    show(<ReviewStage view={conversation} onView={vi.fn()} onContinue={onContinue} onBack={vi.fn()} busy={false} panelOpen />);
    expect(await screen.findByText("Todo está en orden. Puede pasar al último paso.")).toBeInTheDocument();
    expect(screen.getByText(/hay 1 punto que conviene revisar/)).toBeInTheDocument();
    await userEvent.click(screen.getByText("Pasar al último paso"));
    expect(onContinue).toHaveBeenCalled();
  });

  it("asks the program to review again", async () => {
    vi.mocked(api.reviewGet).mockResolvedValue(review([]));
    show(<ReviewStage view={conversation} onView={vi.fn()} onContinue={vi.fn()} onBack={vi.fn()} busy={false} panelOpen />);
    await screen.findByText("Revisar otra vez");
    await userEvent.click(screen.getByText("Revisar otra vez"));
    await waitFor(() => expect(api.reviewGet).toHaveBeenCalledTimes(2));
  });
});

describe("ReadyStage", () => {
  it("makes the guide in Word and says where it was left", async () => {
    vi.mocked(api.guideExport).mockResolvedValue({ file_name: "Apoyos 2027 - guía 2026-10-03.docx", path: "C:\\Users\\Ana\\Downloads\\Apoyos 2027 - guía 2026-10-03.docx" });
    show(<ReadyStage view={conversation} onView={vi.fn()} onBack={vi.fn()} busy={false} panelOpen />);
    expect(screen.getByText(/No sustituye los formatos del donante/)).toBeInTheDocument();
    await userEvent.click(screen.getByText("Generar la guía en Word"));
    await waitFor(() => expect(api.guideExport).toHaveBeenCalledWith("proj_1"));
    expect(await screen.findByText(/La guía quedó en su carpeta de Descargas con el nombre «Apoyos 2027 - guía 2026-10-03.docx»/)).toBeInTheDocument();
    expect(screen.getByText(/C:\\Users\\Ana\\Downloads/)).toBeInTheDocument();
    expect(screen.getByText("Generar otra vez")).toBeInTheDocument();
  });

  it("says why the guide could not be made", async () => {
    vi.mocked(api.guideExport).mockRejectedValue(new Error("La guía traería datos que parecen de una persona."));
    show(<ReadyStage view={conversation} onView={vi.fn()} onBack={vi.fn()} busy={false} panelOpen />);
    await userEvent.click(screen.getByText("Generar la guía en Word"));
    expect(await screen.findByText("La guía traería datos que parecen de una persona.")).toBeInTheDocument();
  });
});
