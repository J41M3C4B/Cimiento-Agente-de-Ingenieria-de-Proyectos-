import type { ElementType, ReactNode } from "react";
import { es } from "../../i18n/es-MX";
import type { Tone } from "./Tag";

/** A tray: the white block where one thing is done (docs/13 §2). */
export function Card({ children, className = "", small, flush, as: As = "div" }: { children: ReactNode; className?: string; small?: boolean; flush?: boolean; as?: ElementType }) {
  return <As className={`card ${small ? "card--pad-sm" : ""} ${flush ? "card--flush" : ""} ${className}`}>{children}</As>;
}

/** What sits inside a tray (a bubble, a figure, a note): a softer surface. */
export function Inset({ children, className = "" }: { children: ReactNode; className?: string }) {
  return <div className={`inset ${className}`}>{children}</div>;
}

/** A tray with a title: where one thing is done. */
export function Section({ title, help, children }: { title: string; help?: string; children: ReactNode }) {
  return (
    <Card as="section">
      <h2 className="text-heading font-bold">{title}</h2>
      {help && <p className="mt-1 text-ui text-ink-2">{help}</p>}
      <div className="mt-4 space-y-4">{children}</div>
    </Card>
  );
}

/**
 * A tray with a folder tab (docs/13 §6). The tab carries the title and, to its right, an optional capsule (`chip`);
 * the whole folder is of one color (`tone`), which the pieces inside can use as `tone-pc`.
 */
export function Folder({
  tone = "sky",
  title,
  chip,
  children,
  className = "",
  bodyClassName = "",
  as: As = "div",
}: {
  tone?: Tone;
  title: ReactNode;
  /** text of the title for the `title` attribute when it is cut short */
  titleText?: string;
  chip?: ReactNode;
  children: ReactNode;
  className?: string;
  bodyClassName?: string;
  as?: ElementType;
}) {
  return (
    <As className={`folder tone-${tone} ${className}`}>
      <div className="folder-top">
        <div className="folder-tab">
          <b className="folder-title">{title}</b>
          {chip}
        </div>
      </div>
      <div className={`folder-body ${bodyClassName}`}>{children}</div>
    </As>
  );
}

/**
 * One line of a page: its title on the left, what is known in the middle and its action on the right.
 * Lines are told apart by a hairline and space, never by boxes.
 */
export function FactRow({ title, note, action, children }: { title: string; note?: string; action?: ReactNode; children: ReactNode }) {
  return (
    <section className="grid gap-x-8 gap-y-3 border-t border-line py-5 first:border-t-0 first:pt-1 md:grid-cols-[200px_minmax(0,1fr)_auto]">
      <div>
        <h2 className="text-ui font-bold">{title}</h2>
        {note && <p className="mt-1 max-w-[190px] text-caption text-ink-3">{note}</p>}
      </div>
      <div className="min-w-0">{children}</div>
      <div className="md:text-right">{action}</div>
    </section>
  );
}

/** What is known, read-only: small label over the value. An empty value says so instead of leaving a hole. */
export function Facts({ items, columns = 3 }: { items: [string, ReactNode][]; columns?: 1 | 2 | 3 }) {
  return (
    <dl className={`grid gap-x-8 gap-y-4 ${columns === 3 ? "sm:grid-cols-3" : columns === 2 ? "sm:grid-cols-2" : ""}`}>
      {items.map(([label, value]) => (
        <div key={label} className="min-w-0">
          <dt className="text-caption text-ink-3">{label}</dt>
          <dd className="mt-0.5 break-words text-ui font-semibold">{value || <span className="font-medium text-ink-3">{es.profile.fields.optionNone}</span>}</dd>
        </div>
      ))}
    </dl>
  );
}

/** The figures a page adds up to: a row of insets, each with its label, its number and its detail. */
export function Figures({ items }: { items: { label: string; value: string; sub?: string; fill?: number; tone?: Tone }[] }) {
  return (
    <dl className="grid grid-cols-2 gap-3 lg:grid-cols-4">
      {items.map((s) => (
        <Inset key={s.label} className="flex flex-col gap-2">
          <dt className="text-ui font-semibold text-ink-2">{s.label}</dt>
          <dd className="flex flex-1 flex-col gap-2">
            <span className="tabular block text-hero font-normal tracking-tight">{s.value}</span>
            {s.fill !== undefined && (
              <span className={`bar tone-${s.tone ?? "ink"} mt-auto block !h-1.5`}>
                <i style={{ width: `${Math.max(0, Math.min(100, s.fill))}%` }} />
              </span>
            )}
            {s.sub && <span className="text-small text-ink-3">{s.sub}</span>}
          </dd>
        </Inset>
      ))}
    </dl>
  );
}
