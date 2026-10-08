import { Bar, Figures, Inset, Tag, Tile } from "../../components/ui";
import { es } from "../../i18n/es-MX";
import { STATE_KEYS, STATE_TONE } from "./GroupDialog";
import type { FacilityBoard, KindTotal } from "./types";

const f = es.facilities;
const b = f.board;
const cb = es.care.board;

/** One kind of space as a bar split by state: green the good ones, amber the fair, red the bad, grey the unchecked. */
function KindRow({ k }: { k: KindTotal }) {
  const parts = [...STATE_KEYS.map((s) => [s, k[s]] as const), ["unchecked", k.unchecked] as const].filter(([, n]) => n > 0);
  return (
    <li className="flex flex-col gap-1.5">
      <div className="flex items-baseline justify-between gap-3">
        <span className="min-w-0 text-ui font-semibold">{f.spaceKinds[k.kind] ?? k.kind}</span>
        <span className="tabular shrink-0 text-ui font-bold">{k.count}</span>
      </div>
      <div className="flex gap-1" role="img" aria-label={parts.map(([s, n]) => `${n} ${s === "unchecked" ? f.unchecked(n) : f.states[s]}`).join(", ")}>
        {parts.map(([s, n]) => (
          <div key={s} style={{ width: `${(n / k.count) * 100}%` }}>
            <Bar percent={100} tone={s === "unchecked" ? "neutral" : STATE_TONE[s]} />
          </div>
        ))}
      </div>
      <div className="flex flex-wrap gap-2">
        {parts.map(([s, n]) => (
          <span key={s} className="text-small text-ink-3">
            {s === "unchecked" ? f.unchecked(n) : `${n} ${f.states[s].toLowerCase()}`}
          </span>
        ))}
      </div>
    </li>
  );
}

/**
 * The board of the facilities (ADR-030): four figures, what the data say crossed with the people served, how each
 * kind of space is and what fails most. The ratios are shown, not graded against the norm yet.
 */
export function FacilitiesBoard({ board }: { board: FacilityBoard }) {
  const i = board.indicators;
  const bad = i.spaces_states.poor + i.spaces_states.unusable;
  const figures = [
    { label: b.perPerson, value: board.built_m2_per_person !== null ? String(board.built_m2_per_person) : "—", sub: i.built_m2 !== null ? f.building.m2(i.built_m2) : undefined },
    { label: b.perBathroom, value: board.people_per_bathroom !== null ? board.people_per_bathroom.toLocaleString("es-MX") : "—", sub: i.bathrooms > 0 ? `${i.bathrooms} ${f.spaceKinds.bathroom.toLowerCase()}` : undefined },
    { label: b.beds, value: i.beds !== null ? String(i.beds) : "—", sub: b.bedsSub(board.served, board.capacity) },
    { label: b.bad, value: String(bad), sub: f.spaces.total(i.spaces), fill: i.spaces > 0 ? (bad / i.spaces) * 100 : undefined, tone: "red" as const },
  ];
  return (
    <div className="flex flex-col gap-4">
      <Figures items={figures} />
      {i.missing.length > 0 && (
        <Inset className="flex flex-col gap-2">
          <h3 className="text-heading font-bold">{f.missing.title}</h3>
          <div className="flex flex-wrap gap-2">
            {i.missing.map((m) => (
              <Tag key={m} tone="amber" variant="soft">
                {f.missing[m] ?? m}
              </Tag>
            ))}
          </div>
        </Inset>
      )}
      <Inset className="flex flex-col gap-4">
        <div>
          <h3 className="text-heading font-bold">{cb.findings}</h3>
          <p className="max-w-[80ch] text-small text-ink-2">{cb.findingsHelp}</p>
        </div>
        {board.insights.length === 0 ? (
          <p className="text-ui text-ink-3">{cb.noFindings}</p>
        ) : (
          <ul className="grid grid-cols-1 gap-3 lg:grid-cols-2">
            {board.insights.map((x) => (
              <li key={x.code} className="flex min-w-0 gap-3 rounded-inset bg-card p-4">
                <Tile small icon={x.for_ai ? "sparkles" : "lock"} tone={x.for_ai ? "violet" : "neutral"} />
                <div className="flex min-w-0 flex-1 flex-col items-start gap-2">
                  <p className="text-ui">{f.insights[x.code]?.(x.values, x.items) ?? x.code}</p>
                  <Tag tone={x.for_ai ? "violet" : "neutral"} variant="soft">
                    {x.for_ai ? cb.forAi : cb.internal}
                  </Tag>
                </div>
              </li>
            ))}
          </ul>
        )}
      </Inset>
      <div className="grid items-start gap-4 lg:grid-cols-2">
        {i.by_kind.length > 0 && (
          <Inset className="flex flex-col gap-4">
            <h3 className="text-heading font-bold">{b.byKind}</h3>
            <ul className="flex flex-col gap-4">
              {i.by_kind.map((k) => (
                <KindRow key={k.kind} k={k} />
              ))}
            </ul>
          </Inset>
        )}
        {i.problems.length > 0 && (
          <Inset className="flex flex-col gap-3">
            <h3 className="text-heading font-bold">{b.problems}</h3>
            <ul className="flex flex-col gap-3">
              {i.problems.map((p) => (
                <li key={p.code} className="flex flex-col gap-1.5">
                  <div className="flex items-baseline justify-between gap-3">
                    <span className="text-ui font-semibold">{f.problems[p.code] ?? p.code}</span>
                    <span className="text-small text-ink-3">{b.problemCount(p.count)}</span>
                  </div>
                  <Bar percent={(p.count / Math.max(...i.problems.map((x) => x.count), 1)) * 100} tone="amber" />
                </li>
              ))}
            </ul>
          </Inset>
        )}
      </div>
      <p className="text-small text-ink-3">{b.noNorm}</p>
    </div>
  );
}
