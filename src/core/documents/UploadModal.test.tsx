import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { UploadModal } from "./UploadModal";

function draw() {
  const onSubmit = vi.fn();
  const { container } = render(
    <UploadModal target={{ col: "inst", label: "Legales", type: "Legales" }} donors={[]} busy={false} notice={null} onSubmit={onSubmit} onClose={() => {}} />,
  );
  const pick = (file: File) => fireEvent.change(container.querySelector("input[type=file]")!, { target: { files: [file] } });
  return { onSubmit, pick };
}

describe("UploadModal", () => {
  it("a PDF or a photo goes as it is, for Rust to read it, with nothing to paste", () => {
    const { onSubmit, pick } = draw();
    const scan = new File(["%PDF-1.4"], "acta escaneada.pdf", { type: "application/pdf" });
    pick(scan);
    expect(screen.getByText(/Si está escaneado o es una foto, leemos las imágenes/)).toBeInTheDocument();
    expect(screen.queryByLabelText(/Pegue aquí el texto/)).not.toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Guardar" }));
    expect(onSubmit).toHaveBeenCalledWith(expect.objectContaining({ name: "acta escaneada", blob: scan, text: "" }));
  });

  it("a kind it cannot read asks to paste its text", () => {
    const { onSubmit, pick } = draw();
    pick(new File(["x"], "presentacion.pptx"));
    expect(screen.getByText(/Este tipo de archivo todavía no lo leemos/)).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Guardar" })).toBeDisabled();
    fireEvent.change(screen.getByLabelText(/Pegue aquí el texto/), { target: { value: "El texto de la presentación." } });
    fireEvent.click(screen.getByRole("button", { name: "Guardar" }));
    expect(onSubmit).toHaveBeenCalledWith(expect.objectContaining({ text: "El texto de la presentación.", blob: null }));
  });
});
