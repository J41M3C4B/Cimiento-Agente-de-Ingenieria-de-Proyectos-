import type { IconName } from "./icons";
import type { Tone } from "./ui";

/**
 * What each module of the institution looks like (ADR-032): its icon and its color, in the rail, in the frame of its
 * own page and in the line «Mi institución» shows for it. The colors keep the meaning they already had: the staff is
 * teal, the people served violet, the facilities sky and the money green. Projects take no color of their own
 * because every project has its own, so the module is the neutral one.
 */
export type ModuleId = "projects" | "staff" | "people" | "facilities" | "finance";

export const MODULE_META: Record<ModuleId, { icon: IconName; tone: Tone }> = {
  projects: { icon: "folder", tone: "ink" },
  staff: { icon: "briefcase", tone: "teal" },
  people: { icon: "heart", tone: "violet" },
  facilities: { icon: "building", tone: "sky" },
  finance: { icon: "wallet", tone: "green" },
};
