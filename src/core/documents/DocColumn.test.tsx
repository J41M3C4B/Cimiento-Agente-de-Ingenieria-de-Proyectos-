import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { DocColumn } from "./DocColumn";
import type { Inside } from "./DocColumn";
import type { DocItem } from "./documentsModel";

const items: DocItem[] = [{ id: "doc_1", name: "Contrato de comodato", ext: "pdf", type: "Legales", year: "2026", detail: "Legales" }];

function draw(query: string, inside: Inside | null) {
  render(
    <DocColumn
      col="inst"
      items={items}
      query={query}
      onQuery={() => {}}
      mode="type"
      onMode={() => {}}
      open={{}}
      onToggle={() => {}}
      onUpload={() => {}}
      onDropFile={() => {}}
      asking={null}
      onAsk={() => {}}
      onRemove={() => {}}
      busy={false}
      inside={inside}
    />,
  );
}

describe("DocColumn, search inside the documents (IA5)", () => {
  it("shows the document, its page and the words around what was asked", () => {
    draw("predial", {
      loading: false,
      passages: [{ chunk: 7, document_id: "doc_1", document: "Contrato de comodato", page: 2, snippet: "…del pago de los servicios y del impuesto predial…" }],
    });
    expect(screen.getByRole("region", { name: "Dentro de sus documentos" })).toBeInTheDocument();
    expect(screen.getByText("página 2")).toBeInTheDocument();
    expect(screen.getByText(/impuesto predial/)).toBeInTheDocument();
    // no document is named «predial», but the passage answers: «No encontramos…» would contradict it
    expect(screen.queryByText("No encontramos documentos con eso.")).not.toBeInTheDocument();
  });

  it("says it is looking, and says when the text does not have it either", () => {
    draw("predial", { loading: true, passages: [] });
    expect(screen.getByText("Buscando dentro de sus documentos…")).toBeInTheDocument();
  });

  it("says when nothing is found by name nor inside", () => {
    draw("zanahoria", { loading: false, passages: [] });
    expect(screen.getByText("No encontramos documentos con eso.")).toBeInTheDocument();
    expect(screen.getByText("No aparece dentro del texto de sus documentos.")).toBeInTheDocument();
  });
});
