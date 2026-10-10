import type { ReactNode } from "react";
import { Icon } from "../icons";
import type { IconName } from "../icons";

/** How many squares a waffle draws at most: a bigger capacity is shown in parts of one hundred. */
const WAFFLE_MAX = 100;

/**
 * The shape of a waffle: how many squares, how many are filled, and how they are laid out (the rows and columns that come
 * closest to the wanted proportion, wide / tall, without leaving many squares over). Display only: the figures come from Rust.
 */
export function waffleShape(places: number, taken: number, aspect = 1.7) {
  const cells = Math.min(Math.max(Math.round(places), 0), WAFFLE_MAX);
  const per = places > WAFFLE_MAX ? places / WAFFLE_MAX : 1;
  const raw = Math.min(Math.max(taken, 0) / per, cells);
  // whoever is there always shows: a single person is at least one square
  const filled = taken > 0 ? Math.min(cells, Math.max(1, Math.round(raw))) : 0;
  let rows = 1;
  let best = Infinity;
  for (let r = 1; r <= 10; r++) {
    const c = Math.ceil(cells / r);
    const score = Math.abs(c / r - aspect) + (c * r - cells) * 0.15;
    if (score < best) {
      best = score;
      rows = r;
    }
  }
  return { cells, filled, rows, cols: Math.ceil(cells / rows) };
}

/**
 * A waffle: one square per place, filled in the blue of the brand for the places that are taken and in gray for the free
 * ones. They fill from the bottom-left, column by column. It takes the height of its parent and the width that follows.
 */
export function Waffle({ places, taken, label, aspect, fill }: { places: number; taken: number; label: string; aspect?: number; /** takes the whole width and height of its parent (the squares stretch) instead of keeping the shape of a square grid */ fill?: boolean }) {
  const { cells, filled, rows, cols } = waffleShape(places, taken, aspect);
  if (cells === 0) return null;
  const squares: ReactNode[] = [];
  for (let c = 0; c < cols; c++) {
    // each column is drawn from its top to its bottom, and fills from its bottom
    for (let r = rows - 1; r >= 0; r--) {
      const k = c * rows + r;
      squares.push(<i key={k} className={k >= cells ? "waffle-cell waffle-cell--none" : k < filled ? "waffle-cell waffle-cell--on" : "waffle-cell"} />);
    }
  }
  return (
    <div
      role="img"
      aria-label={label}
      className={fill ? "waffle waffle--fill" : "waffle"}
      style={{ gridTemplateColumns: `repeat(${cols}, 1fr)`, gridTemplateRows: `repeat(${rows}, 1fr)`, aspectRatio: fill ? undefined : `${cols} / ${rows}` }}
    >
      {squares}
    </div>
  );
}

/**
 * How the income of a year splits: what was spent and what is left, as parts (0..1) of the bigger of the two. Display
 * only: Rust added the sums. `null` when there is nothing to show.
 */
export function gaugeShares(income: number, expenses: number): { spent: number; left: number } | null {
  const scale = Math.max(income, expenses);
  if (!(scale > 0) || income < 0 || expenses < 0) return null;
  return { spent: expenses / scale, left: Math.max(income - expenses, 0) / scale };
}

const R = 186; // the radius of the arc
const STROKE = 26;
const CX = 200;
const CY = 200;
const LENGTH = Math.PI * R;
const CAP = STROKE / 2; // a round end sticks out half the thickness
const GAP = 4; // the air between two colors
const MIN_PIECE = CAP * 2 + 2; // the least a color can show: one full round end
const ARC = `M${CX - R} ${CY}A${R} ${R} 0 0 1 ${CX + R} ${CY}`;

/** Where to draw a piece of the arc so that, with its round ends, it covers exactly from `from` to `to`. */
function piece(from: number, to: number, gapBefore: boolean, gapAfter: boolean) {
  let start = from + CAP + (gapBefore ? GAP / 2 : 0);
  let end = to - CAP - (gapAfter ? GAP / 2 : 0);
  // too short for two round ends: a single dot in the middle
  if (end < start) start = end = (from + to) / 2;
  return { strokeDasharray: `${end - start} ${LENGTH * 2}`, strokeDashoffset: -start };
}

export type GaugePart = "income" | "spent" | "left";

/**
 * A half donut: the whole arc is the income (gray), the spending takes its part in dark blue and what is left follows in
 * the blue of the brand, each with round ends. With no figures only the gray arc shows. When the spending is more than
 * the income the whole arc is spending. Pointing at a color (`onActive`) puts the others in the background; the hollow
 * of the donut (`children`) says what is being pointed at.
 */
export function Gauge({
  spent,
  left,
  label,
  active,
  onActive,
  children,
}: {
  spent: number;
  left: number;
  label: string;
  active?: GaugePart | null;
  onActive?: (part: GaugePart | null) => void;
  /** what the hollow of the donut says: a big figure and its words */
  children?: ReactNode;
}) {
  let a = spent * LENGTH;
  let b = left * LENGTH;
  // a color that is only a sliver still shows as a whole round end, taking that bit from the other one
  if (b > 0 && b < MIN_PIECE) {
    b = MIN_PIECE;
    a = Math.min(a, LENGTH - b);
  }
  if (a > 0 && a < MIN_PIECE) {
    a = MIN_PIECE;
    b = Math.min(b, LENGTH - a);
  }
  const both = a > 0 && b > 0;
  const arc = (part: GaugePart, extra: object) => (
    <path
      d={ARC}
      className={`gauge-arc gauge-arc--${part} ${active === part ? "is-active" : ""}`}
      onMouseEnter={() => onActive?.(part)}
      onMouseLeave={() => onActive?.(null)}
      {...extra}
    />
  );
  return (
    <div className="gauge-wrap">
      <svg viewBox={`0 0 ${CX * 2} ${CY + CAP}`} role="group" aria-label={label} className="gauge" data-active={active ?? undefined}>
        {arc("income", piece(0, LENGTH, false, false))}
        {a > 0 && arc("spent", piece(0, a, false, both))}
        {b > 0 && arc("left", piece(a, a + b, both, false))}
      </svg>
      {children && <div className="gauge-center">{children}</div>}
    </div>
  );
}

/** A dot of one of the three colors of the charts (gray, dark blue, brand blue), for the keys. */
export function ChartDot({ tone }: { tone: "off" | "ink" | "brand" }) {
  return <i aria-hidden="true" className={`chart-dot chart-dot--${tone}`} />;
}

/**
 * A figure of the whole institution (Inicio): its own white tray of a fixed size, with the clean icon and the title in
 * ink; it never grows with what it holds. The whole tray opens the module the figure comes from.
 */
export function FigureCard({ icon, title, onOpen, children }: { icon: IconName; title: string; onOpen: () => void; children: ReactNode }) {
  return (
    <button type="button" onClick={onOpen} className="fig-card">
      <span className="fig-head">
        <Icon name={icon} size={20} />
        {title}
      </span>
      <span className="fig-body">{children}</span>
    </button>
  );
}
