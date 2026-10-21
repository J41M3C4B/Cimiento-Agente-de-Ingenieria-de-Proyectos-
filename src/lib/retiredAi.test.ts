import { describe, expect, it } from "vitest";

// DIS-3: the screens stopped using the old AI of «Mi institución». DIS-4: Proyectos is frozen, so no screen offers or
// calls it. The wrappers in lib/tauri.ts and the types in lib/types.ts go with their commands (IA-1, ERP-4, ERP-5), so
// they are the only places that may still name them.
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

// the commands of Proyectos (modules/projects): nothing of the app shows it while it is frozen
const FROZEN = [
  "project_list", "project_create", "project_create_from_call", "project_delete", "project_set_color", "project_set_donor_kind", "project_advance", "project_go_back", "project_job",
  "conversation_get", "conversation_start", "conversation_send", "conversation_retry",
  "diagnosis_summary_generate", "diagnosis_summary_edit", "diagnosis_summary_confirm",
  "needs_get", "needs_propose", "need_add", "need_rate", "need_select",
  "drafting_get", "drafting_set_asks", "drafting_prepare", "section_draft", "sections_draft_all", "sections_confirm_all", "section_save", "section_confirm",
  "budget_save_item", "budget_delete_item", "budget_confirm", "schedule_save_activity", "schedule_delete_activity", "schedule_confirm",
  "review_get", "guide_export", "call_reading_get", "call_reading_retry", "call_reading_confirm", "call_brief_make",
  "modules/projects",
];

// the wrapper of a command in lib/tauri.ts is its name in camel case (`project_list` → `projectList`)
const camel = (name: string) => name.replace(/_([a-z])/g, (_, c: string) => c.toUpperCase());
const FROZEN_WRAPPERS = FROZEN.filter((n) => !n.includes("/")).map(camel);

describe("the screens and the old AI", () => {
  it("reads the source of the app", () => {
    expect(Object.keys(sources).length).toBeGreaterThan(50);
  });

  it.each([...RETIRED, ...FROZEN, ...FROZEN_WRAPPERS])("nothing under src/ uses «%s»", (name) => {
    const users = Object.entries(sources)
      .filter(([, text]) => text.includes(name))
      .map(([path]) => path);
    expect(users).toEqual([]);
  });
});
