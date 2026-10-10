import { useQuery } from "@tanstack/react-query";
import { Icon } from "./icons";
import { Inset } from "./ui";
import { es } from "../i18n/es-MX";
import { manualEntry } from "../lib/tauri";
import type { ManualEntry } from "../lib/types";

const t = es.manual;

/** The «?» right after the label of a field: it opens what the manual says of it (ADR-034). No AI, no internet. */
export function HelpToggle({ open, onToggle, controls }: { open: boolean; onToggle: () => void; controls: string }) {
  return (
    <button
      type="button"
      aria-label={t.about}
      title={t.about}
      aria-expanded={open}
      aria-controls={controls}
      // inside a <label>: it must not move the focus to the input it belongs to
      onClick={(e) => {
        e.preventDefault();
        onToggle();
      }}
      className={`ml-1.5 inline-flex h-5 w-5 items-center justify-center rounded-pill align-[-4px] transition-colors hover:text-ink ${open ? "text-ink" : "text-ink-3"}`}
    >
      <Icon name="help" size={17} />
    </button>
  );
}

/** «A», «B» y «C». */
const listed = (items: string[]) => (items.length < 2 ? items.join("") : `${items.slice(0, -1).join(", ")} y ${items[items.length - 1]}`);

/** The text of an entry of the manual: its paragraphs (the start «Qué poner:» in bold) and what uses the field. */
export function ManualText({ entry }: { entry: ManualEntry }) {
  return (
    <div className="space-y-2 text-ui text-ink-2">
      {entry.paragraphs.map((p) => {
        const lead = t.leads.find((l) => p.startsWith(l));
        return (
          <p key={p}>
            {lead ? (
              <>
                <span className="font-bold text-ink">{lead}</span>
                {p.slice(lead.length)}
              </>
            ) : (
              p
            )}
          </p>
        );
      })}
      {entry.used_by.length > 0 && (
        <p>
          <span className="font-bold text-ink">{t.usedBy}</span> {listed(entry.used_by.map((c) => t.who[c] ?? c))}.
        </p>
      )}
    </div>
  );
}

/** What the manual says of a field, under it. It is in the program: it answers at once and costs nothing. */
export function ManualPanel({ id, panelId }: { id: string; panelId: string }) {
  const entry = useQuery({ queryKey: ["manual", id], queryFn: () => manualEntry(id), staleTime: Infinity });
  if (entry.isPending) return null;
  return (
    <Inset className="mt-3">
      <div id={panelId} role="region" aria-label={entry.data?.title ?? t.missing}>
        {entry.data ? <ManualText entry={entry.data} /> : <p className="text-ui text-ink-2">{t.missing}</p>}
        <p className="mt-3 text-small text-ink-3">{t.offline}</p>
      </div>
    </Inset>
  );
}
