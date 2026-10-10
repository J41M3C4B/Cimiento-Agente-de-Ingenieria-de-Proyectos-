import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { renderHook, waitFor } from "@testing-library/react";
import type { ReactNode } from "react";
import { beforeEach, describe, expect, it, vi } from "vitest";

vi.mock("../onboarding/api", async (orig) => ({ ...(await orig<typeof import("../onboarding/api")>()), onboardingStatus: vi.fn() }));
vi.mock("../../modules/facilities/api", () => ({ facilitiesOverview: vi.fn() }));
vi.mock("../../modules/facilities/FacilitiesTab", () => ({ FACILITIES_KEY: ["facilities"] }));

import { facilitiesOverview } from "../../modules/facilities/api";
import { onboardingStatus } from "../onboarding/api";
import type { OnboardingStatus } from "../onboarding/api";
import { useFillGaps } from "./gaps";

const KEYS = ["institution", "location", "people", "team", "money", "building"];

function status(missing: Record<string, string[]>, records = { served: 3, staff: 2, fee_payers: 0 }): OnboardingStatus {
  return {
    done: true, ready: false, welcomed: true, can_postpone: true, setup: null, records,
    steps: KEYS.map((key) => ({ key, missing: missing[key] ?? [], complete: (missing[key] ?? []).length === 0 })),
  } as OnboardingStatus;
}

function run() {
  const qc = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  const wrapper = ({ children }: { children: ReactNode }) => <QueryClientProvider client={qc}>{children}</QueryClientProvider>;
  return renderHook(() => useFillGaps(), { wrapper });
}

describe("what the institution still has to fill in", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.mocked(facilitiesOverview).mockResolvedValue({ indicators: { spaces: 5 } } as Awaited<ReturnType<typeof facilitiesOverview>>);
  });

  it("is not ready until Rust says what is missing", () => {
    vi.mocked(onboardingStatus).mockReturnValue(new Promise(() => {}));
    const { result } = run();
    expect(result.current).toEqual({ ready: false, gaps: [] });
  });

  it("with nothing missing the data are complete", async () => {
    vi.mocked(onboardingStatus).mockResolvedValue(status({}));
    const { result } = run();
    await waitFor(() => expect(result.current.ready).toBe(true));
    await waitFor(() => expect(vi.mocked(facilitiesOverview)).toHaveBeenCalled());
    expect(result.current.gaps).toEqual([]);
  });

  it("names each missing piece in the order of the steps and says where it is filled in", async () => {
    vi.mocked(onboardingStatus).mockResolvedValue(status({ institution: ["mission"], location: ["state", "legal_form"], people: ["capacity_total"], money: ["income"], building: ["floors"] }));
    const { result } = run();
    await waitFor(() => expect(result.current.ready).toBe(true));
    expect(result.current.gaps.map((g) => [g.code, g.where])).toEqual([
      ["mission", "institution"],
      ["state", "contact"],
      ["legal_form", "legal"],
      ["capacity_total", "capacity"],
      ["income", "finance"],
      ["floors", "facilities"],
    ]);
    expect(result.current.gaps[0]!.text).toBe("A qué se dedica");
  });

  it("reminds of the people and the spaces nobody registered, unless the data above already cover them", async () => {
    vi.mocked(onboardingStatus).mockResolvedValue(status({ team: ["staff"] }, { served: 0, staff: 0, fee_payers: 0 }));
    vi.mocked(facilitiesOverview).mockResolvedValue({ indicators: { spaces: 0 } } as Awaited<ReturnType<typeof facilitiesOverview>>);
    const { result } = run();
    await waitFor(() => expect(result.current.gaps.map((g) => g.code)).toEqual(["staff", "served_records", "spaces"]));
    // the quick figure of the staff is asked for, so «register your staff» does not repeat it
    expect(result.current.gaps.find((g) => g.code === "staff_records")).toBeUndefined();
    expect(result.current.gaps.map((g) => g.where)).toEqual(["capacity", "people", "facilities"]);
  });
});
