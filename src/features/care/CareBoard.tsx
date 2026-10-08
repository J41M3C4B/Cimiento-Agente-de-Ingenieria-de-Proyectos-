import { Findings } from "../../components/Findings";
import { Alert, Bar, Figures, Inset, Tag } from "../../components/ui";
import type { Tone } from "../../components/ui";
import { es } from "../../i18n/es-MX";
import type { Board, CareOverview, Count } from "./types";

const c = es.care;
const b = c.board;
const peso = (n: number) => `$${n.toLocaleString("es-MX")}`;

/** One block of the board: a small title and what it shows. */
function Panel({ title, help, children }: { title: string; help?: string; children: React.ReactNode }) {
  return (
    <Inset className="flex flex-col gap-4">
      <div>
        <h3 className="text-heading font-bold">{title}</h3>
        {help && <p className="text-small text-ink-2">{help}</p>}
      </div>
      {children}
    </Inset>
  );
}

/** Counts as ranked bars: the name, how many and what share of the people, with a bar below. */
function RankBars({ title, list, labels, total, tone, natural }: { title: string; list: Count[]; labels: Record<string, string>; total: number; tone: Tone; natural?: boolean }) {
  if (list.length === 0) return null;
  const order = Object.keys(labels);
  const rows = [...list].sort((x, y) => (natural ? order.indexOf(x.code) - order.indexOf(y.code) : y.count - x.count));
  const top = Math.max(...rows.map((x) => x.count), 1);
  return (
    <Panel title={title}>
      <ul className="flex flex-col gap-3">
        {rows.map((x) => {
          const label = labels[x.code] ?? x.code;
          const share = total > 0 ? Math.round((x.count / total) * 100) : 0;
          return (
            <li key={x.code} className="flex flex-col gap-1.5">
              <div className="flex items-baseline justify-between gap-3">
                <span className="min-w-0 text-ui font-semibold">{label}</span>
                <span className="tabular shrink-0 text-ui">
                  <b className="font-bold">{x.count}</b>
                  <span className="ml-2 text-small text-ink-3">{b.share(share)}</span>
                </span>
              </div>
              <Bar percent={(x.count / top) * 100} label={`${label}: ${b.people(x.count)}`} tone={tone} />
            </li>
          );
        })}
      </ul>
    </Panel>
  );
}

/** The age pyramid: women on the left, men on the right, one row per age band. */
function Pyramid({ rows, withoutAge, averageAge }: { rows: [string, string, number][]; withoutAge: number; averageAge: number | null }) {
  const bands = Object.keys(c.bands).filter((band) => rows.some(([x]) => x === band));
  const n = (band: string, sex: string) => rows.filter(([x, s]) => x === band && s === sex).reduce((sum, [, , k]) => sum + k, 0);
  const women = rows.filter(([, s]) => s === "female").reduce((sum, [, , k]) => sum + k, 0);
  const men = rows.filter(([, s]) => s === "male").reduce((sum, [, , k]) => sum + k, 0);
  const others = rows.filter(([, s]) => s !== "female" && s !== "male").reduce((sum, [, , k]) => sum + k, 0);
  const top = Math.max(...bands.flatMap((band) => [n(band, "female"), n(band, "male")]), 1);
  const side = (count: number, tone: Tone, align: "left" | "right") => {
    const number = <span className="tabular w-5 shrink-0 text-small font-bold">{count > 0 ? count : ""}</span>;
    const bar = (
      <div style={{ width: `${(count / top) * 84}%` }} aria-hidden="true">
        {count > 0 && <Bar percent={100} tone={tone} />}
      </div>
    );
    return align === "left" ? (
      <div className="flex items-center justify-end gap-2">
        {number}
        {bar}
      </div>
    ) : (
      <div className="flex items-center gap-2">
        {bar}
        {number}
      </div>
    );
  };
  return (
    <Panel title={b.pyramid} help={b.pyramidHelp}>
      {bands.length === 0 ? (
        <p className="text-ui text-ink-3">{b.noChart}</p>
      ) : (
        <>
          <div className="flex flex-wrap gap-2">
            <Tag tone="rose">{`${c.sexes.female} · ${women}`}</Tag>
            <Tag tone="sky">{`${c.sexes.male} · ${men}`}</Tag>
            {averageAge !== null && <Tag variant="line">{b.averageAge(Math.round(averageAge))}</Tag>}
          </div>
          <div role="img" aria-label={`${b.pyramid}: ${c.sexes.female} ${women}, ${c.sexes.male} ${men}`} className="flex flex-col gap-2">
            {bands.map((band) => (
              <div key={band} className="grid grid-cols-[minmax(0,1fr)_6.5rem_minmax(0,1fr)] items-center gap-2">
                {side(n(band, "female"), "rose", "left")}
                <span className="text-center text-small font-semibold text-ink-2">{c.bands[band] ?? band}</span>
                {side(n(band, "male"), "sky", "right")}
              </div>
            ))}
          </div>
        </>
      )}
      {others > 0 && <p className="text-small text-ink-3">{b.pyramidOthers(others)}</p>}
      {withoutAge > 0 && <p className="text-small text-ink-3">{b.withoutAge(withoutAge)}</p>}
    </Panel>
  );
}

/**
 * The board (ADR-029): four figures, what the data say and how the people are made up. It is what shows the value of
 * capturing the records: small data turned into arguments for a project.
 */
export function CareBoard({ board, flavor }: { board: Board; flavor: CareOverview["flavor"] }) {
  const i = board.indicators;
  const figures = [
    { label: b.served, value: String(i.served), sub: board.occupancy_percent !== null ? b.occupancy(board.occupancy_percent) : undefined, fill: board.occupancy_percent ?? undefined, tone: "violet" as const },
    { label: c.tabs.waitlist, value: String(board.waiting), sub: board.free_seats !== null ? b.free(board.free_seats) : undefined, tone: "amber" as const },
    { label: b.cost, value: board.cost_per_person_monthly !== null ? peso(board.cost_per_person_monthly) : "—", sub: i.average_fee !== null ? `${b.averageFee}: ${peso(i.average_fee)}` : undefined },
    { label: b.stay, value: i.average_years !== null ? c.years(Math.round(i.average_years)) : "—", sub: b.thisYear(i.admitted_this_year, i.discharged_this_year, i.deceased_this_year) },
  ];
  // two columns of panels, filled one by one, so the blocks do not leave holes between them
  const ranks: { key: string; title: string; list: Count[]; labels: Record<string, string>; tone: Tone; natural?: boolean; show: boolean }[] = [
    { key: "dependency", title: b.sections.dependency, list: i.dependency, labels: c.dependency, tone: "violet", natural: true, show: true },
    { key: "mobility", title: b.sections.mobility, list: i.mobility, labels: c.mobility, tone: "sky", natural: true, show: true },
    { key: "disabilities", title: b.sections.disabilities, list: i.disabilities, labels: c.disabilities, tone: "teal", show: true },
    { key: "chronic", title: b.sections.chronic, list: i.chronic, labels: c.chronic, tone: "rose", show: flavor !== "children_home" },
    { key: "reasons", title: b.sections.admission_reasons, list: i.admission_reasons, labels: c.admissionReasons, tone: "amber", show: true },
    { key: "programs", title: b.sections.programs, list: i.programs, labels: c.programs, tone: "green", show: true },
    { key: "legal", title: b.sections.legal, list: i.legal, labels: c.legal, tone: "cyan", show: flavor === "children_home" },
    { key: "groups", title: b.sections.groups, list: i.by_group, labels: {}, tone: "rose", show: i.by_group.length > 1 },
  ];
  const panels = [
    { key: "pyramid", node: <Pyramid key="pyramid" rows={i.pyramid} withoutAge={i.without_age} averageAge={i.average_age} /> },
    ...ranks.filter((x) => x.show && x.list.length > 0).map((x) => ({ key: x.key, node: <RankBars key={x.key} title={x.title} list={x.list} labels={x.labels} total={i.served} tone={x.tone} natural={x.natural} /> })),
  ];
  return (
    <div className="flex flex-col gap-4">
      <Figures items={figures} />
      <Findings insights={board.insights} describe={(code, values, items) => c.insights[code]?.(values, items) ?? code} />
      <h3 className="pt-2 text-heading font-bold">{b.charts}</h3>
      <div className="grid items-start gap-4 lg:grid-cols-2">
        {[0, 1].map((col) => (
          <div key={col} className="flex flex-col gap-4">
            {panels.filter((_, n) => n % 2 === col).map((x) => x.node)}
          </div>
        ))}
      </div>
      {i.incomplete > 0 && (
        <Alert tone="info">{b.incomplete(i.incomplete)}</Alert>
      )}
    </div>
  );
}
