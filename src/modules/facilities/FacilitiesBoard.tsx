import { Findings } from "../../components/Findings";
import { Bar, Figures, Inset, Tag, TextButton, Tile } from "../../components/ui";
import { es } from "../../i18n/es-MX";
import { StateBar } from "./StateBar";
import type { FacilityBoard, KindTotal } from "./types";

const f = es.facilities;
const b = f.board;

type View = "spaces" | "building" | "equipment" | "board";
/** Where each missing piece is filled in. */
const WHERE: Record<string, View> = { built_m2: "building", floors: "building", tenure: "building", water: "building", safety: "building", spaces: "spaces", unchecked: "spaces" };

/** One kind of space: how many and how they are, in the colors of the states. */
function KindRow({ k }: { k: KindTotal }) {
  return (
    <li className="flex flex-col gap-2">
      <div className="flex items-baseline justify-between gap-3">
        <span className="min-w-0 text-ui font-semibold">{f.spaceKinds[k.kind] ?? k.kind}</span>
        <span className="tabular shrink-0 text-ui font-bold">{k.count}</span>
      </div>
      <StateBar s={k} count={k.count} />
    </li>
  );
}

/** What is still not said, as a short list that takes the person to the place where it is filled in. */
function Missing({ list, onGo }: { list: string[]; onGo: (v: View) => void }) {
  return (
    <Inset className="flex flex-col gap-4">
      <div className="flex items-start gap-3">
        <Tile small icon="alert" tone="amber" />
        <div className="min-w-0 flex-1">
          <h3 className="text-heading font-bold">{f.missing.title}</h3>
          <p className="max-w-[80ch] text-small text-ink-2">{f.missing.help}</p>
        </div>
      </div>
      <ul className="grid gap-2 sm:grid-cols-2">
        {list.map((m) => (
          <li key={m} className="flex items-center justify-between gap-3 rounded-inset bg-card px-4 py-3">
            <span className="min-w-0 text-ui font-semibold">{f.missing[m] ?? m}</span>
            <TextButton onClick={() => onGo(WHERE[m] ?? "building")}>{f.missing.go}</TextButton>
          </li>
        ))}
      </ul>
    </Inset>
  );
}

/**
 * The board of the facilities (ADR-030): four figures, what is missing, what the data say crossed with the people
 * served, how everything is, kind by kind, and what fails most. The ratios are shown, not graded against the norm yet.
 */
export function FacilitiesBoard({ board, onGo }: { board: FacilityBoard; onGo: (v: View) => void }) {
  const i = board.indicators;
  const bad = i.spaces_states.poor + i.spaces_states.unusable;
  const figures = [
    { label: b.perPerson, value: board.built_m2_per_person !== null ? String(board.built_m2_per_person) : "—", sub: i.built_m2 !== null ? f.building.m2(i.built_m2) : undefined },
    { label: b.perBathroom, value: board.people_per_bathroom !== null ? board.people_per_bathroom.toLocaleString("es-MX") : "—", sub: i.bathrooms > 0 ? `${i.bathrooms} ${f.spaceKinds.bathroom!.toLowerCase()}` : undefined },
    { label: b.beds, value: i.beds !== null ? String(i.beds) : "—", sub: b.bedsSub(board.served, board.capacity) },
    { label: b.bad, value: String(bad), sub: f.spaces.total(i.spaces), fill: i.spaces > 0 ? (bad / i.spaces) * 100 : undefined, tone: "red" as const },
  ];
  const worst = Math.max(...i.problems.map((x) => x.count), 1);
  return (
    <div className="flex flex-col gap-4">
      <Figures items={figures} />
      {i.missing.length > 0 && <Missing list={i.missing} onGo={onGo} />}
      <Findings insights={board.insights} describe={(code, values, items) => f.insights[code]?.(values, items) ?? code} />
      <h3 className="pt-2 text-heading font-bold">{b.charts}</h3>
      <div className="grid items-start gap-4 lg:grid-cols-2">
        <Inset className="flex flex-col gap-3">
          <div className="flex flex-wrap items-baseline justify-between gap-x-4">
            <h3 className="text-heading font-bold">{b.spacesState}</h3>
            <span className="text-small text-ink-3">{f.spaces.total(i.spaces)}</span>
          </div>
          {i.spaces > 0 ? <StateBar s={i.spaces_states} count={i.spaces} thick /> : <p className="text-ui text-ink-3">{b.noneChecked}</p>}
        </Inset>
        <Inset className="flex flex-col gap-3">
          <div className="flex flex-wrap items-baseline justify-between gap-x-4">
            <h3 className="text-heading font-bold">{b.equipmentState}</h3>
            <span className="text-small text-ink-3">{f.equipment.total(i.equipment)}</span>
          </div>
          {i.equipment > 0 ? <StateBar s={i.equipment_states} count={i.equipment} thick /> : <p className="text-ui text-ink-3">{b.noneChecked}</p>}
        </Inset>
        {i.by_kind.length > 0 && (
          <Inset className="flex flex-col gap-4">
            <h3 className="text-heading font-bold">{b.byKind}</h3>
            <ul className="flex flex-col gap-5">
              {i.by_kind.map((k) => (
                <KindRow key={k.kind} k={k} />
              ))}
            </ul>
          </Inset>
        )}
        {i.problems.length > 0 && (
          <Inset className="flex flex-col gap-4">
            <h3 className="text-heading font-bold">{b.problems}</h3>
            <ul className="flex flex-col gap-3">
              {i.problems.map((p) => (
                <li key={p.code} className="flex flex-col gap-1.5">
                  <div className="flex items-baseline justify-between gap-3">
                    <span className="min-w-0 text-ui font-semibold">{f.problems[p.code] ?? p.code}</span>
                    <Tag variant="line">{b.problemCount(p.count)}</Tag>
                  </div>
                  <Bar percent={(p.count / worst) * 100} label={f.problems[p.code] ?? p.code} tone="amber" />
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
