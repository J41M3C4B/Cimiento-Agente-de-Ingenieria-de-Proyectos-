import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { emptySpace, SpaceDialog } from "./GroupDialog";
import type { SpaceData } from "./types";

function dialog(start: SpaceData, onSave = vi.fn().mockResolvedValue(null)) {
  render(<SpaceDialog start={start} isNew kinds={["bedroom", "bathroom", "kitchen", "other"]} floors={2} onSave={onSave} onClose={vi.fn()} />);
  return onSave;
}

describe("a group of spaces", () => {
  it("four bathrooms, three good and one poor, go as counts by state", async () => {
    const onSave = dialog(emptySpace("bathroom"));
    const count = screen.getByLabelText(/¿Cuántos hay\?/);
    await userEvent.clear(count);
    await userEvent.type(count, "4");
    await userEvent.type(screen.getByLabelText(/^Bien/), "3");
    await userEvent.type(screen.getByLabelText(/^Mal/), "1");
    await userEvent.click(screen.getByText("Fugas de agua"));
    await userEvent.click(screen.getByRole("button", { name: "Guardar" }));
    const sent: SpaceData = onSave.mock.calls[0]![0];
    expect([sent.kind, sent.count, sent.good, sent.poor, sent.problems]).toEqual(["bathroom", 4, 3, 1, ["leaks"]]);
  });

  it("what fails is asked only when something is not good, and beds only for bedrooms", async () => {
    dialog(emptySpace("bathroom"));
    expect(screen.queryByText("Fugas de agua")).not.toBeInTheDocument();
    expect(screen.getByText(/¿Tienen barras de apoyo\?/)).toBeInTheDocument();
    expect(screen.queryByLabelText(/¿Cuántas camas/)).not.toBeInTheDocument();
    await userEvent.type(screen.getByLabelText(/^Regular/), "1");
    expect(screen.getByText("Fugas de agua")).toBeInTheDocument();
  });

  it("the ones not counted stay unchecked, and «all good» fills them in", async () => {
    dialog({ ...emptySpace("bedroom"), count: 6, good: 4 });
    expect(screen.getByLabelText(/¿Cuántas camas/)).toBeInTheDocument();
    expect(screen.getByText("2 sin revisar")).toBeInTheDocument();
    await userEvent.click(screen.getByRole("button", { name: "Todos están bien" }));
    expect(screen.queryByText("2 sin revisar")).not.toBeInTheDocument();
  });
});
