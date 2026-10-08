import type { ReactNode } from "react";
import { Icon } from "../icons";
import type { Tone } from "./Tag";

type TabItem<T extends string> = { id: T; label: string; count?: number; alert?: boolean };

/**
 * The tabs of a page (docs/13 §6, folder effect): the chosen tab is white and joins the tray of its content (the first tray under the tabs carries `dock-attach`); changing tab moves the joined tab. There is no gray surface behind. The panel is the
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
  return (
    <div className="dock" data-first={items[0]?.id === value}>
      <div role="tablist" aria-label={label} className="dock-tabs max-w-full overflow-x-auto pr-6">
        {items.map((t) => (
          <button key={t.id} type="button" role="tab" id={`tab-${t.id}`} aria-selected={t.id === value} aria-controls={`panel-${t.id}`} onClick={() => onChange(t.id)} className="dock-tab">
            {t.label}
            {t.count !== undefined && t.count > 0 && <span className="dock-count tabular">{t.count}</span>}
            {t.alert && <span aria-hidden="true" className="h-2 w-2 rounded-pill bg-red" />}
          </button>
        ))}
      </div>
      <div>{children}</div>
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

/**
 * The steps of a long form as buttons the person can jump between. Each one shows how much of it is filled in
 * (a bar and «9 de 12»), or that it does not apply; a step that is complete turns green with a check.
 */
export function StepNav({
  steps,
  current,
  onSelect,
  label,
}: {
  steps: { key: string; label: string; filled?: number; total?: number; na?: boolean; caption?: string; naLabel?: string }[];
  current: number;
  onSelect: (index: number) => void;
  label: string;
}) {
  const here = steps[current];
  return (
    <div className="space-y-2">
    <ol aria-label={label} className="grid gap-3" style={{ gridTemplateColumns: `repeat(${steps.length}, minmax(0, 1fr))` }}>
      {steps.map((s, i) => {
        const now = i === current;
        const known = s.filled !== undefined && s.total !== undefined && s.total > 0 && !s.na;
        const done = known && s.filled === s.total;
        const percent = known ? (s.filled! / s.total!) * 100 : 0;
        return (
          <li key={s.key} className="min-w-0">
            <button type="button" aria-current={now ? "step" : undefined} onClick={() => onSelect(i)} className="flex w-full min-w-0 flex-col gap-1.5 rounded-field text-left">
              <span className={`bar ${done ? "tone-green" : "tone-ink"} ${now ? "" : "opacity-70"}`}>
                <i style={{ width: `${percent}%` }} />
              </span>
              <span className={`flex items-center gap-2 truncate text-small max-sm:sr-only ${now ? "font-extrabold text-ink" : "font-bold text-ink-2"}`}>
                <span className={`grid h-5 w-5 shrink-0 place-items-center rounded-pill text-caption font-extrabold ${done ? "bg-green text-onc" : now ? "bg-ink text-on-ink" : "bg-inset text-ink-2"}`}>
                  {done ? <Icon name="check" size={12} strokeWidth={3} /> : s.na ? "–" : i + 1}
                </span>
                <span className="truncate">{s.label}</span>
              </span>
              <span className="truncate text-caption text-ink-3 max-sm:sr-only">{s.na ? s.naLabel : known ? s.caption : "\u00a0"}</span>
            </button>
          </li>
        );
      })}
    </ol>
    {here && (
      <p className="text-small font-extrabold sm:hidden">
        {here.label}
        <span className="font-medium text-ink-3">{here.na ? ` · ${here.naLabel}` : here.caption ? ` · ${here.caption}` : ""}</span>
      </p>
    )}
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
              <span className={`truncate text-caption ${now ? "font-extrabold text-ink" : done ? "font-semibold text-ink-2 max-sm:hidden" : "font-semibold text-ink-3 max-sm:hidden"}`}>
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
