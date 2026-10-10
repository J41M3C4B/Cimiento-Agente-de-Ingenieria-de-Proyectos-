import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { renderHook, waitFor } from "@testing-library/react";
import type { ReactNode } from "react";
import { beforeEach, describe, expect, it, vi } from "vitest";

vi.mock("../../lib/tauri", () => ({ institutionOverview: vi.fn() }));

import { institutionOverview } from "../../lib/tauri";
import type { InstitutionOverview, Place } from "../../lib/types";
import { useFillGaps } from "./gaps";

// what is missing and how far, as Rust lists it (core/overview.rs, with its own tests): here only the words
function overview(gaps: [string, Place][], percent: number): InstitutionOverview {
  return {
    people: { value: 0, approx: false }, capacity: null, occupied_percent: null, vacant: null, staff: { value: 0, approx: false }, spaces: 0,
    balance_annual_mxn: null, completion: { gaps: gaps.map(([code, place]) => ({ code, place })), percent },
  };
}

function run() {
  const qc = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  const wrapper = ({ children }: { children: ReactNode }) => <QueryClientProvider client={qc}>{children}</QueryClientProvider>;
  return renderHook(() => useFillGaps(), { wrapper });
}

describe("what the institution still has to fill in", () => {
  beforeEach(() => vi.clearAllMocks());

  it("is not ready until Rust says what is missing", () => {
    vi.mocked(institutionOverview).mockReturnValue(new Promise(() => {}));
    expect(run().result.current).toEqual({ ready: false, gaps: [], percent: 0 });
  });

  it("with nothing missing the data are complete", async () => {
    vi.mocked(institutionOverview).mockResolvedValue(overview([], 100));
    const { result } = run();
    await waitFor(() => expect(result.current.ready).toBe(true));
    expect(result.current).toEqual({ ready: true, gaps: [], percent: 100 });
  });

  it("names each missing piece in the words of the person and keeps where Rust says it is filled in", async () => {
    vi.mocked(institutionOverview).mockResolvedValue(
      overview([["mission", "institution"], ["populations", "institution"], ["state", "contact"], ["staff_records", "staff"], ["spaces", "facilities"]], 80),
    );
    const { result } = run();
    await waitFor(() => expect(result.current.ready).toBe(true));
    expect(result.current.gaps.map((g) => [g.text, g.where])).toEqual([
      ["A qué se dedica", "institution"],
      ["A quién atienden", "institution"],
      ["El estado", "contact"],
      ["Todavía no registra a su personal", "staff"],
      ["Todavía no registra sus instalaciones", "facilities"],
    ]);
    expect(result.current.percent).toBe(80);
  });
});
