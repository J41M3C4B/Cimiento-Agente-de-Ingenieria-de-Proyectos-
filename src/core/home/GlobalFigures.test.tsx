import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";

vi.mock("../../lib/tauri", () => ({ profileGet: vi.fn() }));
vi.mock("../../modules/care/api", () => ({ careOverview: vi.fn() }));
vi.mock("../../modules/care/CareTab", () => ({ CARE_KEY: ["care"] }));
vi.mock("../../modules/finance/api", () => ({ FINANCE_KEY: ["finance"], financeGet: vi.fn() }));

import { profileGet } from "../../lib/tauri";
import { careOverview } from "../../modules/care/api";
import { financeGet } from "../../modules/finance/api";
import { GlobalFigures } from "./GlobalFigures";

const profile = (input: object) => ({ input }) as unknown as Awaited<ReturnType<typeof profileGet>>;
const care = (served: number, admitted = 0, discharged = 0, latest: "admitted" | "discharged" | null = null) =>
  ({ board: { indicators: { served, admitted_this_year: admitted, discharged_this_year: discharged, latest_movement: latest } } }) as unknown as Awaited<ReturnType<typeof careOverview>>;
const finance = (balance: number | null, income = 0, expenses: number | null = null) =>
  ({ finances: { balance_annual_mxn: balance, income_annual_mxn: income, income_known: income > 0, expenses_annual_mxn: expenses } }) as unknown as Awaited<ReturnType<typeof financeGet>>;

function show(onGo = vi.fn()) {
  const qc = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  const { container } = render(
    <QueryClientProvider client={qc}>
      <GlobalFigures onGo={onGo} aside={<p>aparte</p>} />
    </QueryClientProvider>,
  );
  return { onGo, container };
}

describe("the figures of the whole institution, at the top of Inicio", () => {
  beforeEach(() => vi.clearAllMocks());

  it("shows the places taken and free, the last movement, the balance of the year and what goes beside them", async () => {
    vi.mocked(profileGet).mockResolvedValue(profile({ capacity_total: 40, served_estimate: null }));
    vi.mocked(careOverview).mockResolvedValue(care(10, 3, 1, "admitted"));
    vi.mocked(financeGet).mockResolvedValue(finance(-1500, 8500, 10000));
    show();
    await waitFor(() => expect(screen.getByText("Vacíos")).toBeInTheDocument());
    expect(screen.getByText("Vacíos").parentElement?.querySelector("b")).toHaveTextContent("30");
    expect(screen.getByText("25 % ocupado")).toBeInTheDocument();
    expect(screen.getByRole("img", { name: "Entraron" }).parentElement?.querySelector("b")).toHaveTextContent("3");
    expect(screen.getByText("aparte")).toBeInTheDocument();
    expect(screen.getByText("−$1,500")).toBeInTheDocument();
  });

  it("of entering and leaving it tells only the one that happened last", async () => {
    vi.mocked(profileGet).mockResolvedValue(profile({ capacity_total: 40, served_estimate: null }));
    vi.mocked(careOverview).mockResolvedValue(care(10, 3, 1, "discharged"));
    vi.mocked(financeGet).mockResolvedValue(finance(null));
    show();
    await waitFor(() => expect(screen.getByRole("img", { name: "Salieron" })).toBeInTheDocument());
    expect(screen.getByRole("img", { name: "Salieron" }).parentElement?.querySelector("b")).toHaveTextContent("1");
    expect(screen.queryByRole("img", { name: "Entraron" })).not.toBeInTheDocument();
  });

  it("with no movement this year it still shows the last one, with its count and no sentence", async () => {
    vi.mocked(profileGet).mockResolvedValue(profile({ capacity_total: 40, served_estimate: null }));
    vi.mocked(careOverview).mockResolvedValue(care(10, 0, 0, "discharged"));
    vi.mocked(financeGet).mockResolvedValue(finance(null));
    show();
    await waitFor(() => expect(screen.getByRole("img", { name: "Salieron" })).toBeInTheDocument());
    expect(screen.getByRole("img", { name: "Salieron" }).parentElement?.querySelector("b")).toHaveTextContent("0");
  });

  it("the places are a waffle with the taken ones filled", async () => {
    vi.mocked(profileGet).mockResolvedValue(profile({ capacity_total: 40, served_estimate: null }));
    vi.mocked(careOverview).mockResolvedValue(care(10));
    vi.mocked(financeGet).mockResolvedValue(finance(null));
    const { container } = show();
    await waitFor(() => expect(screen.getByRole("img", { name: "10 de 40 lugares ocupados" })).toBeInTheDocument());
    expect(container.querySelectorAll(".waffle-cell:not(.waffle-cell--none)")).toHaveLength(40);
    expect(container.querySelectorAll(".waffle-cell--on")).toHaveLength(10);
  });

  it("the year is a half donut with income, expenses and what is available, and the share of the budget in its hollow", async () => {
    vi.mocked(profileGet).mockResolvedValue(profile({ capacity_total: null, served_estimate: null }));
    vi.mocked(careOverview).mockResolvedValue(care(3));
    vi.mocked(financeGet).mockResolvedValue(finance(75000, 300000, 225000));
    const { container } = show();
    await waitFor(() => expect(screen.getByRole("group", { name: "Ingreso $300,000, egresos $225,000, disponible $75,000" })).toBeInTheDocument());
    expect(screen.getByText("Ingreso").previousSibling).toHaveTextContent("$300,000");
    expect(screen.getByText("Egresos").previousSibling).toHaveTextContent("$225,000");
    expect(screen.getByText("Disponible").previousSibling).toHaveTextContent("$75,000");
    expect(screen.getByText("25 %")).toBeInTheDocument();
    expect(screen.getByText(/del presupuesto disponible/)).toBeInTheDocument();
    expect(container.querySelectorAll(".gauge-arc")).toHaveLength(3);
  });

  it("pointing at a color of the donut says what it is and its share of the income", async () => {
    vi.mocked(profileGet).mockResolvedValue(profile({ capacity_total: null, served_estimate: null }));
    vi.mocked(careOverview).mockResolvedValue(care(3));
    vi.mocked(financeGet).mockResolvedValue(finance(75000, 300000, 225000));
    const { container } = show();
    await waitFor(() => expect(container.querySelector(".gauge-arc--spent")).not.toBeNull());
    await userEvent.hover(container.querySelector(".gauge-arc--spent")!);
    expect(screen.getByText("75 %")).toBeInTheDocument();
    expect(screen.getByText("del ingreso se va en egresos")).toBeInTheDocument();
    await userEvent.unhover(container.querySelector(".gauge-arc--spent")!);
    expect(screen.getByText("25 %")).toBeInTheDocument();
    await userEvent.hover(screen.getByText("Ingreso"));
    expect(screen.getByText("100 %")).toBeInTheDocument();
    expect(screen.getByText("es todo lo que entra en el año")).toBeInTheDocument();
  });

  it("with no capacity there is no waffle, and with no spending the donut is only the gray arc", async () => {
    vi.mocked(profileGet).mockResolvedValue(profile({ capacity_total: null, served_estimate: null }));
    vi.mocked(careOverview).mockResolvedValue(care(3));
    vi.mocked(financeGet).mockResolvedValue(finance(null, 5000, null));
    const { container } = show();
    await waitFor(() => expect(screen.getByText("Faltan ingresos o egresos para saber el balance")).toBeInTheDocument());
    expect(container.querySelector(".waffle")).toBeNull();
    expect(container.querySelectorAll(".gauge-arc")).toHaveLength(1);
    expect(screen.getByText("Faltan ingresos o egresos para saber el balance")).toBeInTheDocument();
  });

  it("while nobody is registered, the quick figure stands in and is marked approximate", async () => {
    vi.mocked(profileGet).mockResolvedValue(profile({ capacity_total: 40, served_estimate: 12 }));
    vi.mocked(careOverview).mockResolvedValue(care(0));
    vi.mocked(financeGet).mockResolvedValue(finance(null));
    show();
    await waitFor(() => expect(screen.getByText("≈ 12")).toBeInTheDocument());
  });

  it("each figure opens the module it comes from", async () => {
    vi.mocked(profileGet).mockResolvedValue(profile({ capacity_total: null, served_estimate: null }));
    vi.mocked(careOverview).mockResolvedValue(care(3));
    vi.mocked(financeGet).mockResolvedValue(finance(100));
    const { onGo } = show();
    await userEvent.click(screen.getByRole("button", { name: /Personas que atendemos/ }));
    await userEvent.click(screen.getByRole("button", { name: /Balance del año/ }));
    expect(onGo.mock.calls).toEqual([["people"], ["finance"]]);
  });
});
