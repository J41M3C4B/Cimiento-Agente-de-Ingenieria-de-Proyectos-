import { forwardRef, useEffect, useRef, useState } from "react";
import type { ButtonHTMLAttributes, InputHTMLAttributes, ReactNode, SelectHTMLAttributes, TextareaHTMLAttributes } from "react";
import { es } from "../i18n/es-MX";
import { Icon } from "./icons";
import type { IconName } from "./icons";

// Sistema visual de Cimiento: tablero claro y denso. Un solo azul para lo que se puede pulsar o está
// seleccionado, tarjetas con borde fino, estados en tinta oscura sobre fondo suave y siempre con palabras.
// (docs/08-estilo-redaccion.md)

type Variant = "primary" | "secondary" | "soft" | "danger" | "ghost" | "plain";
type Size = "sm" | "md" | "lg";

const variants: Record<Variant, string> = {
  primary: "bg-stone-900 text-white border-transparent shadow-card hover:bg-stone-700 active:translate-y-px",
  secondary: "bg-white text-stone-900 border-transparent shadow-card hover:bg-stone-50 active:translate-y-px",
  /** the quiet button: «Editar», «Ver todo» */
  soft: "bg-stone-100 text-stone-900 border-transparent hover:bg-stone-200",
  danger: "bg-white text-red-800 border-transparent shadow-card hover:bg-red-50",
  ghost: "bg-transparent text-blue-800 border-transparent hover:bg-blue-50",
  /** neutral, for icon buttons */
  plain: "bg-transparent text-stone-700 border-transparent hover:bg-stone-100 hover:text-stone-900",
};

const sizes: Record<Size, string> = {
  sm: "min-h-[34px] px-3 text-[13px]",
  md: "min-h-[40px] px-4 text-[14px]",
  lg: "min-h-[46px] px-5 text-[15px]",
};

export function Button({
  variant = "secondary",
  size = "md",
  className = "",
  ...props
}: ButtonHTMLAttributes<HTMLButtonElement> & { variant?: Variant; size?: Size }) {
  return (
    <button
      type="button"
      {...props}
      className={`inline-flex items-center justify-center gap-2 rounded-xl border font-semibold transition-colors disabled:cursor-not-allowed disabled:border-transparent disabled:bg-stone-100 disabled:text-stone-500 disabled:shadow-none ${sizes[size]} ${variants[variant]} ${className}`}
    />
  );
}

const inputClass =
  "w-full min-h-[42px] rounded-xl border border-stone-300 bg-white px-3.5 text-[14px] text-stone-900 placeholder:text-stone-500 hover:border-stone-400 focus-visible:border-stone-900 focus-visible:outline-none focus-visible:ring-[3px] focus-visible:ring-stone-200 disabled:bg-stone-100";

type FieldProps = { label: string; hint?: string; error?: string; children: ReactNode; className?: string; hideLabel?: boolean };

export function FieldShell({ label, hint, error, children, className = "", hideLabel }: FieldProps) {
  return (
    <label className={`block ${className}`}>
      <span className={hideLabel ? "sr-only" : "mb-1.5 block text-[13px] font-medium text-stone-700"}>{label}</span>
      {hint && <span className="mb-1.5 block text-[13px] text-stone-600">{hint}</span>}
      {children}
      {error && (
        <span role="alert" className="mt-1.5 flex items-center gap-1.5 text-[13px] font-medium text-red-800">
          <Icon name="alert" size={15} />
          {error}
        </span>
      )}
    </label>
  );
}

type Adornments = { icon?: IconName; prefix?: string; suffix?: string };

export const TextInput = forwardRef<
  HTMLInputElement,
  InputHTMLAttributes<HTMLInputElement> & Omit<FieldProps, "children"> & Adornments
>(function TextInput({ label, hint, error, className, hideLabel, icon, prefix, suffix, ...props }, ref) {
  const left = icon || prefix;
  const base = left || suffix ? inputClass.replace(" px-3.5 ", " ") : inputClass;
  const pad = `${left ? (icon ? "pl-9" : "pl-7") : "pl-3.5"} ${suffix ? "pr-16" : "pr-3"}`;
  return (
    <FieldShell label={label} hint={hint} error={error} className={className} hideLabel={hideLabel}>
      <div className="relative">
        {icon && <Icon name={icon} size={16} className="pointer-events-none absolute left-3 top-1/2 -translate-y-1/2 text-stone-600" />}
        {prefix && <span aria-hidden="true" className="pointer-events-none absolute left-3 top-1/2 -translate-y-1/2 text-[14px] text-stone-600">{prefix}</span>}
        <input ref={ref} {...props} className={`${base} ${pad} ${error ? "border-red-800 ring-[3px] ring-red-50" : ""}`} />
        {suffix && <span aria-hidden="true" className="pointer-events-none absolute right-3 top-1/2 -translate-y-1/2 text-[13px] text-stone-600">{suffix}</span>}
      </div>
    </FieldShell>
  );
});

export const TextArea = forwardRef<
  HTMLTextAreaElement,
  TextareaHTMLAttributes<HTMLTextAreaElement> & Omit<FieldProps, "children">
>(function TextArea({ label, hint, error, className, ...props }, ref) {
  return (
    <FieldShell label={label} hint={hint} error={error} className={className}>
      <textarea ref={ref} rows={4} {...props} className={`${inputClass} py-2.5 leading-relaxed`} />
    </FieldShell>
  );
});

export const Select = forwardRef<
  HTMLSelectElement,
  SelectHTMLAttributes<HTMLSelectElement> & Omit<FieldProps, "children"> & { options: [string, string][] }
>(function Select({ label, hint, error, className, options, hideLabel, ...props }, ref) {
  return (
    <FieldShell label={label} hint={hint} error={error} className={className} hideLabel={hideLabel}>
      <div className="relative">
        <select ref={ref} {...props} className={`${inputClass} appearance-none pr-9`}>
          {options.map(([value, text]) => (
            <option key={value} value={value}>
              {text}
            </option>
          ))}
        </select>
        <Icon name="down" size={16} className="pointer-events-none absolute right-3 top-1/2 -translate-y-1/2 text-stone-600" />
      </div>
    </FieldShell>
  );
});

/** A block with a title: where one thing is done. */
export function Section({ title, help, children }: { title: string; help?: string; children: ReactNode }) {
  return (
    <section className="rounded-2xl bg-white p-6 shadow-card">
      <h2 className="text-[16px] font-semibold leading-snug">{title}</h2>
      {help && <p className="mt-1 text-[14px] text-stone-700">{help}</p>}
      <div className="mt-4 space-y-4">{children}</div>
    </section>
  );
}

const tileTones = {
  blue: "bg-blue-50 text-blue-800",
  violet: "bg-violet-50 text-violet-700",
  teal: "bg-teal-50 text-teal-700",
  amber: "bg-amber-50 text-amber-800",
  green: "bg-green-50 text-green-800",
  neutral: "bg-stone-100 text-stone-700",
} as const;
export type Tone = keyof typeof tileTones;

/** The small square with an icon. Each kind of figure keeps its own color, so they tell themselves apart. */
export function IconTile({ icon, tone = "blue", small }: { icon: IconName; tone?: Tone; small?: boolean }) {
  return (
    <span className={`flex shrink-0 items-center justify-center rounded-lg ${small ? "h-7 w-7" : "h-8 w-8"} ${tileTones[tone]}`}>
      <Icon name={icon} size={small ? 15 : 17} />
    </span>
  );
}

/**
 * A section of the page, flat: a title, what is known about it below and «Editar» on the right. Sections are told
 * apart by a thin line and the space between them, not by boxes.
 */
export function Block({ title, note, action, children }: { title: string; note?: string; action?: ReactNode; children: ReactNode }) {
  return (
    <section className="py-6">
      <header className="mb-4 flex items-start justify-between gap-4">
        <div className="min-w-0">
          <h2 className="text-[15px] font-semibold leading-snug">{title}</h2>
          {note && <p className="mt-0.5 text-[13px] text-stone-600">{note}</p>}
        </div>
        {action}
      </header>
      {children}
    </section>
  );
}

/** «Editar» as a text button: it is there when needed and does not shout. */
export function TextButton({ className = "", ...props }: ButtonHTMLAttributes<HTMLButtonElement>) {
  return <button type="button" {...props} className={`shrink-0 rounded-md px-1.5 py-0.5 text-[13px] font-semibold text-blue-800 transition-colors hover:bg-blue-50 ${className}`} />;
}

/** A card, kept for the secondary things on the side (income, what is missing): the main column has none. */
export function Widget({ title, action, children }: { title: string; action?: ReactNode; children: ReactNode }) {
  return (
    <section className="rounded-2xl bg-white shadow-card">
      <header className="flex items-center justify-between gap-3 px-6 pt-5">
        <h2 className="text-[14px] font-semibold">{title}</h2>
        {action}
      </header>
      <div className="px-6 pb-5 pt-3">{children}</div>
    </section>
  );
}

/** What is known, read-only: small label over the value. An empty value says so instead of leaving a hole. */
export function Facts({ items, columns = 3 }: { items: [string, ReactNode][]; columns?: 1 | 2 | 3 }) {
  return (
    <dl className={`grid gap-x-8 gap-y-4 ${columns === 3 ? "sm:grid-cols-2 lg:grid-cols-3" : columns === 2 ? "sm:grid-cols-2" : ""}`}>
      {items.map(([label, value]) => (
        <div key={label} className="min-w-0">
          <dt className="text-[12px] font-medium text-stone-600">{label}</dt>
          <dd className="mt-0.5 break-words text-[14px] font-medium text-stone-900">
            {value || <span className="font-normal text-stone-500">{es.profile.fields.optionNone}</span>}
          </dd>
        </div>
      ))}
    </dl>
  );
}

/** The figures a screen adds up to: one soft tile each, the number big and its detail small underneath. */
export function StatStrip({ items }: { items: { icon: IconName; label: string; value: string; sub?: string; tone?: Tone }[] }) {
  return (
    <dl className="grid gap-4 sm:grid-cols-2 xl:grid-cols-4">
      {items.map((s) => (
        <div key={s.label} className="rounded-2xl bg-white p-5 shadow-card">
          <dt className="flex items-center justify-between gap-3 text-[13px] font-medium text-stone-600">
            {s.label}
            <IconTile icon={s.icon} tone={s.tone} small />
          </dt>
          <dd className="mt-4">
            <span className="block text-[32px] font-semibold leading-none tracking-[-0.03em] tabular-nums">{s.value}</span>
            {s.sub && <span className="mt-2.5 block text-[12.5px] text-stone-500">{s.sub}</span>}
          </dd>
        </div>
      ))}
    </dl>
  );
}

/** Tabs as a segmented control: the chosen one is filled with dark ink. */
export function Tabs<T extends string>({
  items,
  value,
  onChange,
  label,
}: {
  items: { id: T; label: string; count?: number; alert?: boolean }[];
  value: T;
  onChange: (id: T) => void;
  label: string;
}) {
  return (
    <div role="tablist" aria-label={label} className="inline-flex max-w-full gap-1 overflow-x-auto rounded-2xl bg-white p-1 shadow-card">
      {items.map((t) => {
        const on = t.id === value;
        return (
          <button
            key={t.id}
            type="button"
            role="tab"
            id={`tab-${t.id}`}
            aria-selected={on}
            aria-controls={`panel-${t.id}`}
            onClick={() => onChange(t.id)}
            className={`flex shrink-0 items-center gap-2 rounded-xl px-4 py-2 text-[14px] font-medium transition-colors ${
              on ? "bg-stone-900 text-white" : "text-stone-600 hover:bg-stone-100 hover:text-stone-900"
            }`}
          >
            {t.label}
            {t.count !== undefined && t.count > 0 && (
              <span className={`rounded-md px-1.5 text-[12px] font-semibold tabular-nums ${on ? "bg-white/20 text-white" : "bg-white text-stone-700"}`}>{t.count}</span>
            )}
            {t.alert && <span aria-hidden="true" className="h-2 w-2 rounded-full bg-red-800 ring-2 ring-white" />}
          </button>
        );
      })}
    </div>
  );
}

/** The head of a table: every column title is its own soft pill. */
export function THead({ columns }: { columns: { key: string; title: string; align?: "right" }[] }) {
  return (
    <thead>
      <tr>
        {columns.map((c) => (
          <th key={c.key} scope="col" className="px-0.5 pb-2 text-left font-medium first:pl-0 last:pr-0">
            {c.title && <span className={`block rounded-md bg-stone-100 px-3 py-2 text-[12px] font-medium text-stone-700 ${c.align === "right" ? "text-right" : ""}`}>{c.title}</span>}
          </th>
        ))}
      </tr>
    </thead>
  );
}

/** Edit and remove for one row of a list. They show when the row is pointed at; removing asks first, right there. */
export function RowActions({ onEdit, onRemove, busy }: { onEdit: () => void; onRemove: () => void; busy?: boolean }) {
  const [asking, setAsking] = useState(false);
  if (asking) {
    return (
      <span className="inline-flex items-center gap-1">
        <span className="text-[13px] font-medium text-red-800">{es.roster.table.sure}</span>
        <Button size="sm" variant="danger" disabled={busy} onClick={onRemove}>
          {es.common.yes}
        </Button>
        <Button size="sm" variant="plain" onClick={() => setAsking(false)}>
          {es.common.no}
        </Button>
      </span>
    );
  }
  return (
    <span className="inline-flex opacity-0 transition-opacity focus-within:opacity-100 group-hover:opacity-100">
      <Button size="sm" variant="plain" aria-label={es.roster.table.edit} title={es.roster.table.edit} onClick={onEdit} className="!px-2">
        <Icon name="pencil" size={16} />
      </Button>
      <Button size="sm" variant="plain" aria-label={es.common.remove} title={es.common.remove} onClick={() => setAsking(true)} className="!px-2 hover:!text-red-800">
        <Icon name="trash" size={16} />
      </Button>
    </span>
  );
}

/** Vivid tags: an emblem for a position or a state. Never gray, never pale. */
const tagTones = {
  blue: "bg-[#cfe0ff] text-[#173fb8]",
  violet: "bg-[#ddd2ff] text-[#4c26c4]",
  teal: "bg-[#bff0e6] text-[#06695c]",
  green: "bg-[#c4f0d1] text-[#0f6b32]",
  amber: "bg-[#ffe29a] text-[#7a4a00]",
  orange: "bg-[#ffd0b0] text-[#a53c05]",
  pink: "bg-[#ffcfe5] text-[#a01f62]",
  red: "bg-[#ffc7c7] text-[#a01818]",
} as const;
export type TagTone = keyof typeof tagTones;

/** The colors used to tell positions, groups and people apart (red is left for what is wrong). */
const SERIES: TagTone[] = ["blue", "violet", "teal", "amber", "pink", "green", "orange"];
export const seriesTone = (index: number): TagTone => SERIES[((index % SERIES.length) + SERIES.length) % SERIES.length]!;
/** The same color every time for the same text. */
export const toneOfText = (text: string): TagTone => seriesTone([...text].reduce((h, c) => (h * 31 + c.charCodeAt(0)) >>> 0, 7));

export function Tag({ tone, children }: { tone: TagTone; children: ReactNode }) {
  return <span className={`inline-flex min-h-6 items-center whitespace-nowrap rounded-md px-2.5 text-[12.5px] font-semibold ${tagTones[tone]}`}>{children}</span>;
}

/** The round mark of a person: their initials, in a color of their own. */
export function Avatar({ name }: { name?: string }) {
  const initials =
    (name ?? "")
      .split(/\s+/)
      .filter(Boolean)
      .slice(0, 2)
      .map((w) => w[0]!.toUpperCase())
      .join("") || "·";
  return (
    <span aria-hidden="true" className={`flex h-7 w-7 shrink-0 items-center justify-center rounded-full text-[11px] font-bold ${tagTones[toneOfText(name ?? "")]}`}>
      {initials}
    </span>
  );
}

/** A small note that appears over its element when it is pointed at or focused: for sources, hints and the like. */
export function Tip({ text, children, align = "center" }: { text: string; children: ReactNode; align?: "center" | "start" }) {
  return (
    <span className="group/tip relative inline-flex">
      {children}
      <span
        role="tooltip"
        className={`pointer-events-none absolute bottom-full z-30 mb-1.5 w-max max-w-[260px] rounded-md bg-stone-900 px-2.5 py-1.5 text-[12px] font-medium leading-snug text-white opacity-0 shadow-lift transition-opacity duration-150 group-hover/tip:opacity-100 group-focus-within/tip:opacity-100 ${
          align === "center" ? "left-1/2 -translate-x-1/2" : "left-0"
        }`}
      >
        {text}
      </span>
    </span>
  );
}

/** A part that opens and closes: the title (with a count) always shows, the content only when it is open. */
export function Disclosure({
  title,
  count,
  defaultOpen = false,
  tone = "neutral",
  children,
}: {
  title: string;
  count?: number;
  defaultOpen?: boolean;
  tone?: "neutral" | "warn";
  children: ReactNode;
}) {
  const [open, setOpen] = useState(defaultOpen);
  return (
    <div className={`rounded-2xl ${tone === "warn" ? "border border-amber-300 bg-amber-50/60" : "bg-white shadow-card"}`}>
      <button
        type="button"
        aria-expanded={open}
        onClick={() => setOpen(!open)}
        className="flex w-full items-center gap-3 px-4 py-3 text-left"
      >
        <span className="min-w-0 flex-1 text-[14px] font-semibold">{title}</span>
        {count !== undefined && <span className="rounded-md bg-stone-100 px-1.5 text-[12px] font-semibold tabular-nums text-stone-700">{count}</span>}
        <Icon name="down" size={16} className={`shrink-0 text-stone-600 transition-transform duration-200 ${open ? "rotate-180" : ""}`} />
      </button>
      <div className={`grid transition-[grid-template-rows] duration-200 ease-out ${open ? "grid-rows-[1fr]" : "grid-rows-[0fr]"}`}>
        <div className="overflow-hidden">
          <div className="border-t border-stone-200 px-4 py-3">{children}</div>
        </div>
      </div>
    </div>
  );
}

const alertTones = {
  info: { box: "bg-blue-50", badge: "text-blue-800", icon: "info" },
  warn: { box: "bg-amber-50", badge: "text-amber-800", icon: "warn" },
  error: { box: "bg-red-50", badge: "text-red-800", icon: "alert" },
  ok: { box: "bg-green-50", badge: "text-green-800", icon: "check" },
} as const;

export function Alert({ tone, children }: { tone: "info" | "warn" | "error" | "ok"; children: ReactNode }) {
  const t = alertTones[tone];
  return (
    <div role={tone === "error" ? "alert" : "status"} className={`flex items-start gap-3 rounded-lg p-3 text-[14px] ${t.box}`}>
      <span className={`flex h-7 w-7 shrink-0 items-center justify-center rounded-full bg-white ${t.badge}`}>
        <Icon name={t.icon as IconName} size={15} strokeWidth={tone === "ok" ? 2.6 : 1.8} />
      </span>
      <div className="min-w-0 flex-1 pt-0.5">{children}</div>
    </div>
  );
}

const chipTones = {
  neutral: "bg-stone-100 text-stone-700",
  blue: "bg-blue-50 text-blue-900",
  green: "bg-green-50 text-green-800",
  amber: "bg-amber-50 text-amber-800",
  red: "bg-red-50 text-red-800",
} as const;

/** A short label for a state: «Confirmada», «Por revisar»… (always with words, never only a color). */
export function Chip({ tone = "neutral", icon, dot, children }: { tone?: keyof typeof chipTones; icon?: IconName; dot?: boolean; children: ReactNode }) {
  return (
    <span className={`inline-flex min-h-6 items-center gap-1.5 whitespace-nowrap rounded-md px-2 text-[12.5px] font-medium ${chipTones[tone]}`}>
      {dot && <span aria-hidden="true" className="h-1.5 w-1.5 rounded-full bg-current" />}
      {icon && <Icon name={icon} size={13} strokeWidth={2.6} />}
      {children}
    </span>
  );
}

/** Steps of a flow: what is done, where the person is and what comes next. */
export function Steps({ steps, current, compact }: { steps: { key: string; label: string }[]; current: number; compact?: boolean }) {
  if (compact) {
    // one slim row: small circles joined by a line, and the name only for the step the person is at
    return (
      <ol className="flex flex-wrap items-center gap-y-2">
        {steps.map((s, i) => {
          const done = i < current;
          const now = i === current;
          return (
            <li key={s.key} aria-current={now ? "step" : undefined} className="flex items-center">
              {i > 0 && <span aria-hidden="true" className={`mx-1.5 h-0.5 w-5 rounded-full ${done || now ? "bg-stone-900" : "bg-stone-300"}`} />}
              <span
                title={s.label}
                className={`flex h-6 w-6 items-center justify-center rounded-full border-2 text-[11px] font-semibold ${
                  done ? "border-stone-900 bg-stone-900 text-white" : now ? "border-stone-900 bg-white text-stone-900" : "border-stone-300 bg-transparent text-stone-500"
                }`}
              >
                {done ? <Icon name="check" size={12} strokeWidth={3} /> : i + 1}
              </span>
              {now && <span className="ml-2 text-[13px] font-semibold text-stone-900">{s.label}</span>}
            </li>
          );
        })}
      </ol>
    );
  }
  return (
    <ol className="flex">
      {steps.map((s, i) => {
        const done = i < current;
        const now = i === current;
        return (
          <li key={s.key} aria-current={now ? "step" : undefined} className="relative flex min-w-0 flex-1 flex-col items-center gap-2 text-center">
            {i > 0 && <span aria-hidden="true" className={`absolute left-[-50%] top-[15px] h-0.5 w-full ${done || now ? "bg-stone-900" : "bg-stone-300"}`} />}
            <span
              className={`relative z-10 flex h-8 w-8 items-center justify-center rounded-full border-2 text-[13px] font-semibold ${
                done ? "border-stone-900 bg-stone-900 text-white" : now ? "border-stone-900 bg-white text-stone-900 ring-[4px] ring-stone-200" : "border-stone-300 bg-white text-stone-700"
              }`}
            >
              {done ? <Icon name="check" size={14} strokeWidth={3} /> : i + 1}
            </span>
            <span className={`text-[13px] ${now ? "font-semibold text-stone-900" : done ? "font-medium text-stone-900" : "font-medium text-stone-700"}`}>{s.label}</span>
          </li>
        );
      })}
    </ol>
  );
}

/** A bar cut in equal parts (a project's progress at a glance). */
export function Segments({ total, filled, label }: { total: number; filled: number; label?: string }) {
  return (
    <div className="flex gap-1" role="img" aria-label={label ?? `${filled} de ${total}`}>
      {Array.from({ length: total }, (_, i) => (
        <span key={i} className={`h-1.5 flex-1 rounded-full ${i < filled ? "bg-stone-900" : "bg-stone-300/70"}`} />
      ))}
    </div>
  );
}

/** A progress bar: how much of something is done. */
export function Bar({ percent, label }: { percent: number; label?: string }) {
  return (
    <div className="h-2 w-full overflow-hidden rounded-full bg-stone-200" role="progressbar" aria-valuemin={0} aria-valuemax={100} aria-valuenow={Math.round(percent)} aria-label={label}>
      <div className="h-full rounded-full bg-stone-900" style={{ width: `${Math.max(0, Math.min(100, percent))}%` }} />
    </div>
  );
}

/**
 * A window in the middle of the screen. Escape closes it; with `dismissable` a click outside does too (only for
 * windows where closing loses nothing important). `footer` holds the buttons, on a soft strip.
 */
export function Modal({
  title,
  children,
  onClose,
  footer,
  size = "md",
  dismissable = false,
}: {
  title: string;
  children: ReactNode;
  onClose?: () => void;
  footer?: ReactNode;
  size?: "md" | "lg" | "xl";
  dismissable?: boolean;
}) {
  const ref = useRef<HTMLDivElement>(null);
  // the latest «close» is kept apart: a window written with an inline function must not lose the person's focus
  // every time it is drawn again (typing in a field draws it again)
  const closing = useRef(onClose);
  closing.current = onClose;
  useEffect(() => {
    if (!ref.current?.contains(document.activeElement)) ref.current?.focus();
    const onKey = (e: KeyboardEvent) => {
      if (e.key !== "Escape") return;
      const windows = document.querySelectorAll('[role="dialog"]');
      if (windows[windows.length - 1] === ref.current) closing.current?.();
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, []);
  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-stone-900/40 p-4 backdrop-blur-[3px]"
      onMouseDown={(e) => dismissable && e.target === e.currentTarget && onClose?.()}
    >
      <div
        ref={ref}
        tabIndex={-1}
        role="dialog"
        aria-modal="true"
        aria-label={title}
        className={`flex max-h-[92vh] w-full flex-col overflow-hidden rounded-3xl bg-white shadow-lift outline-none ${size === "xl" ? "max-w-5xl" : size === "lg" ? "max-w-3xl" : "max-w-xl"}`}
      >
        <header className="flex items-center justify-between gap-4 px-6 pb-3 pt-5">
          <h2 className="text-[18px] font-semibold tracking-tight">{title}</h2>
          {onClose && (
            <Button size="sm" variant="plain" aria-label={es.common.close} title={es.common.close} onClick={onClose} className="!px-2">
              <Icon name="x" size={18} />
            </Button>
          )}
        </header>
        <div className="min-h-0 flex-1 space-y-4 overflow-y-auto px-6 pb-6 pt-1">{children}</div>
        {footer && <footer className="flex flex-wrap justify-end gap-2.5 border-t border-stone-200 bg-stone-50 px-6 py-3.5">{footer}</footer>}
      </div>
    </div>
  );
}
