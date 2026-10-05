import { describeCounts, es } from "../i18n/es-MX";
import type { QuarantineReport } from "../lib/types";
import { Alert, Button, Modal } from "./ui";

const q = es.quarantine;

/** Friendly name of the place where something was found (e.g. a table row of the profile). */
function fieldName(path: string): string {
  if (q.fieldNames[path]) return q.fieldNames[path];
  if (path.startsWith("population")) return es.profile.sections.population;
  if (path.startsWith("staff")) return es.profile.sections.staff;
  if (path.startsWith("facilities")) return es.profile.sections.facilities;
  if (path.startsWith("income")) return es.profile.sections.income;
  return es.profile.sections.general;
}

export function QuarantineDialog({
  report,
  busy,
  onRedact,
  onNotPersonal,
  onCancel,
}: {
  report: QuarantineReport;
  busy?: boolean;
  onRedact: () => void;
  onNotPersonal: () => void;
  onCancel: () => void;
}) {
  return (
    <Modal title={q.title} onClose={onCancel}>
      <p className="text-[15px]">{q.intro}</p>
      <p className="text-[15px] font-semibold">{q.summary(describeCounts(report.counts))}</p>

      <div>
        <h3 className="font-semibold">{q.where}</h3>
        <ul className="mt-2 space-y-3">
          {report.fields.map((f) => (
            <li key={f.path} className="rounded-lg border border-stone-200 p-3">
              <p className="font-semibold">
                {fieldName(f.path)} · {describeCounts(f.counts)}
              </p>
              <p className="mt-1 text-[13px] text-stone-700">{q.preview}</p>
              <p className="mt-1 whitespace-pre-wrap rounded bg-stone-100 p-2">{f.redacted_preview}</p>
            </li>
          ))}
        </ul>
      </div>

      {report.has_blocking && <Alert tone="warn">{q.blockingHelp}</Alert>}

      <div className="flex flex-wrap gap-3 pt-2">
        <Button variant="primary" onClick={onRedact} disabled={busy}>
          {q.redact}
        </Button>
        <Button onClick={onCancel} disabled={busy}>
          {q.cancel}
        </Button>
      </div>

      {!report.has_blocking && (
        <div className="border-t border-stone-200 pt-4">
          <Button onClick={onNotPersonal} disabled={busy}>
            {q.notPersonal}
          </Button>
          <p className="mt-1 text-[14px] text-stone-700">{q.notPersonalHelp}</p>
        </div>
      )}
    </Modal>
  );
}
