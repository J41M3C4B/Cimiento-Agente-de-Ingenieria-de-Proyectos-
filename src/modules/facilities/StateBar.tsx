import { Bar } from "../../components/ui";
import type { Tone } from "../../components/ui";
import { es } from "../../i18n/es-MX";
import type { States } from "./types";

const f = es.facilities;
export const STATE_KEYS = ["good", "fair", "poor", "unusable"] as const;
/** One color per state: green, amber, red, and black for what is out of service; grey for what is not checked yet. */
export const STATE_TONE: Record<string, Tone> = { good: "green", fair: "amber", poor: "red", unusable: "ink", unchecked: "ink" };
/** What is not checked is the same grey ink, faded, so it shows on any surface. */
const FADE = (key: string) => (key === "unchecked" ? "opacity-30" : "");

type Part = { key: string; n: number; text: string };

/** The states that have something, with their words: «4 bien», «1 regular», «2 sin revisar». */
export function stateParts(s: States, count: number): Part[] {
  const rest = Math.max(0, count - (s.good + s.fair + s.poor + s.unusable));
  const parts: Part[] = STATE_KEYS.filter((k) => s[k] > 0).map((k) => ({ key: k, n: s[k], text: `${s[k]} ${f.states[k]!.toLowerCase()}` }));
  if (rest > 0) parts.push({ key: "unchecked", n: rest, text: f.unchecked(rest) });
  return parts;
}

/** A small round mark of the color of a state, to put before its words. */
export function Swatch({ tone, faded }: { tone: Tone; faded?: boolean }) {
  return (
    <span aria-hidden="true" className={`inline-block w-2.5 shrink-0 ${faded ? "opacity-30" : ""}`}>
      <Bar percent={100} tone={tone} />
    </span>
  );
}

/** The words under a bar: each state with its mark. */
export function StateLegend({ parts, className = "" }: { parts: Part[]; className?: string }) {
  return (
    <ul className={`flex flex-wrap gap-x-4 gap-y-1 text-small text-ink-2 ${className}`}>
      {parts.map((p) => (
        <li key={p.key} className="flex items-center gap-1.5">
          <Swatch tone={STATE_TONE[p.key]!} faded={p.key === "unchecked"} />
          {p.text}
        </li>
      ))}
    </ul>
  );
}

/**
 * How many are in each state as one bar cut in parts, in the colors of the states; what is not checked yet is grey.
 * It reads in a glance whether a group is fine or not, and the words under it say how many.
 */
export function StateBar({ s, count, legend = true, thick }: { s: States; count: number; legend?: boolean; thick?: boolean }) {
  const parts = stateParts(s, count);
  const total = parts.reduce((n, p) => n + p.n, 0);
  if (total === 0) return <span className="text-ink-3">—</span>;
  return (
    <div className="flex min-w-0 flex-col gap-1.5">
      <div role="img" aria-label={parts.map((p) => p.text).join(", ")} className="flex gap-1">
        {parts.map((p) => (
          <div key={p.key} style={{ width: `${(p.n / total) * 100}%` }} className={`${thick ? "[&_.bar]:!h-3" : ""} ${FADE(p.key)}`}>
            <Bar percent={100} tone={STATE_TONE[p.key]} />
          </div>
        ))}
      </div>
      {legend && <StateLegend parts={parts} />}
    </div>
  );
}
