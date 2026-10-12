import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
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

describe("DocColumn, what each document is (IA6)", () => {
  it("shows the kind in the row and lets the person say another", () => {
    const onType = vi.fn();
    const recognized: DocItem = { ...items[0]!, name: "Constancia SAT", docType: "tax_status_certificate", detail: "Constancia de situación fiscal · 2026" };
    render(
      <DocColumn col="inst" items={[recognized]} query="" onQuery={() => {}} mode="type" onMode={() => {}} open={{}} onToggle={() => {}}
        onUpload={() => {}} onDropFile={() => {}} asking={null} onAsk={() => {}} onRemove={() => {}} busy={false} onType={onType} />,
    );
    expect(screen.getByText(/Constancia de situación fiscal/)).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Decir qué documento es «Constancia SAT»" }));
    expect(onType).toHaveBeenCalledWith(recognized);
  });
});

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
