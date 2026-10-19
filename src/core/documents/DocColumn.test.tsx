import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { DocColumn } from "./DocColumn";
import type { Inside } from "./DocColumn";
import type { DocItem } from "./documentsModel";

const items: DocItem[] = [
  {
    id: "doc_1",
    name: "Contrato de comodato",
    file: "comodato.pdf",
    ext: "pdf",
    type: "Legales",
    year: "2026",
    detail: "comodato.pdf · 2 fragmentos · 2026",
    record: { type: "other", id: "rec_1", custom_name: "Contrato de comodato" },
  },
];

function draw(query: string, inside: Inside | null, onOpen = vi.fn()) {
  render(
    <DocColumn
      items={items}
      query={query}
      onQuery={() => {}}
      mode="type"
      onMode={() => {}}
      open={{}}
      onToggle={() => {}}
      onUpload={() => {}}
      onOpen={onOpen}
      asking={null}
      onAsk={() => {}}
      onRemove={() => {}}
      busy={false}
      inside={inside}
    />,
  );
  return onOpen;
}

describe("DocColumn, the drawer of the Expediente (ADR-036)", () => {
  it("shows each file as its document of the Expediente and opens its window", () => {
    const onOpen = draw("", null);
    expect(screen.getByText("Expediente")).toBeInTheDocument();
    expect(screen.getByText("comodato.pdf · 2 fragmentos · 2026")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Abrir Contrato de comodato" }));
    expect(onOpen).toHaveBeenCalledWith(items[0]);
  });
});

describe("DocColumn, search inside the documents (IA5)", () => {
  it("shows the document, its page and the words around what was asked", () => {
    draw("predial", {
      loading: false,
      passages: [{ chunk: 7, document_id: "doc_1", document: "comodato.pdf", page: 2, snippet: "…del pago de los servicios y del impuesto predial…" }],
    });
    expect(screen.getByRole("region", { name: "Dentro de sus documentos" })).toBeInTheDocument();
    expect(screen.getByText("página 2")).toBeInTheDocument();
    expect(screen.getByText(/impuesto predial/)).toBeInTheDocument();
    // no document is named «predial», but the passage answers: «No encontramos…» would contradict it
    expect(screen.queryByText("No encontramos documentos con eso.")).not.toBeInTheDocument();
  });

  it("says it is looking", () => {
    draw("predial", { loading: true, passages: [] });
    expect(screen.getByText("Buscando dentro de sus documentos…")).toBeInTheDocument();
  });

  it("says when nothing is found by name nor inside", () => {
    draw("zanahoria", { loading: false, passages: [] });
    expect(screen.getByText("No encontramos documentos con eso.")).toBeInTheDocument();
    expect(screen.getByText("No aparece dentro del texto de sus documentos.")).toBeInTheDocument();
  });
});
