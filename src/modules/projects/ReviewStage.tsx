import { useQuery } from "@tanstack/react-query";
import { Icon } from "../../components/icons";
import { Alert, Button, Eyebrow, Inset, StatusDot } from "../../components/ui";
import { es } from "../../i18n/es-MX";
import { reviewGet } from "../../lib/tauri";
import type { CheckLevel, ConversationView, ReviewCheck, ReviewView } from "../../lib/types";
import { Said, Thinking } from "./ChatParts";
import { ConversationChat } from "./ConversationChat";
import { Workspace } from "./Workspace";

const t = es.review;

const groups: [CheckLevel, string, "warn" | "error" | "info"][] = [
  ["error", t.errorsTitle, "error"],
  ["warn", t.warningsTitle, "warn"],
  ["info", t.infoTitle, "info"],
];

/** The result of the review in the panel: the three counts and, under them, what to fix, what to look at and what to keep in mind. */
function ReviewList({ review }: { review: ReviewView | undefined }) {
  if (!review) return <p className="py-6 text-ui text-ink-2">{es.common.loading}</p>;
  const by = (level: CheckLevel): ReviewCheck[] => review.report.checks.filter((c) => c.level === level);
  const counts: [string, number, "red" | "amber" | "neutral"][] = [
    [t.counts.errors, review.report.errors, "red"],
    [t.counts.warnings, review.report.warnings, "amber"],
    [t.counts.info, by("info").length, "neutral"],
  ];
  return (
    <div className="flex h-full min-h-0 flex-col">
      <div className="min-h-0 flex-1 space-y-6 overflow-y-auto">
        <p className="text-small text-ink-3">{t.intro}</p>
        <Inset>
          <dl className="space-y-2">
            {counts.map(([label, n, tone]) => (
              <div key={label} className="flex items-center justify-between gap-3">
                <dt className="text-ui font-semibold text-ink-2">
                  <StatusDot tone={tone}>{label}</StatusDot>
                </dt>
                <dd className="tabular text-heading font-bold">{n}</dd>
              </div>
            ))}
          </dl>
        </Inset>
        {review.report.errors === 0 && <Alert tone="ok">{t.allGood}</Alert>}
        {groups.map(([level, title, tone]) =>
          by(level).length > 0 ? (
            <section key={level} className="space-y-3">
              <h3 className="text-ui font-bold">{title}</h3>
              <ul className="space-y-2">
                {by(level).map((c, i) => (
                  <li key={`${c.code}-${i}`}>
                    <Alert tone={tone}>
                      <p>{c.text}</p>
                      {level === "error" && c.target && <p className="mt-1 text-small text-ink-2">{t.fixHere}</p>}
                    </Alert>
                  </li>
                ))}
              </ul>
            </section>
          ) : null,
        )}
      </div>
    </div>
  );
}

/**
 * The automatic review, as the next part of the same chat. The assistant says what it will check and what it found;
 * the details are in the panel, and going on (or going back to fix something) is its footer. The program does the
 * checking, not the AI.
 */
export function ReviewStage({
  view,
  onView,
  onContinue,
  onBack,
  busy,
  panelOpen,
}: {
  view: ConversationView;
  onView: (v: ConversationView) => void;
  onContinue: () => void;
  onBack: () => void;
  busy: boolean;
  panelOpen: boolean;
}) {
  const review = useQuery({ queryKey: ["review", view.project.id], queryFn: () => reviewGet(view.project.id), refetchOnMount: "always" });
  const r = review.data;
  const clean = r !== undefined && r.report.errors === 0;
  const checking = review.isFetching;

  const result = !r || checking ? null : r.report.errors > 0 ? t.resultErrors(r.report.errors) : r.report.warnings > 0 ? t.resultWarnings(r.report.warnings) : t.resultClean;

  const tail = (
    <div className="space-y-6">
      <Said active={checking}>{t.start}</Said>
      {checking && <Thinking phrases={t.thinking} />}
      {result && <Said>{result}</Said>}
    </div>
  );

  const next = (
    <div className="space-y-3">
      <Eyebrow>{t.nextStep}</Eyebrow>
      <Button variant="primary" className="w-full" onClick={onContinue} disabled={!clean || busy || checking}>
        {t.toReady}
        <Icon name="next" size={16} />
      </Button>
      <div className="flex flex-col gap-2">
        <Button size="sm" className="w-full" onClick={() => void review.refetch()} disabled={checking || busy}>
          {t.recheck}
        </Button>
        <Button size="sm" variant="ghost" className="w-full" onClick={onBack} disabled={busy}>
          <Icon name="back" size={16} />
          {t.back}
        </Button>
      </div>
    </div>
  );

  return (
    <Workspace panelOpen={panelOpen} readingId={view.project.call_reading_id} project={<ReviewList review={r} />} projectLabel={t.tab} footer={next}>
      <ConversationChat view={view} onView={onView} disabled={busy || checking} tail={tail} later={{ placeholder: t.placeholder, hint: t.hint }} />
    </Workspace>
  );
}
