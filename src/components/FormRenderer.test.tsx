import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { fireEvent, render, screen } from "@testing-library/react";
import { useState } from "react";
import { describe, expect, it, vi } from "vitest";

vi.mock("../lib/tauri", () => ({ manualEntry: vi.fn() }));

import { manualEntry } from "../lib/tauri";
import type { FieldSpec, FormSpec, FormValues } from "../lib/types";
import { applies, FormRenderer } from "./FormRenderer";

const field = (id: string, more: Partial<FieldSpec>): FieldSpec => ({
  id, kind: "text", options: [], required: false, applies_when: { when: "always" }, sensitivity: "public", ai: "as_is", used_by: [], min: null, max: null, ...more,
});

// the form «Su institución» as Rust describes it (core/institution/forms.rs), shortened
const SPEC: FormSpec = {
  id: "institution.identity",
  sections: [
    { id: "who", columns: 1, fields: [field("institution.name", { required: true })] },
    {
      id: "attention",
      columns: 1,
      fields: [
        field("institution.populations", { kind: "multi_select", options: ["early_childhood", "childhood", "older_adults"], required: true }),
        field("institution.sex_served", { kind: "select", options: ["women", "men", "all"], applies_when: { when: "filled", field: "institution.populations" } }),
      ],
    },
  ],
};

function Harness({ start, seen }: { start: FormValues; seen: (v: FormValues) => void }) {
  const [values, setValues] = useState(start);
  return (
    <FormRenderer
      spec={SPEC}
      values={values}
      onChange={(v) => {
        seen(v);
        setValues(v);
      }}
      errors={{ "institution.name": "Elija una opción de la lista." }}
    />
  );
}

describe("FormRenderer", () => {
  it("draws each field with its words and shows a field only when it applies", () => {
    let last: FormValues = {};
    render(<Harness start={{ "institution.name": "Casa" }} seen={(v) => (last = v)} />);
    expect(screen.getByLabelText(/Nombre de la institución/)).toHaveValue("Casa");
    expect(screen.queryByText("¿Atienden a mujeres, a hombres o a ambos?")).toBeNull();

    // marked out of order, kept in the order of the list
    fireEvent.click(screen.getByLabelText("Personas mayores (60 años o más)"));
    fireEvent.click(screen.getByLabelText("Niñez (6 a 11 años)"));
    expect(last["institution.populations"]).toEqual(["childhood", "older_adults"]);

    // now the sex served applies, and a short list is a row of pills
    fireEvent.click(screen.getByLabelText("Mujeres y hombres"));
    expect(last["institution.sex_served"]).toBe("all");
    expect(screen.getByRole("alert")).toHaveTextContent("Elija una opción de la lista.");
  });

  it("decides whether a field applies the way Rust does", () => {
    const sex = SPEC.sections[1]!.fields[1]!;
    expect(applies(sex, {})).toBe(false);
    expect(applies(sex, { "institution.populations": [] })).toBe(false);
    expect(applies(sex, { "institution.populations": ["childhood"] })).toBe(true);
    const onlyB = field("x", { applies_when: { when: "any_of", field: "kinds", values: ["b"] } });
    expect(applies(onlyB, { kinds: ["a", "b"] })).toBe(true);
    expect(applies(onlyB, { kinds: "a" })).toBe(false);
  });

  it("the «?» of a field shows what the manual says of it, and the label still leads to its field", async () => {
    vi.mocked(manualEntry).mockResolvedValue({
      id: "institution.name",
      title: "Nombre de la institución",
      paragraphs: ["Qué poner: el nombre con el que se presentan.", "Para qué sirve: aparece en sus proyectos."],
      used_by: ["projects", "ai"],
    });
    const qc = new QueryClient({ defaultOptions: { queries: { retry: false } } });
    render(
      <QueryClientProvider client={qc}>
        <Harness start={{}} seen={() => {}} />
      </QueryClientProvider>,
    );
    // the label names the field, without the «?» in its name
    expect(screen.getByRole("textbox", { name: "Nombre de la institución*" })).toBeInTheDocument();
    fireEvent.click(screen.getByText("Nombre de la institución"));
    expect(screen.queryByText("Qué poner:")).toBeNull();

    const toggles = screen.getAllByRole("button", { name: "¿Qué pongo aquí?" });
    expect(toggles).toHaveLength(2); // one per field shown: the name and whom they serve
    fireEvent.click(toggles[0]!);
    expect(await screen.findByText("Qué poner:")).toBeInTheDocument();
    expect(screen.getByText(/aparece en sus proyectos/)).toBeInTheDocument();
    expect(screen.getByText(/Proyectos y la ayuda automática\./)).toBeInTheDocument();
    expect(toggles[0]).toHaveAttribute("aria-expanded", "true");
    expect(manualEntry).toHaveBeenCalledWith("institution.name");

    // one at a time, and the same «?» closes it
    fireEvent.click(toggles[0]!);
    expect(screen.queryByText("Qué poner:")).toBeNull();
  });
});
