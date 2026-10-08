import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { IssueSummary } from "./IssueSummary";

const STEPS = ["Uno", "Dos"];
function summary(issues: { code: string; field: string; blocking: boolean }[], onGo = vi.fn()) {
  render(<IssueSummary issues={issues} describe={(c) => `texto de ${c}`} stepOf={(f) => (f.startsWith("b") ? 1 : 0)} stepName={(n) => STEPS[n]!} onGo={onGo} />);
  return onGo;
}

describe("what a record asks to review", () => {
  it("says what each thing is, in which step, and takes the person there", async () => {
    const onGo = summary([{ code: "a", field: "a1", blocking: false }, { code: "b", field: "b1", blocking: false }]);
    expect(screen.getByText("Hay 2 cosas por revisar")).toBeInTheDocument();
    expect(screen.getByText("texto de b")).toBeInTheDocument();
    await userEvent.click(screen.getAllByRole("button", { name: "Ver" })[1]!);
    expect(onGo).toHaveBeenCalledWith(1);
  });

  it("keeps what blocks the saving apart from what is only a notice", () => {
    summary([{ code: "a", field: "a1", blocking: true }, { code: "b", field: "b1", blocking: false }]);
    expect(screen.getByText("Falta corregir 1 cosa para guardar")).toBeInTheDocument();
    expect(screen.getByText("Hay 1 cosa por revisar")).toBeInTheDocument();
  });

  it("shows nothing when there is nothing to review", () => {
    summary([]);
    expect(screen.queryByRole("button", { name: "Ver" })).not.toBeInTheDocument();
  });
});
