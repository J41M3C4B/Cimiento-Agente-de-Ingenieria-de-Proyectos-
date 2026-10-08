import { useState } from "react";
import type { ReactNode } from "react";
import { Icon } from "../../components/icons";
import type { IconName } from "../../components/icons";
import { Alert, Button, Inset, Logo, Steps, Tag, Tile } from "../../components/ui";
import { BrandArt } from "./BrandArt";
import type { Tone } from "../../components/ui";
import { es } from "../../i18n/es-MX";
import type { OnboardingStatus } from "./api";

const o = es.onboarding;

const POINT_ICONS: IconName[] = ["check", "lock", "smile"];

/**
 * The frame of the first start, split in two: on the left the flat blue of the brand with the logo, a picture and
 * what the program promises; on the right the forms, with the steps at the top, the content that scrolls and the
 * buttons always at the bottom. On a narrow window the blue shrinks to a band with the logo.
 */
export function StartFrame({ children, top, footer, title, text }: { children: ReactNode; /** what stays at the top while the rest scrolls: the steps */ top?: ReactNode; /** the buttons: always at the bottom */ footer: ReactNode; title?: string; text?: string }) {
  const a = o.aside;
  return (
    <div className="onb text-body">
      <aside className="onb-brand">
        <Logo onBrand className="h-10 self-start" />
        <BrandArt className="onb-art" />
        <div className="onb-copy space-y-3">
          <h2 className="text-title font-bold leading-tight tracking-tight">{title ?? a.title}</h2>
          <p className="text-body font-semibold">{text ?? a.text}</p>
        </div>
        <ul className="hidden flex-col gap-4 lg:flex">
          {a.points.map(([head, line], i) => (
            <li key={head} className="flex items-start gap-3">
              <span className="grid h-8 w-8 shrink-0 place-items-center rounded-field bg-on-brand text-brand">
                <Icon name={POINT_ICONS[i] ?? "check"} size={16} strokeWidth={2.4} />
              </span>
              <span className="min-w-0">
                <b className="block text-ui font-bold">{head}</b>
                <span className="block text-small font-semibold">{line}</span>
              </span>
            </li>
          ))}
        </ul>
      </aside>
      <main className="onb-side">
        {top && <div className="onb-in space-y-6 pt-8 lg:pt-10">{top}</div>}
        <div className="flex min-h-0 flex-1 flex-col overflow-y-auto">
          <div className={`onb-in flex flex-1 flex-col gap-6 pb-6 ${top ? "pt-6" : "pt-8 lg:pt-10"}`}>{children}</div>
        </div>
        <div className="border-t border-line">
          <div className="onb-in py-4">{footer}</div>
          <p className="onb-in pb-3 text-caption text-ink-3">{es.access.footer}</p>
        </div>
      </main>
    </div>
  );
}

const ICONS: IconName[] = ["sparkles", "shield", "help"];
const TONES: Tone[] = ["violet", "green", "sky"];

/** Three short screens, once per person: what SociAI does, how the data are cared for, and where to ask. */
export function Welcome({ onDone, busy }: { onDone: () => void; busy?: boolean }) {
  const [n, setN] = useState(0);
  const steps = o.welcome.steps;
  const last = n === steps.length - 1;
  return (
    <StartFrame
      top={<Steps steps={steps.map((x, i) => ({ key: String(i), label: x.title }))} current={n} tone={TONES[n] ?? "violet"} compact />}
      footer={
        <div className="flex items-center justify-between gap-3">
          <span className="text-small font-semibold text-ink-3">{o.stepOf(n + 1, steps.length)}</span>
          <div className="flex gap-2">
            {n > 0 && <Button onClick={() => setN(n - 1)}>{o.welcome.back}</Button>}
            <Button variant="primary" disabled={busy} onClick={() => (last ? onDone() : setN(n + 1))}>
              {last ? o.welcome.start : o.welcome.next}
            </Button>
          </div>
        </div>
      }
    >
      <div key={n} className="anim-rise my-auto space-y-6 py-6">
        <Tile icon={ICONS[n] ?? "sparkles"} tone={TONES[n] ?? "violet"} />
        <div className="space-y-3">
          <h1 className="text-title font-bold leading-tight tracking-tight">{steps[n]!.title}</h1>
          <p className="max-w-[52ch] text-body text-ink-2">{steps[n]!.text}</p>
        </div>
      </div>
    </StartFrame>
  );
}

/** One line of the setup: what it is, how it is, and a mark. */
function SetupRow({ icon, title, value, ok }: { icon: IconName; title: string; value: string; ok: boolean }) {
  return (
    <li className="flex items-center gap-4 rounded-inset bg-inset p-4">
      <Tile icon={icon} tone={ok ? "green" : "amber"} />
      <div className="min-w-0 flex-1">
        <b className="block text-ui font-bold">{title}</b>
        <span className="block text-small text-ink-2">{value}</span>
      </div>
      <Tag tone={ok ? "green" : "amber"} variant="soft" icon={ok ? "check" : "warn"}>
        {ok ? o.setup.ready : o.setup.pending}
      </Tag>
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
    { icon: "building" as IconName, title: o.setup.data, value: pending === 0 ? o.setup.aiReady : o.setup.dataMissing(pending), ok: pending === 0 },
  ];
  const done = rows.filter((r) => r.ok).length;
  return (
    <StartFrame
      footer={
        <div className="flex flex-wrap justify-end gap-2">
          <Button onClick={onLater}>{o.setup.later}</Button>
          <Button variant="primary" onClick={onFill}>
            {o.setup.fillNow}
          </Button>
        </div>
      }
    >
      <div className="space-y-6">
        <div className="flex items-start gap-4">
          <Tile icon="sliders" tone="ink" />
          <div className="min-w-0 flex-1 space-y-1.5">
            <h1 className="text-title font-bold leading-tight tracking-tight">{o.setup.title}</h1>
            <p className="max-w-[60ch] text-ui text-ink-2">{o.setup.help}</p>
          </div>
        </div>
        <div className="flex items-center gap-3">
          <span className="text-small font-semibold text-ink-2">{o.setup.stepsDone(done, rows.length)}</span>
          <div className="min-w-0 flex-1">
            <Steps steps={rows.map((r) => ({ key: r.title, label: r.title }))} current={done - 1} tone="green" compact />
          </div>
        </div>
        <ul className="space-y-3">
          {rows.map((r) => (
            <SetupRow key={r.title} icon={r.icon} title={r.title} value={r.value} ok={r.ok} />
          ))}
        </ul>
        {error && <Alert tone="error">{error}</Alert>}
        {examples && <Inset className="!p-4">{examples}</Inset>}
      </div>
    </StartFrame>
  );
}
