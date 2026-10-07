import type { ReactNode } from "react";
import { Icon } from "../icons";
import type { Tone } from "./Tag";

type TabItem<T extends string> = { id: T; label: string; count?: number; alert?: boolean };

/**
 * Cut-out tabs over a gray tray (docs/13 §6): the chosen tab lifts in the color of the tray. The panel is the
 * `children` (use `TabPanel`); each tab names it with `aria-controls`.
 */
export function Dock<T extends string>({
  items,
  value,
  onChange,
  label,
  children,
}: {
  items: TabItem<T>[];
  value: T;
  onChange: (id: T) => void;
  label: string;
  children: ReactNode;
}) {
  const first = items[0]?.id === value;
  return (
    <div className={`dock ${first ? "" : "dock--detached"}`}>
      <div role="tablist" aria-label={label} className="dock-tabs max-w-full overflow-x-auto pr-6">
        {items.map((t) => (
          <button key={t.id} type="button" role="tab" id={`tab-${t.id}`} aria-selected={t.id === value} aria-controls={`panel-${t.id}`} onClick={() => onChange(t.id)} className="dock-tab">
            {t.label}
            {t.count !== undefined && t.count > 0 && <span className="dock-count tabular">{t.count}</span>}
            {t.alert && <span aria-hidden="true" className="h-2 w-2 rounded-pill bg-red" />}
          </button>
        ))}
      </div>
      <div className="dock-body">{children}</div>
    </div>
  );
}

/** The panel of one tab of a `Dock`: it stays in the page (only hidden) so what was typed in it is not lost. */
export function TabPanel({ id, active, children, className = "" }: { id: string; active: boolean; children: ReactNode; className?: string }) {
  return (
    <div role="tabpanel" id={`panel-${id}`} aria-labelledby={`tab-${id}`} hidden={!active} className={className}>
      {children}
    </div>
  );
}

/** A segmented control: a soft track with the chosen option lifted. For switching how something is shown. */
export function Segmented<T extends string>({
  items,
  value,
  onChange,
  label,
  prefix,
}: {
  items: { id: T; label: string; count?: number }[];
  value: T;
  onChange: (id: T) => void;
  label: string;
  /** a quiet word before the options: «Agrupar por» */
  prefix?: string;
}) {
  return (
    <div role="group" aria-label={label} className="seg items-center">
      {prefix && <span className="mr-1.5 pl-3 text-caption font-semibold text-ink-3">{prefix}</span>}
      {items.map((t) => (
        <button key={t.id} type="button" aria-pressed={t.id === value} onClick={() => onChange(t.id)} className="seg-item">
          {t.label}
          {t.count !== undefined && t.count > 0 && <span className="seg-count tabular">{t.count}</span>}
        </button>
      ))}
    </div>
  );
}

/** Steps of a flow: six bars of one color; what is done and where the person is are full, what comes next is a tint. */
export function Steps({ steps, current, tone = "pc", compact }: { steps: { key: string; label: string }[]; current: number; tone?: Tone; compact?: boolean }) {
  return (
    <ol className={`grid gap-2 tone-${tone}`} style={{ gridTemplateColumns: `repeat(${steps.length}, minmax(0, 1fr))` }}>
      {steps.map((s, i) => {
        const done = i < current;
        const now = i === current;
        return (
          <li key={s.key} aria-current={now ? "step" : undefined} className="flex min-w-0 flex-col gap-2" title={s.label}>
            <i className={`step-bar ${done || now ? "on" : ""}`} />
            {!compact && (
              <span className={`truncate text-caption ${now ? "font-extrabold text-ink" : done ? "font-semibold text-ink-2" : "font-semibold text-ink-3"}`}>
                {done && <Icon name="check" size={12} strokeWidth={3} className="mr-1 inline" />}
                {s.label}
              </span>
            )}
          </li>
        );
      })}
    </ol>
  );
}

/** A bar cut in equal parts (a project's progress at a glance). */
export function Segments({ total, filled, label, tone = "ink" }: { total: number; filled: number; label?: string; tone?: Tone }) {
  return (
    <div className={`segments tone-${tone}`} role="img" aria-label={label ?? `${filled} de ${total}`}>
      {Array.from({ length: total }, (_, i) => (
        <i key={i} className={i < filled ? "on" : ""} />
      ))}
    </div>
  );
}

/** A progress bar: how much of something is done. */
export function Bar({ percent, label, tone = "ink" }: { percent: number; label?: string; tone?: Tone }) {
  return (
    <div className={`bar tone-${tone}`} role="progressbar" aria-valuemin={0} aria-valuemax={100} aria-valuenow={Math.round(percent)} aria-label={label}>
      <i style={{ width: `${Math.max(0, Math.min(100, percent))}%` }} />
    </div>
  );
}

/** The head of a table: small quiet titles over a hairline. */
export function THead({ columns }: { columns: { key: string; title: string; align?: "right" }[] }) {
  return (
    <thead>
      <tr>
        {columns.map((c) => (
          <th key={c.key} scope="col" className={c.align === "right" ? "!text-right" : ""}>
            {c.title}
          </th>
        ))}
      </tr>
    </thead>
  );
}

/** The progress of a flow as small dots in one color: what is done is full, where the person is has a ring. */
export function StepDots({ total, at, done, tone = "pc", className = "" }: { total: number; at: number; done?: boolean; tone?: Tone; className?: string }) {
  return (
    <span className={`dots tone-${tone} ${className}`} aria-hidden="true">
      {Array.from({ length: total }, (_, i) => (
        <i key={i} className={done || i < at ? "on" : i === at ? "on now" : ""} />
      ))}
    </span>
  );
}
