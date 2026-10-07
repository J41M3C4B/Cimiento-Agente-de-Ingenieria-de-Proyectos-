import { useEffect, useRef, useState } from "react";
import type { ReactNode } from "react";
import { es } from "../../i18n/es-MX";
import { Icon } from "../icons";
import type { IconName } from "../icons";
import { Button, IconButton } from "./Button";

/**
 * A window in the middle of the screen. Escape closes it; with `dismissable` a click outside does too (only for
 * windows where closing loses nothing important). `footer` holds the buttons.
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
    <div className="scrim" onMouseDown={(e) => dismissable && e.target === e.currentTarget && onClose?.()}>
      <div ref={ref} tabIndex={-1} role="dialog" aria-modal="true" aria-label={title} className={`modal ${size === "xl" ? "max-w-5xl" : size === "lg" ? "max-w-3xl" : "max-w-[640px]"}`}>
        <header className="flex items-center justify-between gap-4 px-6 pb-2 pt-6">
          <h2 className="text-subtitle font-bold tracking-tight">{title}</h2>
          {onClose && <IconButton icon="x" label={es.common.close} variant="default" size="sm" onClick={onClose} />}
        </header>
        <div className="min-h-0 flex-1 space-y-6 overflow-y-auto px-6 pb-6 pt-2">{children}</div>
        {footer && <footer className="modal-foot">{footer}</footer>}
      </div>
    </div>
  );
}

/** A small note that appears over its element when it is pointed at or focused: for sources, hints and the like. */
export function Tip({ text, children, align = "center" }: { text: string; children: ReactNode; align?: "center" | "start" }) {
  return (
    <span className="group/tip relative inline-flex">
      {children}
      <span
        role="tooltip"
        className={`tooltip bottom-full mb-1.5 max-w-[260px] !whitespace-normal group-hover/tip:opacity-100 group-focus-within/tip:opacity-100 ${align === "center" ? "left-1/2 -translate-x-1/2" : "left-0"}`}
      >
        {text}
      </span>
    </span>
  );
}

const alertTones = {
  info: { tone: "sky", icon: "info" },
  warn: { tone: "amber", icon: "warn" },
  error: { tone: "red", icon: "alert" },
  ok: { tone: "green", icon: "check" },
} as const;

/** A notice in the color of its state (blue says, yellow warns, red failed, green is done), always with words. */
export function Alert({ tone, children }: { tone: "info" | "warn" | "error" | "ok"; children: ReactNode }) {
  const t = alertTones[tone];
  return (
    <div role={tone === "error" ? "alert" : "status"} className={`alert tone-${t.tone}`}>
      <span className="alert-badge">
        <Icon name={t.icon as IconName} size={15} strokeWidth={tone === "ok" ? 2.6 : 1.8} />
      </span>
      <div className="min-w-0 flex-1 pt-0.5">{children}</div>
    </div>
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
    <div className={`rounded-inset ${tone === "warn" ? "tone-amber bg-amber/20" : "bg-inset"}`}>
      <button type="button" aria-expanded={open} onClick={() => setOpen(!open)} className="flex w-full items-center gap-3 px-5 py-3.5 text-left">
        <span className="min-w-0 flex-1 text-ui font-bold">{title}</span>
        {count !== undefined && <span className="tabular rounded-pill bg-card px-2 text-caption font-bold leading-5 text-ink-2">{count}</span>}
        <Icon name="down" size={18} className={`shrink-0 text-ink-2 transition-transform duration-200 ${open ? "rotate-180" : ""}`} />
      </button>
      <div className={`grid transition-[grid-template-rows] duration-200 ease-out ${open ? "grid-rows-[1fr]" : "grid-rows-[0fr]"}`}>
        <div className="overflow-hidden">
          <div className="border-t border-line px-5 py-4">{children}</div>
        </div>
      </div>
    </div>
  );
}

/** Edit and remove for one row of a list. They show when the row is pointed at; removing asks first, right there. */
export function RowActions({ onEdit, onRemove, busy }: { onEdit: () => void; onRemove: () => void; busy?: boolean }) {
  const [asking, setAsking] = useState(false);
  if (asking) {
    return (
      <span className="inline-flex items-center gap-2">
        <span className="text-small font-bold text-red">{es.roster.table.sure}</span>
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
    <span className="inline-flex gap-1 opacity-0 transition-opacity focus-within:opacity-100 group-hover:opacity-100 [@media(hover:none)]:opacity-100">
      <IconButton icon="pencil" label={es.roster.table.edit} variant="plain" size="sm" onClick={onEdit} />
      <IconButton icon="trash" label={es.common.remove} variant="plain" size="sm" onClick={() => setAsking(true)} />
    </span>
  );
}

/** A short notice at the bottom of the screen that confirms what was just done. */
export function Toast({ children, tone = "ok" }: { children: ReactNode; tone?: "ok" | "error" }) {
  return (
    <div role="status" className="toast anim-rise">
      <span className={`tone-${tone === "ok" ? "green" : "red"} grid h-5 w-5 place-items-center rounded-pill bg-[var(--c)] text-onc`}>
        <Icon name={tone === "ok" ? "check" : "alert"} size={13} strokeWidth={3} />
      </span>
      {children}
    </div>
  );
}
