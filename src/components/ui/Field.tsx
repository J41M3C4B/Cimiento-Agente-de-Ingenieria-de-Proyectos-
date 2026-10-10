import { forwardRef, useId } from "react";
import type { InputHTMLAttributes, ReactNode, SelectHTMLAttributes, TextareaHTMLAttributes } from "react";
import { Icon } from "../icons";
import type { IconName } from "../icons";
import { IconButton } from "./Button";
import type { Tone } from "./Tag";

type FieldProps = {
  label: string;
  hint?: string;
  error?: string;
  children: ReactNode;
  className?: string;
  hideLabel?: boolean;
  required?: boolean;
  /** something small right after the label, on its line: the «?» of the manual (ADR-034) */
  labelAside?: ReactNode;
};

/**
 * With something after the label (a button), the label cannot wrap the field: its control would be that button. Then
 * the label names its field by id, and the button stays out of the label.
 */
function useControlId(labelAside: ReactNode, own?: string) {
  const id = useId();
  return labelAside ? (own ?? id) : own;
}

/** The label, the help and the error of a field. Every field in the program is wrapped in this. */
export function FieldShell({ label, hint, error, children, className = "", hideLabel, required, labelAside, controlId }: FieldProps & { controlId?: string }) {
  if (labelAside && !hideLabel) {
    return (
      <div className={`block min-w-0 ${className}`}>
        <span className="field-label">
          <label htmlFor={controlId}>
            {label}
            {required && <span className="req">*</span>}
          </label>
          {labelAside}
        </span>
        {children}
        {hint && <span className="field-hint">{hint}</span>}
        {error && (
          <span role="alert" className="field-error">
            <Icon name="alert" size={15} />
            {error}
          </span>
        )}
      </div>
    );
  }
  return (
    <label className={`block min-w-0 ${className}`}>
      <span className={hideLabel ? "sr-only" : "field-label"}>
        {label}
        {required && <span className="req">*</span>}
      </span>
      {children}
      {hint && !hideLabel && <span className="field-hint">{hint}</span>}
      {error && (
        <span role="alert" className="field-error">
          <Icon name="alert" size={15} />
          {error}
        </span>
      )}
    </label>
  );
}

type Adornments = { icon?: IconName; prefix?: string; suffix?: string; pill?: boolean };

export const TextInput = forwardRef<HTMLInputElement, InputHTMLAttributes<HTMLInputElement> & Omit<FieldProps, "children"> & Adornments>(function TextInput(
  { label, hint, error, className, hideLabel, required, labelAside, icon, prefix, suffix, pill, ...props },
  ref,
) {
  const pad = `${icon ? "field--icon" : prefix ? "field--prefix" : ""} ${suffix ? "field--suffix" : ""}`;
  const controlId = useControlId(labelAside, props.id);
  return (
    <FieldShell label={label} hint={hint} error={error} className={className} hideLabel={hideLabel} required={required} labelAside={labelAside} controlId={controlId}>
      <span className="field-wrap block">
        {icon && (
          <span aria-hidden="true" className="field-adorn field-adorn--l">
            <Icon name={icon} size={18} />
          </span>
        )}
        {prefix && (
          <span aria-hidden="true" className="field-adorn field-adorn--l">
            {prefix}
          </span>
        )}
        <input ref={ref} {...props} id={controlId} className={`field ${pill ? "field--pill" : ""} ${pad} ${error ? "field--invalid" : ""}`} />
        {suffix && (
          <span aria-hidden="true" className="field-adorn field-adorn--suffix">
            {suffix}
          </span>
        )}
      </span>
    </FieldShell>
  );
});

/** The search box: the same field, as a pill, with its magnifier. */
export const Search = forwardRef<HTMLInputElement, InputHTMLAttributes<HTMLInputElement> & { label: string; className?: string }>(function Search({ label, className, ...props }, ref) {
  return <TextInput ref={ref} label={label} hideLabel icon="search" pill type="search" className={className} {...props} />;
});

export const TextArea = forwardRef<HTMLTextAreaElement, TextareaHTMLAttributes<HTMLTextAreaElement> & Omit<FieldProps, "children">>(function TextArea(
  { label, hint, error, className, hideLabel, required, labelAside, ...props },
  ref,
) {
  const controlId = useControlId(labelAside, props.id);
  return (
    <FieldShell label={label} hint={hint} error={error} className={className} hideLabel={hideLabel} required={required} labelAside={labelAside} controlId={controlId}>
      <textarea ref={ref} rows={4} {...props} id={controlId} className={`field field--area ${error ? "field--invalid" : ""}`} />
    </FieldShell>
  );
});

export const Select = forwardRef<HTMLSelectElement, SelectHTMLAttributes<HTMLSelectElement> & Omit<FieldProps, "children"> & { options: [string, string][] }>(function Select(
  { label, hint, error, className, options, hideLabel, required, labelAside, ...props },
  ref,
) {
  const empty = props.value === "";
  const controlId = useControlId(labelAside, props.id);
  return (
    <FieldShell label={label} hint={hint} error={error} className={className} hideLabel={hideLabel} required={required} labelAside={labelAside} controlId={controlId}>
      <span className="field-wrap block">
        <select ref={ref} {...props} id={controlId} className={`field field--select ${empty ? "text-ink-3" : ""} ${error ? "field--invalid" : ""}`}>
          {options.map(([value, text]) => (
            <option key={value} value={value}>
              {text}
            </option>
          ))}
        </select>
        <span aria-hidden="true" className="field-adorn field-adorn--r !text-ink-2">
          <Icon name="down" size={18} />
        </span>
      </span>
    </FieldShell>
  );
});

/**
 * A field that is not typed into: the covered value of an identifier (CURP, RFC…) in the same box as every other
 * field, with its actions («Mostrar», «Cambiar») on the right.
 */
export function MaskedField({ label, hint, error, value, actions }: { label: string; hint?: string; error?: string; value: ReactNode; actions?: ReactNode }) {
  return (
    <div className="min-w-0">
      <span className="field-label">{label}</span>
      <div className={`field field--static ${error ? "field--invalid" : ""}`}>
        <Icon name="lock" size={16} className="shrink-0 text-ink-3" />
        <span className="tabular min-w-0 flex-1 truncate font-bold">{value}</span>
        {actions && <span className="flex shrink-0 items-center gap-1">{actions}</span>}
      </div>
      {hint && <span className="field-hint">{hint}</span>}
      {error && (
        <span role="alert" className="field-error">
          <Icon name="alert" size={15} />
          {error}
        </span>
      )}
    </div>
  );
}

/** A box to tick. The state lives with whoever uses it. */
export function Check({ checked, onChange, label }: { checked: boolean; onChange: (v: boolean) => void; label: string }) {
  return (
    <button type="button" role="checkbox" aria-checked={checked} aria-label={label} onClick={() => onChange(!checked)} className="check">
      <Icon name="check" size={14} strokeWidth={3} />
    </button>
  );
}

/** An on/off switch with its words. */
export const Switch = forwardRef<HTMLInputElement, InputHTMLAttributes<HTMLInputElement> & { label: string }>(function Switch({ label, className = "", ...props }, ref) {
  return (
    <label className={`switch ${className}`}>
      <input ref={ref} type="checkbox" {...props} />
      <span className="switch-track" aria-hidden="true" />
      {label}
    </label>
  );
});

/** A short option as a pill: the chosen one fills with its color. For states and yes/no. */
export const Choice = forwardRef<HTMLInputElement, InputHTMLAttributes<HTMLInputElement> & { tone?: Tone; children: ReactNode }>(function Choice(
  { tone = "ink", children, ...props },
  ref,
) {
  return (
    <label className="choice">
      <input ref={ref} type="radio" {...props} />
      <span className={`choice-face tone-${tone}`}>{children}</span>
    </label>
  );
});

/** An option that needs explaining: a card with a title and one line. */
export const RadioCard = forwardRef<HTMLInputElement, InputHTMLAttributes<HTMLInputElement> & { title: string; note?: string; icon?: ReactNode }>(function RadioCard(
  { title, note, icon, ...props },
  ref,
) {
  return (
    <label className="radio-card">
      <input ref={ref} type="radio" {...props} />
      <span className="radio-card-face">
        {icon}
        <span className="min-w-0 flex-1">
          <b className="block font-bold">{title}</b>
          {note && <span className="block text-small font-medium text-ink-3">{note}</span>}
        </span>
        <span className="radio-card-dot" aria-hidden="true" />
      </span>
    </label>
  );
});

/**
 * A whole number with its «−» and «+»: how many are in a state, how many there are. It can also be typed. The tile
 * carries the color of what is counted, so the four states of something tell themselves apart at a glance.
 */
export function Stepper({
  label,
  hint,
  value,
  onChange,
  tone = "ink",
  canAdd = true,
  less,
  more,
}: {
  label: string;
  hint?: string;
  value: number;
  onChange: (n: number) => void;
  tone?: Tone;
  /** false when there is nothing left to add (the total is reached) */
  canAdd?: boolean;
  less: string;
  more: string;
}) {
  const id = useId();
  return (
    <div className={`tone-${tone} flex min-w-0 flex-col gap-2 rounded-inset bg-inset p-3`}>
      <div className="flex min-w-0 items-center gap-2">
        <span aria-hidden="true" className="h-4 w-1 shrink-0 rounded-pill bg-[var(--c)]" />
        <label htmlFor={id} className="min-w-0 text-ui font-bold">
          {label}
        </label>
      </div>
      {hint && <span className="text-caption text-ink-3">{hint}</span>}
      <div className="mt-auto flex items-center gap-1">
        <IconButton icon="minus" label={`${less}: ${label}`} size="sm" variant="plain" disabled={value <= 0} onClick={() => onChange(Math.max(0, value - 1))} />
        <input
          id={id}
          inputMode="numeric"
          placeholder="0"
          value={value === 0 ? "" : String(value)}
          onChange={(e) => onChange(Math.max(0, Number(e.target.value.replace(/\D/g, "")) || 0))}
          className="field tabular !h-ctl-sm min-w-0 flex-1 !px-1 text-center font-bold"
        />
        <IconButton icon="plus" label={`${more}: ${label}`} size="sm" variant="plain" disabled={!canAdd} onClick={() => onChange(value + 1)} />
      </div>
    </div>
  );
}
