import { useState } from "react";
import type { ReactNode } from "react";
import { Icon } from "../../components/icons";
import type { IconName } from "../../components/icons";
import { Alert, Button, Logo, Status } from "../../components/ui";
import { HelpPreview, ModuleBubbles, PrivacyFlow } from "./WelcomeArt";
import { es } from "../../i18n/es-MX";
import type { OnboardingStatus } from "./api";

const o = es.onboarding;

const POINT_ICONS: IconName[] = ["link", "trendingUp", "sparkles"];

/** What the blue panel says: always a big title, one line of information and a few facts with their icon. */
export type Aside = { title: string; soft?: string; text: string; points: [string, string, IconName][] };

const FACT_ICONS: IconName[] = ["check", "lock"];
const withIcons = (facts: [string, string][], icons: IconName[] = FACT_ICONS): Aside["points"] => facts.map(([head, line], i) => [head, line, icons[i] ?? "check"]);

/** The welcome says what SociAI is. */
const PITCH: Aside = { title: o.aside.title, soft: o.aside.soft, text: o.aside.text, points: withIcons(o.aside.points, POINT_ICONS) };

/** The data steps say what is being done in this one, in the same shape as the welcome. */
export const asideOf = (key: string): Aside => ({ ...(o.about[key] ?? o.about.institution!), points: withIcons(o.facts) });

function AsideBody({ aside }: { aside: Aside }) {
  return (
    <div className="onb-aside">
      <h2 className="onb-aside-title">
        <Hero title={aside.title} highlight={aside.soft ?? ""} className="onb-aside-soft" />
      </h2>
      <p className="onb-aside-lead">{aside.text}</p>
      <ul className="onb-points">
        {aside.points.map(([head, line, icon]) => (
          <li key={head} className="onb-point">
            <span className="onb-point-icon">
              <Icon name={icon} size={20} strokeWidth={2} />
            </span>
            <span className="onb-point-text min-w-0">
              <b>{head}</b>
              <span>{line}</span>
            </span>
          </li>
        ))}
      </ul>
    </div>
  );
}

/**
 * The frame of the first start, split in two (docs/13 §6). On the left the flat blue of the brand with the logo and
 * what it says: the pitch in the welcome, and in the other screens why the step asks what it asks. On the right four
 * fixed zones: the progress (A), the title at one height (B, inside the content), the content that scrolls (C) and the
 * buttons (D). On a narrow window the blue shrinks to a band with the logo.
 */
export function StartFrame({ children, top, footer, aside = PITCH }: { children: ReactNode; /** zone A: what tells where the person is */ top?: ReactNode; /** zone D: the buttons */ footer: ReactNode; aside?: Aside }) {
  return (
    <div className="onb text-body">
      <aside className="onb-brand">
        <Logo onBrand className="onb-logo h-10" />
        <span aria-hidden="true" className="onb-art" />
        <AsideBody aside={aside} />
      </aside>
      <main className="onb-side">
        <div className="onb-top">
          <div className="onb-in">{top}</div>
        </div>
        <div className="onb-scroll">
          <div className="onb-in">{children}</div>
        </div>
        <div className="onb-foot">
          <div className="onb-in">{footer}</div>
        </div>
      </main>
    </div>
  );
}

/** The title in the navy of the brand; the name of the program, whole and heavier, in the blue of the brand. */
function Hero({ title, highlight, className = "font-extrabold text-brand" }: { title: string; highlight: string; className?: string }) {
  const at = highlight ? title.indexOf(highlight) : -1;
  if (at < 0) return <>{title}</>;
  return (
    <>
      {title.slice(0, at)}
      <span className={className}>{highlight}</span>
      {title.slice(at + highlight.length)}
    </>
  );
}

const ART = [ModuleBubbles, PrivacyFlow, HelpPreview];

/**
 * Three screens, once per person: what SociAI is (the institution and its modules, each in its own color), how the data
 * are cared for (what stays here and what the help sees) and where to ask (the page of «Ayuda»). Each one is a title
 * at the same height, a line that says it and a drawing in the rest of the space.
 */
export function Welcome({ onDone, busy }: { onDone: () => void; busy?: boolean }) {
  const [n, setN] = useState(0);
  const steps = o.welcome.steps;
  const step = steps[n]!;
  const last = n === steps.length - 1;
  const Art = ART[n] ?? ModuleBubbles;
  return (
    <StartFrame
      top={
        <ol className="onb-bars" aria-label={o.stepOf(n + 1, steps.length)}>
          {steps.map((s, i) => (
            <i key={s.title} aria-current={i === n ? "step" : undefined} className={i <= n ? "on" : ""} />
          ))}
        </ol>
      }
      footer={
        <div className="flex items-center justify-between gap-3">
          {n > 0 ? <Button onClick={() => setN(n - 1)}>{o.welcome.back}</Button> : <span />}
          <Button variant="primary" disabled={busy} onClick={() => (last ? onDone() : setN(n + 1))}>
            {last ? o.welcome.start : o.welcome.next}
          </Button>
        </div>
      }
    >
      <div key={n} className="anim-rise onb-welcome">
        <div className="onb-intro">
          <h1 className="onb-hero break-words text-headline font-bold tracking-tight">
            <Hero title={step.title} highlight={step.highlight} />
          </h1>
          <p className="onb-lead">{step.text}</p>
        </div>
        <div className="onb-stage">
          <Art />
        </div>
      </div>
    </StartFrame>
  );
}

/** One line of the setup: what it is, how it is, and its state at the right. */
function SetupRow({ icon, title, value, ok }: { icon: IconName; title: string; value: string; ok: boolean }) {
  return (
    <li className="onb-task">
      <span aria-hidden="true" className="onb-task-icon">
        <Icon name={icon} size={20} />
      </span>
      <div className="min-w-0 flex-1">
        <b>{title}</b>
        <span className="onb-task-detail">{value}</span>
      </div>
      <Status kind={ok ? "ok" : "pending"}>{ok ? o.setup.ready : o.setup.pending}</Status>
    </li>
  );
}

/**
 * What the administrator sees while the institution has not finished its data: the setup of the computer and the
 * choice of filling the data now or leaving them to the direction (ADR-031).
 */
export function Setup({
  status, onFill, onLater, examples, error,
}: {
  status: OnboardingStatus;
  onFill: () => void;
  onLater: () => void;
  examples?: ReactNode;
  error?: string | null;
}) {
  const setup = status.setup;
  const pending = status.steps.filter((s) => !s.complete).length;
  const rows = [
    ...(setup ? [{ icon: "sparkles" as IconName, title: o.setup.ai, value: setup.ai_ready ? o.setup.aiReady : o.setup.aiMissing, ok: setup.ai_ready }] : []),
    ...(setup ? [{ icon: "users" as IconName, title: o.setup.accounts, value: o.setup.accountsCount(setup.managers), ok: setup.managers > 0 }] : []),
    { icon: "building" as IconName, title: o.setup.data, value: pending === 0 ? o.setup.dataDone : o.setup.dataMissing(pending), ok: pending === 0 },
  ];
  return (
    <StartFrame
      aside={{ ...o.setup.about, points: withIcons(o.setup.facts, ["users", "lock"]) }}
      footer={
        <div className="flex flex-wrap items-center justify-between gap-3">
          <Button onClick={onLater}>{o.setup.later}</Button>
          <Button variant="primary" onClick={onFill}>
            {o.setup.fillNow}
          </Button>
        </div>
      }
    >
      <div className="space-y-6">
        <div className="space-y-2">
          <h1 className="onb-title text-title font-bold leading-tight tracking-tight">{o.setup.title}</h1>
          <p className="onb-lead">{o.setup.help}</p>
        </div>
        <ul className="onb-tasks">
          {rows.map((r) => (
            <SetupRow key={r.title} icon={r.icon} title={r.title} value={r.value} ok={r.ok} />
          ))}
        </ul>
        <p className="onb-narrow-only max-w-[52ch] text-ui text-ink-2">
          <b className="font-bold text-ink">{o.setup.facts[0]![0]}.</b> {o.setup.facts[0]![1]}
        </p>
        {error && <Alert tone="error">{error}</Alert>}
        {examples && <div className="onb-dev">{examples}</div>}
      </div>
    </StartFrame>
  );
}
