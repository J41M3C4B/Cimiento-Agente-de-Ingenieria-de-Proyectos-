import type { ButtonHTMLAttributes } from "react";
import { Icon } from "../icons";
import type { IconName } from "../icons";
import type { Tone } from "./Tag";

type Variant = "primary" | "secondary" | "soft" | "danger" | "destructive" | "ghost" | "plain";

/** The only button. `md` is 44 high, `sm` 36 (docs/13 §7.1). One `primary` per tray at most. */
export function Button({
  variant = "secondary",
  size = "md",
  className = "",
  ...props
}: ButtonHTMLAttributes<HTMLButtonElement> & { variant?: Variant; size?: "sm" | "md" }) {
  return <button type="button" {...props} className={`btn btn--${variant} ${size === "sm" ? "btn--sm" : ""} ${className}`} />;
}

/** A round button with only an icon. It always needs its name (`label`), which also shows as a tooltip. */
export function IconButton({
  icon,
  label,
  size = "md",
  variant = "default",
  tip,
  tone,
  className = "",
  ...props
}: Omit<ButtonHTMLAttributes<HTMLButtonElement>, "children"> & {
  icon: IconName;
  label: string;
  size?: "sm" | "md";
  variant?: "default" | "plain" | "danger";
  /** a label that opens to the side on hover (used by the left rail) */
  tip?: string;
  /** the color of a module: tinted at rest, solid when it is the page the person is on */
  tone?: Tone;
}) {
  const v = variant === "default" ? "" : `icon-btn--${variant}`;
  const t = tone ? `tone-${tone} icon-btn--tone` : "";
  return (
    <button type="button" aria-label={label} title={tip ? undefined : label} data-tip={tip} {...props} className={`icon-btn ${size === "sm" ? "icon-btn--sm" : ""} ${v} ${t} ${className}`}>
      <Icon name={icon} size={size === "sm" ? 16 : 18} />
    </button>
  );
}

/** «Editar» as a quiet text link: it is there when needed and does not shout. */
export function TextButton({ className = "", ...props }: ButtonHTMLAttributes<HTMLButtonElement>) {
  return <button type="button" {...props} className={`shrink-0 rounded-tick px-2 py-1 text-small font-bold text-ink-3 transition-colors hover:text-ink ${className}`} />;
}
