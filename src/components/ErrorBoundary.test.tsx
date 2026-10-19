import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { ErrorBoundary } from "./ErrorBoundary";

function Broken(): never {
  throw new Error("boom");
}

describe("ErrorBoundary", () => {
  it("shows what happened and a way to go on instead of a blank window", () => {
    const quiet = vi.spyOn(console, "error").mockImplementation(() => {});
    render(
      <ErrorBoundary>
        <Broken />
      </ErrorBoundary>,
    );
    expect(screen.getByText("Algo falló en esta pantalla")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Volver a cargar" })).toBeInTheDocument();
    quiet.mockRestore();
  });

  it("lets the program through when nothing fails", () => {
    render(
      <ErrorBoundary>
        <p>todo bien</p>
      </ErrorBoundary>,
    );
    expect(screen.getByText("todo bien")).toBeInTheDocument();
  });
});
