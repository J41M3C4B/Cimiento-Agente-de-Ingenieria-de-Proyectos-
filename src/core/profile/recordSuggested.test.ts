import { describe, expect, it } from "vitest";
import { suggestedText } from "./recordReading";

describe("what a file kept left in «Mi institución» (ADR-042 §5)", () => {
  it("says how many suggestions and in which windows", () => {
    const text = suggestedText({
      key: "annual_report",
      kind: "annual_report",
      outcome: {
        status: "proposed",
        fields: [
          ["institution.capacity", "institution.staff_paid_estimate"],
          ["institution.capacity", "institution.staff_volunteer_estimate"],
          ["institution.identity", "institution.services"],
        ],
        dropped: 0,
      },
    });
    expect(text).toContain("3 datos más");
    expect(text).toContain("«Capacidad y cifras rápidas» y «Su institución»");
  });

  it("says nothing when it found nothing or there is no help, and says when the help failed", () => {
    expect(suggestedText({ key: "k", kind: "annual_report", outcome: { status: "nothing", reason: "nothing_found" } })).toBeNull();
    expect(suggestedText({ key: "k", kind: "annual_report", outcome: { status: "unavailable", reason: "not_configured" } })).toBeNull();
    expect(suggestedText({ key: "k", kind: "annual_report", outcome: { status: "unavailable", reason: "network" } })).toContain("Llenar con mis documentos");
  });
});
