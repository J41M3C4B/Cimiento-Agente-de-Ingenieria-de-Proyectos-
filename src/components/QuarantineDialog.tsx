import { describeCounts, es } from "../i18n/es-MX";
import type { QuarantineReport } from "../lib/types";
import { Alert, Button, Inset, Modal } from "./ui";

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
    <Modal
      title={q.title}
      onClose={onCancel}
      footer={
        <>
          <Button onClick={onCancel} disabled={busy}>
            {q.cancel}
          </Button>
          <Button variant="primary" onClick={onRedact} disabled={busy}>
            {q.redact}
          </Button>
        </>
      }
    >
      <div className="space-y-2">
        <p className="text-body">{q.intro}</p>
        <p className="text-body font-bold">{q.summary(describeCounts(report.counts))}</p>
      </div>

      <div>
        <h3 className="text-ui font-bold">{q.where}</h3>
        <ul className="mt-3 space-y-3">
          {report.fields.map((f) => (
            <li key={f.path}>
              <Inset className="space-y-1">
                <p className="text-ui font-bold">
                  {fieldName(f.path)} · {describeCounts(f.counts)}
                </p>
                <p className="text-small text-ink-3">{q.preview}</p>
                <p className="whitespace-pre-wrap rounded-field bg-card p-3 text-ui">{f.redacted_preview}</p>
              </Inset>
            </li>
          ))}
        </ul>
      </div>

      {report.has_blocking && <Alert tone="warn">{q.blockingHelp}</Alert>}

      {!report.has_blocking && (
        <div className="space-y-2 border-t border-line pt-4">
          <Button onClick={onNotPersonal} disabled={busy}>
            {q.notPersonal}
          </Button>
          <p className="text-small text-ink-3">{q.notPersonalHelp}</p>
        </div>
      )}
    </Modal>
  );
}
