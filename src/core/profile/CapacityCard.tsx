import { Button, Card, Inset, TextButton } from "../../components/ui";
import { es } from "../../i18n/es-MX";
import type { ProfileView } from "../../lib/types";

const t = es.profile;
const c = t.finance.capacity;
const count = (n: number) => n.toLocaleString("es-MX");

/**
 * Capacity: how many people fit, which is something the institution says about itself. The people served, the staff
 * and the money are figures of their modules and are shown there (and in the tiles above), not here. It opens its own
 * window to edit.
 */
export function CapacityCard({ view, onEdit }: { view: ProfileView; onEdit: () => void }) {
  const p = view.input;
  const capacity = p.capacity_total;
  const known = capacity !== null || !!p.notes;
  return (
    <Card className="flex flex-col gap-5">
      <div className="flex flex-wrap items-center justify-between gap-3">
        <h2 className="text-heading font-bold">{c.title}</h2>
        <TextButton onClick={onEdit}>{t.edit}</TextButton>
      </div>

      {!known ? (
        <Inset className="flex flex-col items-start gap-3">
          <p className="text-ui text-ink-2">{c.empty}</p>
          <Button size="sm" variant="primary" onClick={onEdit}>
            {c.action}
          </Button>
        </Inset>
      ) : (
        <>
          <div className="flex items-baseline gap-2">
            <span className="tabular text-hero font-normal tracking-tight">{capacity !== null ? count(capacity) : "—"}</span>
            <span className="text-ui font-semibold text-ink-2">{c.fits}</span>
          </div>

          {p.notes && (
            <div className="border-t border-line pt-4">
              <div className="text-caption text-ink-3">{c.notes}</div>
              <p className="mt-0.5 whitespace-pre-line break-words text-ui font-medium">{p.notes}</p>
            </div>
          )}
        </>
      )}
    </Card>
  );
}
