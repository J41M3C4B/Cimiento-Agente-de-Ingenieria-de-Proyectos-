import { describe, expect, it } from "vitest";

// DIS-3: the screens stopped using the old AI of «Mi institución». The wrappers in lib/tauri.ts and the types in
// lib/types.ts go with their commands (IA-1, ERP-5), so they are the only places that may still name them.
const sources = import.meta.glob(["../**/*.ts", "../**/*.tsx", "!../**/*.test.ts", "!../**/*.test.tsx", "!../lib/tauri.ts", "!../lib/types.ts"], { query: "?raw", import: "default", eager: true }) as Record<string, string>;

const RETIRED = [
  "assistant_ask",
  "assistantAsk",
  "capture_ask",
  "captureAsk",
  "institution_fill_from_documents",
  "institutionFillFromDocuments",
  "institution_fill_all_from_documents",
  "institutionFillAllFromDocuments",
  "record-suggested",
  "useAssistant",
  "AssistantProvider",
];

describe("the screens and the old AI", () => {
  it("reads the source of the app", () => {
    expect(Object.keys(sources).length).toBeGreaterThan(50);
  });

  it.each(RETIRED)("nothing under src/ uses «%s»", (name) => {
    const users = Object.entries(sources)
      .filter(([, text]) => text.includes(name))
      .map(([path]) => path);
    expect(users).toEqual([]);
  });
});
