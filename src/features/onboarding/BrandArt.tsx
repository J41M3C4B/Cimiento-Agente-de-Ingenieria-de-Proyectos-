const C = 200;
const point = (deg: number, r: number) => {
  const a = (deg * Math.PI) / 180;
  return { x: +(C + r * Math.cos(a)).toFixed(1), y: +(C + r * Math.sin(a)).toFixed(1) };
};

type Ring = { r: number; angles: number[]; dot: number; fill: string[] };
// the circles of people around the center: the closer to the middle, the closer the institution
const RINGS: Ring[] = [
  { r: 62, angles: [20, 140, 260], dot: 7, fill: ["fill-cyan", "fill-teal", "fill-cyan"] },
  { r: 112, angles: [60, 130, 205, 285, 340], dot: 6, fill: ["fill-sky", "fill-violet", "fill-cyan", "fill-teal", "fill-sky"] },
  { r: 160, angles: [15, 70, 110, 165, 225, 265, 320], dot: 5, fill: ["fill-teal", "fill-sky", "fill-cyan", "fill-violet", "fill-sky", "fill-teal", "fill-cyan"] },
];

const gap = (a: number, b: number) => Math.abs(((a - b + 540) % 360) - 180);
const nearest = (angle: number, from: number[]) => from.reduce((best, x) => (gap(x, angle) < gap(best, angle) ? x : best));

/**
 * The picture on the blue panel: a constellation of people linked around one bright point. It only decorates, so
 * it is hidden from screen readers. It turns very slowly and the middle breathes; both stop with reduced motion.
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
      <defs>
        <radialGradient id="onb-glow" cx="50%" cy="50%" r="50%">
          <stop offset="0" stopColor="currentColor" stopOpacity="0.55" />
          <stop offset="1" stopColor="currentColor" stopOpacity="0" />
        </radialGradient>
      </defs>
      <g className="onb-art-spin">
        <g stroke="currentColor" strokeOpacity="0.16" strokeWidth="1.2">
          {RINGS.map((ring) => (
            <circle key={ring.r} cx={C} cy={C} r={ring.r} />
          ))}
          <circle cx={C} cy={C} r="196" strokeDasharray="2 7" strokeLinecap="round" />
        </g>
        <g stroke="currentColor" strokeOpacity="0.34" strokeWidth="1.4" strokeLinecap="round">
          {links}
        </g>
        {RINGS.map((ring, i) =>
          ring.angles.map((angle, n) => {
            const p = point(angle, ring.r);
            return (
              <g key={`${i}-${n}`}>
                <circle cx={p.x} cy={p.y} r={ring.dot * 2.4} fill="currentColor" fillOpacity="0.08" />
                <circle cx={p.x} cy={p.y} r={ring.dot} className={ring.fill[n]} />
              </g>
            );
          }),
        )}
      </g>
      <g className="onb-art-core">
        <circle cx={C} cy={C} r="58" fill="url(#onb-glow)" />
        <circle cx={C} cy={C} r="22" fill="currentColor" />
        <path className="fill-brand" d="M200 187l3.4 9.6 9.6 3.4-9.6 3.4-3.4 9.6-3.4-9.6-9.6-3.4 9.6-3.4z" />
      </g>
    </svg>
  );
}
