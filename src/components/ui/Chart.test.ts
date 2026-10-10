import { describe, expect, it } from "vitest";
import { gaugeShares, waffleShape } from "./Chart";

describe("waffle", () => {
  it("one square per place, the taken ones filled", () => {
    const s = waffleShape(40, 10, 1.7);
    expect(s).toMatchObject({ cells: 40, filled: 10, rows: 5, cols: 8 });
    // a taller space asks for more rows than columns
    expect(waffleShape(40, 10, 0.9).rows).toBeGreaterThan(waffleShape(40, 10, 0.9).cols);
  });
  it("never fills more than there are places, and a single person always shows", () => {
    expect(waffleShape(10, 25).filled).toBe(10);
    expect(waffleShape(500, 1).filled).toBe(1);
    expect(waffleShape(40, 0).filled).toBe(0);
  });
  it("a big capacity is shown in parts of one hundred", () => {
    const s = waffleShape(400, 100);
    expect(s.cells).toBe(100);
    expect(s.filled).toBe(25);
  });
  it("no places, no squares", () => {
    expect(waffleShape(0, 3).cells).toBe(0);
  });
});

describe("gauge", () => {
  it("spent and left are parts of the income", () => {
    expect(gaugeShares(300, 225)).toEqual({ spent: 0.75, left: 0.25 });
  });
  it("when spending passes income the whole arc is spending and nothing is left", () => {
    expect(gaugeShares(100, 200)).toEqual({ spent: 1, left: 0 });
  });
  it("nothing to show without figures", () => {
    expect(gaugeShares(0, 0)).toBeNull();
    expect(gaugeShares(-1, 5)).toBeNull();
  });
});
