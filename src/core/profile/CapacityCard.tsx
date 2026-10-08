import { Icon } from "../../components/icons";
import { Bar, Button, Card, Inset, Tile } from "../../components/ui";
import { es } from "../../i18n/es-MX";
import type { ProfileView } from "../../lib/types";

const t = es.profile;
const c = t.finance.capacity;
const count = (n: number) => n.toLocaleString("es-MX");

/**
 * Capacity and quick figures: how many people fit, how many are served today (against the capacity, with the room
 * that is left) and the two staff figures the person gave to start. They are the numbers that stand in until the
 * records are in their modules, so the ones that are only a guess say so. It opens its own window to edit.
 */
export function CapacityCard({ view, onEdit }: { view: ProfileView; onEdit: () => void }) {
  const p = view.input;
  const capacity = p.capacity_total;
  const served = p.served_estimate;
  const known = capacity !== null || served !== null || p.staff_paid_estimate !== null || p.staff_volunteer_estimate !== null || !!p.notes;
  const full = capacity && served !== null ? (served / capacity) * 100 : undefined;
  const left = capacity !== null && served !== null ? capacity - served : null;
  return (
    <Card className="flex flex-col gap-5">
      <div className="flex flex-wrap items-center justify-between gap-3">
        <h2 className="text-heading font-bold">{c.title}</h2>
        <Button size="sm" variant="secondary" onClick={onEdit}>
          <Icon name="pencil" size={16} />
          {t.edit}
        </Button>
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
          <div className="space-y-3">
            <div className="flex items-baseline gap-2">
              <span className="tabular text-hero font-normal tracking-tight">{capacity !== null ? count(capacity) : "—"}</span>
              <span className="text-ui font-semibold text-ink-2">{c.fits}</span>
            </div>
            {full !== undefined && <Bar percent={full} label={c.serving(count(served!))} tone="violet" />}
            {served !== null && (
              <p className="flex flex-wrap items-center gap-x-2 text-small text-ink-2">
                <b className="text-ink">{c.servingApprox(count(served))}</b>
                {left !== null && <span className="text-ink-3">{left >= 0 ? c.free(count(left)) : c.over}</span>}
              </p>
            )}
          </div>

          <div className="grid grid-cols-2 gap-3">
            <Inset className="flex items-center gap-3 !px-4 !py-3">
              <Tile icon="briefcase" tone="teal" small />
              <div className="min-w-0">
                <div className="tabular text-heading font-bold">{p.staff_paid_estimate !== null ? `≈ ${count(p.staff_paid_estimate)}` : "—"}</div>
                <div className="text-caption text-ink-3">{c.paid}</div>
              </div>
            </Inset>
            <Inset className="flex items-center gap-3 !px-4 !py-3">
              <Tile icon="heart" tone="teal" small />
              <div className="min-w-0">
                <div className="tabular text-heading font-bold">{p.staff_volunteer_estimate !== null ? `≈ ${count(p.staff_volunteer_estimate)}` : "—"}</div>
                <div className="text-caption text-ink-3">{c.volunteers}</div>
              </div>
            </Inset>
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
