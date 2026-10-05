import { useState } from "react";
import { QuarantineDialog } from "../../components/QuarantineDialog";
import { Icon } from "../../components/icons";
import { Alert, Button, Chip, Modal, RowActions, Select, TextInput } from "../../components/ui";
import { es } from "../../i18n/es-MX";
import { scheduleConfirm, scheduleDeleteActivity, scheduleSaveActivity, toAppError } from "../../lib/tauri";
import type { ActivityView, Decision, DraftingView, QuarantineReport } from "../../lib/types";

const t = es.drafting;

interface Pending {
  id: string | null;
  title: string;
  start: number;
  end: number;
  report: QuarantineReport;
}

/** The months at a glance: one bar per activity, from its first month to its last, with the months numbered. */
function Gantt({ activities, total }: { activities: ActivityView[]; total: number }) {
  return (
    <div className="overflow-x-auto rounded-xl border border-stone-200 p-4">
      <div className="min-w-[560px] space-y-2">
        <div className="grid grid-cols-[minmax(8rem,14rem)_1fr] items-end gap-3">
          <span />
          <div className="grid text-center text-[11px] font-medium text-stone-600" style={{ gridTemplateColumns: `repeat(${total}, minmax(0, 1fr))` }} aria-hidden>
            {Array.from({ length: total }, (_, i) => (
              <span key={i}>{i + 1}</span>
            ))}
          </div>
        </div>
        {activities.map((a) => (
          <div key={a.id} className="grid grid-cols-[minmax(8rem,14rem)_1fr] items-center gap-3">
            <span className="truncate text-[13px] font-medium">{a.title}</span>
            <div className="relative h-5 rounded-md bg-stone-100" aria-hidden>
              <span
                className={`absolute top-0 h-5 rounded-md ${a.origin === "ai_assumption" ? "bg-blue-300" : "bg-blue-800"}`}
                style={{ left: `${((a.start_month - 1) / total) * 100}%`, width: `${((a.end_month - a.start_month + 1) / total) * 100}%` }}
              />
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}

/**
 * The schedule: the assistant proposes the activities and their months; the person adjusts them. The program
 * validates the months and the duration. The bars show it all at once; each activity can be changed in a window.
 */
export function ScheduleEditor({ view, onView, disabled }: { view: DraftingView; onView: (v: DraftingView) => void; disabled: boolean }) {
  const project = view.project.id;
  const [editing, setEditing] = useState<string | "new" | null>(null);
  const [title, setTitle] = useState("");
  const [start, setStart] = useState("1");
  const [end, setEnd] = useState("1");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [pending, setPending] = useState<Pending | null>(null);
  const working = busy || disabled;
  const { activities, duration_months, confirmed } = view.schedule;
  // the call may limit how long the project lasts; the months to choose from go a little beyond what is planned
  const limit = view.requirements.max_duration_months?.value ?? 24;
  const monthOptions: [string, string][] = Array.from({ length: Math.max(limit, duration_months, 1) }, (_, i) => [String(i + 1), String(i + 1)]);

  async function guarded(fn: () => Promise<void>) {
    setBusy(true);
    setError(null);
    try {
      await fn();
    } catch (e) {
      setPending(null);
      setError(toAppError(e).message);
    } finally {
      setBusy(false);
    }
  }

  const save = (id: string | null, text: string, from: number, to: number, decision?: Decision) =>
    guarded(async () => {
      const out = await scheduleSaveActivity(project, id, text, from, to, decision);
      if (out.status === "quarantine") return setPending({ id, title: text, start: from, end: to, report: out.report });
      setPending(null);
      setEditing(null);
      setTitle("");
      onView(out.view);
    });

  const open = (a: ActivityView | null) => {
    setEditing(a ? a.id : "new");
    setTitle(a?.title ?? "");
    setStart(String(a?.start_month ?? 1));
    setEnd(String(a?.end_month ?? 1));
  };
  const canSave = title.trim() !== "" && Number(end) >= Number(start);
  const close = () => setEditing(null);

  return (
    <div className="space-y-5">
      <div className="flex flex-wrap items-end justify-between gap-3">
        <p className="max-w-2xl text-stone-700">{t.scheduleIntro}</p>
        <Button size="sm" variant={activities.length === 0 ? "primary" : "secondary"} onClick={() => open(null)} disabled={working}>
          <Icon name="plus" size={15} strokeWidth={2.4} />
          {t.addActivity}
        </Button>
      </div>

      {activities.length === 0 ? (
        <div className="rounded-xl border border-dashed border-stone-300 px-6 py-10 text-center text-stone-700">{t.emptySchedule}</div>
      ) : (
        <>
          <Gantt activities={activities} total={Math.max(duration_months, 1)} />
          <ul className="divide-y divide-stone-200 rounded-xl border border-stone-200">
            {activities.map((a) => (
              <li key={a.id} className="group flex flex-wrap items-center gap-3 px-4 py-2.5">
                <p className="min-w-0 flex-1 font-medium">
                  {a.title}
                  {a.origin === "ai_assumption" && (
                    <span className="ml-2 align-middle">
                      <Chip tone="blue">{t.proposed}</Chip>
                    </span>
                  )}
                </p>
                <p className="text-[13px] text-stone-700">{t.months(a.start_month, a.end_month)}</p>
                <RowActions onEdit={() => open(a)} onRemove={() => guarded(async () => onView(await scheduleDeleteActivity(project, a.id)))} busy={working} />
              </li>
            ))}
          </ul>
        </>
      )}

      {error && <Alert tone="warn">{error}</Alert>}
      {activities.length > 0 && <p className="font-semibold">{t.projectLasts(duration_months)}</p>}
      {activities.length > 0 &&
        (confirmed ? (
          <Alert tone="ok">{t.scheduleConfirmed}</Alert>
        ) : (
          <Button variant="primary" disabled={working} onClick={() => guarded(async () => onView(await scheduleConfirm(project)))}>
            {t.confirmSchedule}
          </Button>
        ))}

      {editing !== null && (
        <Modal
          title={editing === "new" ? t.addActivity : t.editActivity}
          onClose={close}
          footer={
            <>
              <Button onClick={close} disabled={working}>
                {t.cancel}
              </Button>
              <Button type="submit" form="activity" variant="primary" disabled={working || !canSave}>
                {t.saveActivity}
              </Button>
            </>
          }
        >
          <form
            id="activity"
            className="space-y-4"
            onSubmit={(e) => {
              e.preventDefault();
              if (canSave) void save(editing === "new" ? null : editing, title, Number(start), Number(end));
            }}
          >
            <TextInput label={t.activity.title} autoFocus value={title} onChange={(e) => setTitle(e.target.value)} />
            <div className="grid gap-3 sm:grid-cols-2">
              <Select label={t.activity.from} options={monthOptions} value={start} onChange={(e) => setStart(e.target.value)} />
              <Select label={t.activity.to} options={monthOptions} value={end} onChange={(e) => setEnd(e.target.value)} />
            </div>
            {Number(end) < Number(start) && <Alert tone="warn">{t.monthsOrder}</Alert>}
          </form>
        </Modal>
      )}

      {pending && (
        <QuarantineDialog
          report={pending.report}
          busy={busy}
          onRedact={() => save(pending.id, pending.title, pending.start, pending.end, "redact")}
          onNotPersonal={() => save(pending.id, pending.title, pending.start, pending.end, "not_personal")}
          onCancel={() => setPending(null)}
        />
      )}
    </div>
  );
}
