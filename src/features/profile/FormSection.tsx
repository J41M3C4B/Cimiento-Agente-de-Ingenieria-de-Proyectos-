import type { ReactNode } from "react";
import { Eyebrow } from "../../components/ui";

/** A group of fields in a window: a small heading in capitals with a hairline, then the fields (docs/13 §7, «Formulario»). */
export function FormSection({ title, children }: { title: string; children: ReactNode }) {
  return (
    <section aria-label={title} className="flex flex-col gap-3">
      <div aria-hidden="true" className="flex items-center gap-3">
        <Eyebrow>{title}</Eyebrow>
        <span className="h-px flex-1 bg-line" />
      </div>
      {children}
    </section>
  );
}
