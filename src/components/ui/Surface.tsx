import { useLayoutEffect, useRef } from "react";
import type { ElementType, ReactNode } from "react";
import { es } from "../../i18n/es-MX";
import type { IconName } from "../icons";
import { Eyebrow } from "./Content";
import { Tile } from "./Tag";
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
  tabs,
  children,
  className = "",
  bodyClassName = "",
  as: As = "div",
}: {
  tone?: Tone;
  /** the one tab of a folder with a single view */
  title?: ReactNode;
  /** text of the title for the `title` attribute when it is cut short */
  titleText?: string;
  chip?: ReactNode;
  /** a folder with several views: one tab each, the chosen one in the folder's color and joined to the body */
  tabs?: { items: { id: string; label: string; count?: number }[]; value: string; onChange: (id: string) => void; label: string };
  children: ReactNode;
  className?: string;
  bodyClassName?: string;
  as?: ElementType;
}) {
  const bodyRef = useRef<HTMLDivElement>(null);
  const panelRef = useRef<HTMLDivElement>(null);
  const bodyHeight = useRef<number | null>(null);
  const view = tabs?.value;

  // Keep the height of the body up to date, so a change of view can start from the one that was on screen.
  useLayoutEffect(() => {
    const el = bodyRef.current;
    if (!el || view === undefined || typeof ResizeObserver === "undefined") return;
    const watch = new ResizeObserver(() => { bodyHeight.current = el.getBoundingClientRect().height; });
    watch.observe(el);
    return () => watch.disconnect();
  }, [view === undefined]);

  // A change of view is a transformation, not a jump: the body grows or shrinks to its new height and the content settles in.
  useLayoutEffect(() => {
    const el = bodyRef.current;
    if (!el || view === undefined) return;
    const next = el.getBoundingClientRect().height;
    const prev = bodyHeight.current;
    bodyHeight.current = next;
    if (prev === null || typeof el.animate !== "function" || window.matchMedia("(prefers-reduced-motion: reduce)").matches) return;
    panelRef.current?.animate(
      [{ opacity: 0, transform: "translateY(10px)" }, { opacity: 1, transform: "none" }],
      { duration: 460, easing: "cubic-bezier(0.22, 1, 0.36, 1)" },
    );
    if (Math.abs(prev - next) < 1) return;
    const keep = { flex: el.style.flex, overflow: el.style.overflow };
    el.style.flex = "none";
    el.style.overflow = "hidden";
    const grow = el.animate([{ height: `${prev}px` }, { height: `${next}px` }], { duration: 420, easing: "cubic-bezier(0.22, 1, 0.36, 1)" });
    const done = () => { el.style.flex = keep.flex; el.style.overflow = keep.overflow; };
    grow.onfinish = done;
    grow.oncancel = done;
  }, [view]);

  return (
    <As className={`folder tone-${tone} ${className}`}>
      <div className="folder-top">
        {tabs ? (
          <div role="tablist" aria-label={tabs.label} className="folder-tabs">
            {tabs.items.map((t, i) => {
              const on = t.id === tabs.value;
              return (
                <button key={t.id} type="button" role="tab" id={`ftab-${t.id}`} aria-selected={on} aria-controls={`fpanel-${t.id}`} onClick={() => tabs.onChange(t.id)} className="folder-tabbtn" style={{ zIndex: tabs.items.length - i }}>
                  {t.label}
                  {t.count !== undefined && t.count > 0 && <span className="folder-count tabular">{t.count}</span>}
                </button>
              );
            })}
          </div>
        ) : (
          <div className="folder-tab">
            <b className="folder-title">{title}</b>
            {chip}
          </div>
        )}
      </div>
      <div
        ref={bodyRef}
        className={`folder-body ${bodyClassName}`}
        {...(tabs ? { role: "tabpanel", id: `fpanel-${tabs.value}`, "aria-labelledby": `ftab-${tabs.value}` } : {})}
      >
        {tabs ? <div ref={panelRef} className="folder-panel">{children}</div> : children}
      </div>
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

/** The head of every page: its title, one line that says what it is for, and (on the right) its main action. */
export function PageHeader({ title, intro, action }: { title: string; intro?: string; action?: ReactNode }) {
  return (
    <header className="flex flex-wrap items-center justify-between gap-x-6 gap-y-4">
      <div className="min-w-[260px] flex-1 space-y-1">
        <h1 className="text-title font-bold tracking-tight">{title}</h1>
        {intro && <p className="text-ui text-ink-2">{intro}</p>}
      </div>
      {action}
    </header>
  );
}

/** A group of fields in a window: a small heading in capitals with a hairline, then the fields (docs/13 §7, «Formulario»). */
export function FormSection({ title, icon, children }: { title: string; icon?: IconName; children: ReactNode }) {
  return (
    <section aria-label={title} className="flex flex-col gap-4">
      <div aria-hidden="true" className="flex items-center gap-3">
        {icon && <Tile icon={icon} tone="neutral" small />}
        <Eyebrow>{title}</Eyebrow>
        <span className="h-px flex-1 bg-line" />
      </div>
      {children}
    </section>
  );
}
