import { Alert, Tag, TextButton } from "./ui";
import { es } from "../i18n/es-MX";

type Issue = { code: string; field: string; blocking: boolean };
const t = es.common.review;

/**
 * What a record asks to review, up front and in words: each one with the step it is in and a button that takes the
 * person there. «Cosas por revisar» in a list would mean nothing if opening the record did not say what they are.
 */
export function IssueSummary({
  issues,
  describe,
  stepOf,
  stepName,
  onGo,
}: {
  issues: Issue[];
  describe: (code: string) => string;
  stepOf: (field: string) => number;
  stepName: (step: number) => string;
  onGo: (step: number) => void;
}) {
  const group = (list: Issue[], tone: "error" | "warn", title: string) =>
    list.length > 0 && (
      <Alert tone={tone}>
        <b className="block font-bold">{title}</b>
        <ul className="mt-2 flex flex-col gap-2">
          {list.map((i, n) => (
            <li key={`${i.code}-${i.field}-${n}`} className="flex flex-wrap items-center gap-x-3 gap-y-1">
              <span className="min-w-0 flex-1 text-ui">{describe(i.code)}</span>
              <Tag variant="line">{stepName(stepOf(i.field))}</Tag>
              <TextButton onClick={() => onGo(stepOf(i.field))}>{t.go}</TextButton>
            </li>
          ))}
        </ul>
      </Alert>
    );
  const blocking = issues.filter((i) => i.blocking);
  const warnings = issues.filter((i) => !i.blocking);
  return (
    <>
      {group(blocking, "error", t.mustFix(blocking.length))}
      {group(warnings, "warn", t.title(warnings.length))}
    </>
  );
}
