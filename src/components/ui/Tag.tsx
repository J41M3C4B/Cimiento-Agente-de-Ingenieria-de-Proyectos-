import type { ReactNode } from "react";
import { Icon } from "../icons";
import type { IconName } from "../icons";

/** The colors of the system (docs/13 §3.2) plus `ink` (black), `neutral` (gray) and `pc` (the project's own color). */
export type Tone = "sky" | "violet" | "rose" | "red" | "amber" | "green" | "teal" | "cyan" | "ink" | "neutral" | "pc";
export type TagTone = Tone;

/** The colors that tell people, positions and groups apart (red is left for what is wrong, amber for «attention»). */
const SERIES: Tone[] = ["sky", "violet", "teal", "rose", "green", "cyan", "amber"];
export const seriesTone = (index: number): Tone => SERIES[((index % SERIES.length) + SERIES.length) % SERIES.length]!;
/** The same color every time for the same text. */
export const toneOfText = (text: string): Tone => seriesTone([...text].reduce((h, c) => (h * 31 + c.charCodeAt(0)) >>> 0, 7));

/** A short label: a state or a category. `solid` for states, `soft` for context, `line` for neutral facts. Always with words. */
export function Tag({
  tone = "neutral",
  variant = "solid",
  icon,
  children,
  className = "",
}: {
  tone?: Tone;
  variant?: "solid" | "soft" | "line";
  icon?: IconName;
  children: ReactNode;
  className?: string;
}) {
  const v = variant === "solid" ? "" : `tag--${variant}`;
  return (
    <span className={`tag tone-${tone} ${v} ${className}`}>
      {icon && <Icon name={icon} size={14} strokeWidth={2.6} />}
      {children}
    </span>
  );
}

/** The round mark of a person: their initials, in a color of their own. */
export function Avatar({ name, tone, size = "md" }: { name?: string; tone?: Tone; size?: "sm" | "md" }) {
  const initials =
    (name ?? "")
      .split(/\s+/)
      .filter(Boolean)
      .slice(0, 2)
      .map((w) => w[0]!.toUpperCase())
      .join("") || "·";
  return (
    <span aria-hidden="true" className={`avatar tone-${tone ?? toneOfText(name ?? "")} ${size === "sm" ? "avatar--sm" : ""}`}>
      {initials}
    </span>
  );
}

/** The square with an icon. Each kind of figure keeps its own color, so they tell themselves apart. */
export function Tile({ icon, tone = "sky", small }: { icon: IconName; tone?: Tone; small?: boolean }) {
  return (
    <span className={`tile tone-${tone} ${small ? "tile--sm" : ""}`}>
      <Icon name={icon} size={small ? 16 : 20} />
    </span>
  );
}

/** A state told with a small dot and its words (never only a color). */
export function StatusDot({ tone, children }: { tone: "green" | "amber" | "red" | "neutral"; children: ReactNode }) {
  const dot = { green: "bg-green", amber: "bg-amber", red: "bg-red", neutral: "bg-ink-3" }[tone];
  return (
    <span className="inline-flex items-center gap-2">
      <span aria-hidden="true" className={`h-2 w-2 shrink-0 rounded-pill ${dot}`} />
      {children}
    </span>
  );
}
