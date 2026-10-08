import { Inset, Tag, Tile } from "./ui";
import { es } from "../i18n/es-MX";

const b = es.care.board;

/** What the data say, each finding with a mark that tells whether the automatic help also knows it or it stays with the person. */
export function Findings({ insights, describe }: { insights: { code: string; values: Record<string, number>; items: string[]; for_ai: boolean }[]; describe: (code: string, values: Record<string, number>, items: string[]) => string }) {
  return (
    <Inset className="flex flex-col gap-4">
      <div>
        <h3 className="text-heading font-bold">{b.findings}</h3>
        <p className="max-w-[80ch] text-small text-ink-2">{b.findingsHelp}</p>
      </div>
      {insights.length === 0 ? (
        <p className="text-ui text-ink-3">{b.noFindings}</p>
      ) : (
        <ul className="grid grid-cols-1 gap-3 lg:grid-cols-2">
          {insights.map((x) => (
            <li key={x.code} className="flex min-w-0 gap-3 rounded-inset bg-card p-4">
              <Tile small icon={x.for_ai ? "sparkles" : "lock"} tone={x.for_ai ? "violet" : "neutral"} />
              <div className="flex min-w-0 flex-1 flex-col items-start gap-2">
                <p className="text-ui">{describe(x.code, x.values, x.items)}</p>
                <Tag tone={x.for_ai ? "violet" : "neutral"} variant="soft">
                  {x.for_ai ? b.forAi : b.internal}
                </Tag>
              </div>
            </li>
          ))}
        </ul>
      )}
    </Inset>
  );
}
