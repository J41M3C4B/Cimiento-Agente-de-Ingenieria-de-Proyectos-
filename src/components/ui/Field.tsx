import { forwardRef } from "react";
import type { InputHTMLAttributes, ReactNode, SelectHTMLAttributes, TextareaHTMLAttributes } from "react";
import { Icon } from "../icons";
import type { IconName } from "../icons";
import type { Tone } from "./Tag";

type FieldProps = { label: string; hint?: string; error?: string; children: ReactNode; className?: string; hideLabel?: boolean; required?: boolean };

/** The label, the help and the error of a field. Every field in the program is wrapped in this. */
export function FieldShell({ label, hint, error, children, className = "", hideLabel, required }: FieldProps) {
  return (
    <label className={`block min-w-0 ${className}`}>
      <span className={hideLabel ? "sr-only" : "field-label"}>
        {label}
        {required && <span className="req">*</span>}
      </span>
      {hint && !hideLabel && <span className="field-hint">{hint}</span>}
      {children}
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
  { label, hint, error, className, hideLabel, required, icon, prefix, suffix, pill, ...props },
  ref,
) {
  const pad = `${icon ? "field--icon" : prefix ? "field--prefix" : ""} ${suffix ? "field--suffix" : ""}`;
  return (
    <FieldShell label={label} hint={hint} error={error} className={className} hideLabel={hideLabel} required={required}>
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
        <input ref={ref} {...props} className={`field ${pill ? "field--pill" : ""} ${pad} ${error ? "field--invalid" : ""}`} />
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
  { label, hint, error, className, hideLabel, required, ...props },
  ref,
) {
  return (
    <FieldShell label={label} hint={hint} error={error} className={className} hideLabel={hideLabel} required={required}>
      <textarea ref={ref} rows={4} {...props} className={`field field--area ${error ? "field--invalid" : ""}`} />
    </FieldShell>
  );
});

export const Select = forwardRef<HTMLSelectElement, SelectHTMLAttributes<HTMLSelectElement> & Omit<FieldProps, "children"> & { options: [string, string][] }>(function Select(
  { label, hint, error, className, options, hideLabel, required, ...props },
  ref,
) {
  const empty = props.value === "";
  return (
    <FieldShell label={label} hint={hint} error={error} className={className} hideLabel={hideLabel} required={required}>
      <span className="field-wrap block">
        <select ref={ref} {...props} className={`field field--select ${empty ? "text-ink-3" : ""} ${error ? "field--invalid" : ""}`}>
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
