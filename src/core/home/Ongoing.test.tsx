import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";

vi.mock("../../modules/projects/projectCall", () => ({
  shortDate: () => "10 feb",
  stepInfo: () => ({ at: 1, done: false, tag: "Paso 2 · Diagnóstico" }),
  useProjectCall: () => ({ funder: "Nacional Monte de Piedad", closes: null, days: null }),
}));

import type { ProjectRow } from "../../lib/types";
import { Ongoing } from "./Ongoing";

const project = (id: string, stage = "DIAGNOSIS") => ({ id, title: `Proyecto ${id}`, stage, color: null }) as unknown as ProjectRow;

function show(projects: ProjectRow[], loaded = true) {
  const fns = { onOpenProject: vi.fn(), onNewProject: vi.fn(), onGoProjects: vi.fn() };
  render(<Ongoing projects={projects} loaded={loaded} {...fns} />);
  return fns;
}

describe("the long folder of Inicio", () => {
  it("has two tabs and opens on what is under way, with the project in progress and its next step", async () => {
    const { onOpenProject } = show([project("a"), project("b", "READY")]);
    expect(screen.getByRole("tab", { name: /En curso/ })).toHaveAttribute("aria-selected", "true");
    expect(screen.getByRole("tab", { name: "Notificaciones" })).toHaveAttribute("aria-selected", "false");
    expect(screen.getByText("Proyecto a")).toBeInTheDocument();
    expect(screen.getByText("Contar su idea y llegar a la causa de fondo")).toBeInTheDocument();
    await userEvent.click(screen.getByRole("button", { name: /Proyecto b/ }));
    expect(onOpenProject).toHaveBeenCalledWith("b");
  });

  it("counts only the projects that are under way on its tab", () => {
    show([project("a"), project("b"), project("c", "READY")]);
    expect(screen.getByRole("tab", { name: /En curso/ })).toHaveTextContent("2");
  });

  it("the other tab is the empty place of the notices", async () => {
    show([project("a")]);
    await userEvent.click(screen.getByRole("tab", { name: "Notificaciones" }));
    expect(screen.getByText("Todo al día")).toBeInTheDocument();
    expect(screen.queryByText("Proyecto a")).not.toBeInTheDocument();
  });

  it("with no projects it invites to start one, and says nothing while they are still being read", async () => {
    const { onNewProject } = show([]);
    await userEvent.click(screen.getByRole("button", { name: /Empezar un proyecto nuevo/ }));
    expect(onNewProject).toHaveBeenCalled();
  });

  it("while the projects are not read yet it does not claim there are none", () => {
    show([], false);
    expect(screen.queryByText("Todavía no tiene proyectos.")).not.toBeInTheDocument();
  });
});
