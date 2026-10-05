import { useQuery } from "@tanstack/react-query";
import { Icon } from "../../components/icons";
import { Alert, Button } from "../../components/ui";
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
  if (!review) return <p className="px-6 py-6 text-stone-700">{es.common.loading}</p>;
  const by = (level: CheckLevel): ReviewCheck[] => review.report.checks.filter((c) => c.level === level);
  const counts: [string, number, string][] = [
    [t.counts.errors, review.report.errors, "text-red-800 bg-red-50"],
    [t.counts.warnings, review.report.warnings, "text-amber-800 bg-amber-50"],
    [t.counts.info, by("info").length, "text-blue-800 bg-blue-50"],
  ];
  return (
    <div className="flex h-full min-h-0 flex-col">
      <header className="space-y-1 px-6 pb-4 pt-6">
        <p className="text-[12px] font-medium uppercase tracking-[0.08em] text-stone-600">{t.tab}</p>
        <p className="text-[13px] text-stone-700">{t.intro}</p>
      </header>
      <div className="min-h-0 flex-1 space-y-5 overflow-y-auto px-6 pb-6">
        <div className="grid grid-cols-3 gap-2.5">
          {counts.map(([label, n, tone]) => (
            <div key={label} className={`rounded-xl px-3.5 py-3 ${tone}`}>
              <p className="text-[22px] font-semibold leading-none">{n}</p>
              <p className="mt-1.5 text-[12px] font-semibold">{label}</p>
            </div>
          ))}
        </div>
        {review.report.errors === 0 && <Alert tone="ok">{t.allGood}</Alert>}
        {groups.map(([level, title, tone]) =>
          by(level).length > 0 ? (
            <section key={level} className="space-y-2">
              <h3 className="text-[14px] font-semibold">{title}</h3>
              <ul className="space-y-2">
                {by(level).map((c, i) => (
                  <li key={`${c.code}-${i}`}>
                    <Alert tone={tone}>
                      <p>{c.text}</p>
                      {level === "error" && c.target && <p className="mt-1 text-[13px] text-stone-700">{t.fixHere}</p>}
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
      <p className="text-[12px] font-medium uppercase tracking-[0.08em] text-stone-600">{t.nextStep}</p>
      <Button variant="primary" className="w-full" onClick={onContinue} disabled={!clean || busy || checking}>
        {t.toReady}
        <Icon name="next" size={15} />
      </Button>
      <div className="grid grid-cols-2 gap-2">
        <Button size="sm" onClick={() => void review.refetch()} disabled={checking || busy}>
          {t.recheck}
        </Button>
        <Button size="sm" variant="ghost" onClick={onBack} disabled={busy}>
          <Icon name="back" size={15} />
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
