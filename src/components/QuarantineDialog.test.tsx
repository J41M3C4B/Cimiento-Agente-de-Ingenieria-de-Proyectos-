import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import type { QuarantineReport } from "../lib/types";
import { QuarantineDialog } from "./QuarantineDialog";

const report = (blocking: boolean): QuarantineReport => ({
  has_blocking: blocking,
  counts: blocking ? { curp: 1 } : { phone: 1 },
  fields: [
    {
      path: "notes",
      counts: blocking ? { curp: 1 } : { phone: 1 },
      blocking,
      redacted_preview: blocking ? "CURP [CURP OCULTA]" : "Llamar al [TELÉFONO OCULTO]",
    },
  ],
});

describe("QuarantineDialog", () => {
  it("shows the covered text and a plain summary", () => {
    render(<QuarantineDialog report={report(true)} onRedact={() => {}} onNotPersonal={() => {}} onCancel={() => {}} />);
    expect(screen.getByText(/Encontramos 1 posible CURP/)).toBeInTheDocument();
    expect(screen.getByText("CURP [CURP OCULTA]")).toBeInTheDocument();
  });

  it("does not offer 'not personal data' when something is blocking", () => {
    render(<QuarantineDialog report={report(true)} onRedact={() => {}} onNotPersonal={() => {}} onCancel={() => {}} />);
    expect(screen.queryByText("Esto no son datos personales de nadie")).not.toBeInTheDocument();
  });

  it("offers it for warnings only, and the buttons call back", async () => {
    const onRedact = vi.fn();
    const onNotPersonal = vi.fn();
    const onCancel = vi.fn();
    render(<QuarantineDialog report={report(false)} onRedact={onRedact} onNotPersonal={onNotPersonal} onCancel={onCancel} />);
    await userEvent.click(screen.getByText("Esto no son datos personales de nadie"));
    await userEvent.click(screen.getByText("Tapar y seguir"));
    await userEvent.click(screen.getByText("Mejor no guardarlo"));
    expect(onNotPersonal).toHaveBeenCalledOnce();
    expect(onRedact).toHaveBeenCalledOnce();
    expect(onCancel).toHaveBeenCalledOnce();
  });
});
