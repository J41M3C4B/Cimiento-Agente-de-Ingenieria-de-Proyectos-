import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { ProjectRow, ReadingRow } from "../../lib/types";

vi.mock("../../lib/tauri", () => ({
  projectCreateFromCall: vi.fn(),
  projectSetDonorKind: vi.fn(),
  toAppError: (e: unknown) => ({ code: "x", message: String(e) }),
}));

import * as api from "../../lib/tauri";
import { itemsOf, NewProjectForm, withRole, yearIsValid } from "./NewProjectForm";

const project: ProjectRow = {
  id: "proj_1",
  institution_id: "i",
  profile_id: "p",
  title: "Apoyos 2027",
  initial_request: null,
  stage: "DIAGNOSIS",
  needs_review: false,
  created_at: "2026-10-03T10:00:00Z",
  kind: "call",
  call_reading_id: "read_1",
  color: null,
  donor_kind: null,
};

const reading: ReadingRow = {
  id: "read_1",
  name: "Apoyos 2027",
  funder: "Fundación Ficticia",
  year: 2027,
  status: "waiting",
  note: null,
  created_at: "2026-10-03T10:00:00Z",
  finished_at: null,
  confirmed_at: null,
  files: [],
};

const pdf = (name: string, content = "hola") => new File([content], name, { type: "application/pdf" });

function show(onCreated = vi.fn()) {
  const qc = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  render(
    <QueryClientProvider client={qc}>
      <NewProjectForm onCreated={onCreated} onCancel={vi.fn()} />
    </QueryClientProvider>,
  );
  return onCreated;
}

async function fillText(name = "Apoyos 2027", funder = "Fundación Ficticia", year = "2027") {
  await userEvent.type(screen.getByLabelText(/Nombre de la convocatoria/), name);
  await userEvent.type(screen.getByLabelText(/Quién la convoca/), funder);
  await userEvent.type(screen.getByLabelText(/^Año/), year);
}

/** Step 1 done: the files are chosen and the person goes on to the data. */
async function chooseAndGoOn(files: File[], extras = false) {
  if (extras) await userEvent.click(screen.getByLabelText(/Esta convocatoria trae más documentos/));
  await userEvent.upload(screen.getByLabelText(extras ? "Archivos de la convocatoria" : "Archivo de la convocatoria"), files);
}

beforeEach(() => {
  vi.resetAllMocks();
});

describe("the roles of the files of a call", () => {
  it("takes the first file as the call and the rest as annexes", () => {
    const items = itemsOf([pdf("a.pdf"), pdf("b.pdf"), pdf("c.pdf")]);
    expect(items.map((i) => i.role)).toEqual(["main", "annex", "annex"]);
  });

  it("marking a file as the call takes the mark off the one that had it", () => {
    const items = withRole(itemsOf([pdf("a.pdf"), pdf("b.pdf")]), 1, "main");
    expect(items.map((i) => i.role)).toEqual(["annex", "main"]);
    // any other character leaves the rest alone
    expect(withRole(items, 0, "guide").map((i) => i.role)).toEqual(["guide", "main"]);
  });

  it("a year is four digits", () => {
    expect(["2026", " 2026 "].every(yearIsValid)).toBe(true);
    expect(["", "26", "20266", "dos mil"].some(yearIsValid)).toBe(false);
  });
});

describe("NewProjectForm", () => {
  it("is two steps: the files first, and nobody goes on without one", async () => {
    show();
    expect(screen.getByText("Paso 1 de 2 · Suba la convocatoria")).toBeInTheDocument();
    expect(screen.getByText("Siguiente")).toBeDisabled();
    await chooseAndGoOn([pdf("bases.pdf")]);
    expect(screen.getByText("bases.pdf")).toBeInTheDocument();
    expect(screen.getByText("Siguiente")).toBeEnabled();
    await userEvent.click(screen.getByText("Siguiente"));
    expect(screen.getByText("Paso 2 de 2 · Cuéntenos de la convocatoria")).toBeInTheDocument();
    // the person can go back and the file is still there
    await userEvent.click(screen.getByText("Atrás"));
    expect(screen.getByText("bases.pdf")).toBeInTheDocument();
  });

  it("does not let the person start until there is a name, who gives the call and a year", async () => {
    show();
    await chooseAndGoOn([pdf("bases.pdf")]);
    await userEvent.click(screen.getByText("Siguiente"));
    const start = screen.getByText("Empezar el proyecto");
    expect(start).toBeDisabled();
    await fillText("Apoyos 2027", "Fundación Ficticia", "27");
    expect(start).toBeDisabled();
    expect(screen.getByText("Escriba el año con cuatro cifras, por ejemplo 2026.")).toBeInTheDocument();
    await userEvent.type(screen.getByLabelText(/^Año/), "27");
    expect(start).toBeEnabled();
  });

  it("with one file, that file is the call: it goes as text with what the person wrote, and the project opens", async () => {
    vi.mocked(api.projectCreateFromCall).mockResolvedValue({ status: "created", project, reading });
    const onCreated = show();
    await chooseAndGoOn([pdf("bases.pdf")]);
    await userEvent.click(screen.getByText("Siguiente"));
    await fillText();
    await userEvent.click(screen.getByText("Empezar el proyecto"));
    await waitFor(() => expect(onCreated).toHaveBeenCalledWith(project));
    expect(api.projectCreateFromCall).toHaveBeenCalledWith([{ name: "bases.pdf", data: btoa("hola"), role: "main" }], "Apoyos 2027", "Fundación Ficticia", 2027, undefined);
  });

  it("with extras, each file says what it is and exactly one is the call", async () => {
    vi.mocked(api.projectCreateFromCall).mockResolvedValue({ status: "created", project, reading });
    show();
    await chooseAndGoOn([pdf("guia.pdf", "g"), pdf("bases.pdf", "b")], true);
    // the first one was taken as the call; the person says it is the other one
    await userEvent.selectOptions(screen.getByLabelText("¿Qué es «bases.pdf»?"), "main");
    await userEvent.selectOptions(screen.getByLabelText("¿Qué es «guia.pdf»?"), "guide");
    await userEvent.click(screen.getByText("Siguiente"));
    await fillText();
    await userEvent.click(screen.getByText("Empezar el proyecto"));
    await waitFor(() => expect(api.projectCreateFromCall).toHaveBeenCalled());
    expect(vi.mocked(api.projectCreateFromCall).mock.calls[0][0].map((f) => [f.name, f.role])).toEqual([
      ["guia.pdf", "guide"],
      ["bases.pdf", "main"],
    ]);
  });

  it("asks which file is the call when none is marked and does not let the person go on", async () => {
    show();
    await chooseAndGoOn([pdf("a.pdf"), pdf("b.pdf")], true);
    await userEvent.selectOptions(screen.getByLabelText("¿Qué es «a.pdf»?"), "annex");
    expect(screen.getByText(/Marque cuál de los archivos es la convocatoria/)).toBeInTheDocument();
    expect(screen.getByText("Siguiente")).toBeDisabled();
  });

  it("a file that cannot be read takes the person back to the files and says why in plain words", async () => {
    vi.mocked(api.projectCreateFromCall).mockResolvedValue({ status: "unreadable", file: "hojas.pdf", reason: "scanned" });
    const onCreated = show();
    await chooseAndGoOn([pdf("hojas.pdf")]);
    await userEvent.click(screen.getByText("Siguiente"));
    await fillText();
    await userEvent.click(screen.getByText("Empezar el proyecto"));
    expect(await screen.findByText(/hojas.pdf: Ese PDF es una foto de las hojas/)).toBeInTheDocument();
    expect(screen.getByText("Paso 1 de 2 · Suba la convocatoria")).toBeInTheDocument();
    expect(onCreated).not.toHaveBeenCalled();
  });
});
