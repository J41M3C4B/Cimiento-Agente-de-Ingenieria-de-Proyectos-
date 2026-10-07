import type { ProjectColor } from "./types";

/**
 * The color of a project, as the person chose it, mapped to the accents of the visual system (docs/13 §15).
 * The database and the backend keep the old names (`PROJECT_COLORS` in src-tauri); this is the only table that
 * translates them. `amber` and `red` are not offered when choosing (docs/13 §3.4: they mean «attention» and «wrong»)
 * but a project that already has one keeps showing it.
 */
export type AccentTone = "sky" | "violet" | "rose" | "red" | "amber" | "green" | "teal" | "cyan";

export const PROJECT_TONE: Record<ProjectColor, AccentTone> = {
  blue: "sky",
  violet: "violet",
  pink: "rose",
  orange: "cyan",
  teal: "teal",
  green: "green",
  amber: "amber",
  red: "red",
};

/** The colors offered in the picker, in the order they are shown. */
export const PICKABLE_PROJECT_COLORS: ProjectColor[] = ["blue", "violet", "pink", "orange", "teal", "green"];

/** The color a project has when nobody picked one: always the same for the same project. */
export const defaultProjectColor = (id: string): ProjectColor =>
  PICKABLE_PROJECT_COLORS[[...id].reduce((h, c) => (h * 31 + c.charCodeAt(0)) >>> 0, 7) % PICKABLE_PROJECT_COLORS.length]!;

export const projectTone = (color: ProjectColor | null, id: string): AccentTone => PROJECT_TONE[color ?? defaultProjectColor(id)];
