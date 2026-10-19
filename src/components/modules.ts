import type { IconName } from "./icons";

/**
 * What each module of the institution looks like (ADR-032): its line icon. Its color is the accent of its page
 * (`accentOf`, docs/13 §3.7), so a module is not given a color of its own here.
 */
export type ModuleId = "projects" | "staff" | "people" | "facilities" | "finance";

export const MODULE_META: Record<ModuleId, { icon: IconName }> = {
  projects: { icon: "folder" },
  staff: { icon: "briefcase" },
  people: { icon: "heart" },
  facilities: { icon: "building" },
  finance: { icon: "wallet" },
};

/** The accent of a page: the module's own, Documentos' own, or the blue of the institution for the rest of the core and the settings. */
export type Accent = "institution" | "documents" | ModuleId;
export const accentOf = (page: string): Accent => (page in MODULE_META ? (page as ModuleId) : page === "documents" ? "documents" : "institution");
