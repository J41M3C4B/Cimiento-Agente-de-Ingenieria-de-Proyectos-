const C = 200;
const point = (deg: number, r: number) => {
  const a = (deg * Math.PI) / 180;
  return { x: +(C + r * Math.cos(a)).toFixed(1), y: +(C + r * Math.sin(a)).toFixed(1) };
};

type Ring = { r: number; angles: number[]; dot: number; full: boolean[] };
// the circles of people around the center: the closer to the middle, the closer the institution
const RINGS: Ring[] = [
  { r: 62, angles: [20, 140, 260], dot: 8, full: [true, true, true] },
  { r: 112, angles: [60, 130, 205, 285, 340], dot: 7, full: [true, false, true, true, false] },
  { r: 160, angles: [15, 70, 110, 165, 225, 265, 320], dot: 6, full: [false, true, false, true, false, true, false] },
];

const gap = (a: number, b: number) => Math.abs(((a - b + 540) % 360) - 180);
const nearest = (angle: number, from: number[]) => from.reduce((best, x) => (gap(x, angle) < gap(best, angle) ? x : best));

/**
 * The picture on the blue panel: a constellation of people linked around one point. It is flat (white lines and
 * dots on the blue, no gradients or glow) and only decorates, so it is hidden from screen readers.
 */
export function BrandArt({ className = "" }: { className?: string }) {
  const links = RINGS.flatMap((ring, i) => {
    const inner = i === 0 ? null : RINGS[i - 1]!;
    return ring.angles.map((angle, n) => {
      const from = point(angle, ring.r);
      const to = inner ? point(nearest(angle, inner.angles), inner.r) : { x: C, y: C };
      return <line key={`${i}-${n}`} x1={from.x} y1={from.y} x2={to.x} y2={to.y} />;
    });
  });

  return (
    <svg viewBox="0 0 400 400" fill="none" aria-hidden="true" focusable="false" className={className}>
      <g stroke="currentColor" strokeOpacity="0.28" strokeWidth="1.4">
        {RINGS.map((ring) => (
          <circle key={ring.r} cx={C} cy={C} r={ring.r} />
        ))}
        <circle cx={C} cy={C} r="196" strokeDasharray="2 7" strokeLinecap="round" />
      </g>
      <g stroke="currentColor" strokeOpacity="0.5" strokeWidth="1.6" strokeLinecap="round">
        {links}
      </g>
      {RINGS.map((ring, i) =>
        ring.angles.map((angle, n) => {
          const p = point(angle, ring.r);
          return <circle key={`${i}-${n}`} cx={p.x} cy={p.y} r={ring.dot} fill="currentColor" fillOpacity={ring.full[n] ? 1 : 0.55} />;
        }),
      )}
      <circle cx={C} cy={C} r="24" fill="currentColor" />
      <path className="fill-brand" d="M200 185l3.8 11.2 11.2 3.8-11.2 3.8-3.8 11.2-3.8-11.2-11.2-3.8 11.2-3.8z" />
    </svg>
  );
}
