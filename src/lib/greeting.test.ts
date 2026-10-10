import { describe, expect, it } from "vitest";
import { dayPartOf, firstName } from "./greeting";

describe("the greeting follows the hour of the day", () => {
  it("morning from 5:00 to 11:59, afternoon to 18:59, evening from 19:00 to 4:59", () => {
    expect([4, 5, 11].map(dayPartOf)).toEqual(["evening", "morning", "morning"]);
    expect([12, 18].map(dayPartOf)).toEqual(["afternoon", "afternoon"]);
    expect([19, 23, 0].map(dayPartOf)).toEqual(["evening", "evening", "evening"]);
  });

  it("uses the first name only, and nothing when there is no name", () => {
    expect(firstName("  Jaime Alberto Caballero ")).toBe("Jaime");
    expect(firstName("")).toBe("");
    expect(firstName(undefined)).toBe("");
  });
});
