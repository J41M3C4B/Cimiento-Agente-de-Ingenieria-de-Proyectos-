import { useId, useState } from "react";
import type { ReactNode } from "react";
import { Icon } from "./icons";
import { HelpToggle, ManualPanel } from "./ManualHelp";
import { Choice, FormSection, Select, TextArea, TextInput } from "./ui";
import { es } from "../i18n/es-MX";
import type { FieldSpec, FormSpec, FormValue, FormValues } from "../lib/types";

const words = (id: string) => es.forms.fields[id] ?? { label: id };
/** Up to this many options are shown as pills; more go in a list that drops down. */
const PILLS_UP_TO = 4;

const filled = (v: FormValue | undefined) => (Array.isArray(v) ? v.length > 0 : typeof v === "string" ? v.trim() !== "" : v !== undefined);

/**
 * Whether a field applies with what is typed now, so it shows or hides as the person answers. Rust decides it again
 * when saving (`common/forms.rs`) and drops what does not apply; this only keeps the screen in step.
 */
export function applies(f: FieldSpec, values: FormValues): boolean {
  const c = f.applies_when;
  if (c.when === "always") return true;
  const v = values[c.field];
  if (c.when === "filled") return filled(v);
  return Array.isArray(v) ? v.some((x) => c.values.includes(x)) : typeof v === "string" && c.values.includes(v);
}

type Props = {
  spec: FormSpec;
  values: FormValues;
  onChange: (values: FormValues) => void;
  /** The problem of a field, already in words, by field id. */
  errors?: Record<string, string>;
};

/**
 * A form described in Rust (ADR-033), drawn with the components of the system: a section per group of fields, and
 * per field the input its kind asks for. The words come from `es.forms` under the id of each field; the «?» next to
 * each label opens what the manual says of it (ADR-034), one at a time.
 */
export function FormRenderer({ spec, values, onChange, errors = {} }: Props) {
  const set = (id: string, v: FormValue) => onChange({ ...values, [id]: v });
  const [help, setHelp] = useState<string | null>(null);
  return (
    <div className="space-y-6">
      {spec.sections.map((s) => {
        const shown = s.fields.filter((f) => applies(f, values));
        if (shown.length === 0) return null;
        return (
          <FormSection key={s.id} title={es.forms.sections[s.id] ?? s.id}>
            <div className={`grid grid-cols-1 gap-4 ${s.columns > 1 ? "sm:grid-cols-2" : ""}`}>
              {shown.map((f) => {
                const panelId = `manual-${f.id}`;
                const aside = <HelpToggle open={help === f.id} onToggle={() => setHelp(help === f.id ? null : f.id)} controls={panelId} />;
                return (
                  <div key={f.id} className="min-w-0">
                    <Field f={f} value={values[f.id]} error={errors[f.id]} onChange={(v) => set(f.id, v)} aside={aside} />
                    {help === f.id && <ManualPanel id={f.id} panelId={panelId} />}
                  </div>
                );
              })}
            </div>
          </FormSection>
        );
      })}
    </div>
  );
}

function Field({ f, value, error, onChange, aside }: { f: FieldSpec; value: FormValue | undefined; error?: string; onChange: (v: FormValue) => void; aside: ReactNode }) {
  const w = words(f.id);
  const label = (code: string) => w.options?.[code] ?? code;
  const text = typeof value === "string" ? value : value === undefined ? "" : String(value);

  switch (f.kind) {
    case "long_text":
      return <TextArea label={w.label} hint={w.hint} error={error} required={f.required} labelAside={aside} value={text} onChange={(e) => onChange(e.target.value)} />;
    case "multi_select": {
      const chosen = Array.isArray(value) ? value : [];
      return (
        <Pills label={w.label} hint={w.hint} error={error} required={f.required} labelAside={aside}>
          {f.options.map((code) => (
            <Choice
              key={code}
              type="checkbox"
              checked={chosen.includes(code)}
              // the codes keep the order of the list, whatever the order they were marked in
              onChange={(e) => onChange(f.options.filter((x) => (x === code ? e.target.checked : chosen.includes(x))))}
            >
              {label(code)}
            </Choice>
          ))}
        </Pills>
      );
    }
    case "select":
      if (f.options.length <= PILLS_UP_TO) {
        return (
          <Pills label={w.label} hint={w.hint} error={error} required={f.required} labelAside={aside}>
            {f.options.map((code) => (
              <Choice key={code} name={f.id} checked={text === code} onChange={() => onChange(code)}>
                {label(code)}
              </Choice>
            ))}
          </Pills>
        );
      }
      return (
        <Select
          label={w.label}
          hint={w.hint}
          error={error}
          required={f.required}
          labelAside={aside}
          options={[["", es.facilities.select], ...f.options.map((c): [string, string] => [c, label(c)])]}
          value={text}
          onChange={(e) => onChange(e.target.value)}
        />
      );
    case "number":
    case "money":
    case "year":
      return (
        <TextInput
          label={w.label}
          hint={w.hint}
          error={error}
          required={f.required}
          labelAside={aside}
          inputMode="numeric"
          prefix={f.kind === "money" ? "$" : undefined}
          value={text}
          // a number travels as a number; what is not one stays as typed and Rust says what is wrong
          onChange={(e) => onChange(/^\d+$/.test(e.target.value.trim()) ? Number(e.target.value.trim()) : e.target.value)}
        />
      );
    default:
      return (
        <TextInput
          label={w.label}
          hint={w.hint}
          error={error}
          required={f.required}
          labelAside={aside}
          inputMode={f.kind === "email" ? "email" : f.kind === "phone" ? "tel" : undefined}
          type={f.kind === "date" ? "date" : "text"}
          value={text}
          onChange={(e) => onChange(e.target.value)}
        />
      );
  }
}

/** A group of pills with the label, help and error of every field. */
function Pills({ label, hint, error, required, labelAside, children }: { label: string; hint?: string; error?: string; required: boolean; labelAside?: ReactNode; children: ReactNode }) {
  // named by its words alone, so the «?» does not end up in the name of the group
  const labelId = useId();
  return (
    <fieldset className="min-w-0" aria-labelledby={labelId}>
      <legend className="field-label">
        <span id={labelId}>
          {label}
          {required && <span className="req">*</span>}
        </span>
        {labelAside}
      </legend>
      <div className="flex flex-wrap gap-2">{children}</div>
      {hint && <span className="field-hint">{hint}</span>}
      {error && (
        <span role="alert" className="field-error">
          <Icon name="alert" size={15} />
          {error}
        </span>
      )}
    </fieldset>
  );
}
