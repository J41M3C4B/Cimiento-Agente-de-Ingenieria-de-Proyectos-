import { useState } from "react";
import type { ReactNode } from "react";
import { Icon } from "../../components/icons";
import type { IconName } from "../../components/icons";
import { Alert, Button, Card, Inset, Steps, Tag, Tile } from "../../components/ui";
import type { Tone } from "../../components/ui";
import { es } from "../../i18n/es-MX";
import type { OnboardingStatus } from "./api";

const o = es.onboarding;

/** The frame of the first start: the name of the program over a card. */
export function StartFrame({ children, wide }: { children: ReactNode; wide?: boolean }) {
  return (
    <div className="flex min-h-screen flex-col items-center justify-center gap-6 bg-canvas p-4 text-body text-ink">
      <div className="flex items-center gap-3">
        <span className="grid h-ctl w-ctl place-items-center rounded-field bg-ink text-on-ink">
          <Icon name="logo" size={22} />
        </span>
        <b className="text-heading font-extrabold tracking-tight">{es.app.name}</b>
      </div>
      <Card className={`w-full !p-8 ${wide ? "max-w-[860px]" : "max-w-[560px]"}`}>{children}</Card>
      <p className="text-small text-ink-3">{es.access.footer}</p>
    </div>
  );
}

const ICONS: IconName[] = ["sparkles", "shield", "help"];
const TONES: Tone[] = ["violet", "green", "sky"];

/** Three short screens, once per person: what Cimiento does, how the data are cared for, and where to ask. */
export function Welcome({ onDone, busy }: { onDone: () => void; busy?: boolean }) {
  const [n, setN] = useState(0);
  const steps = o.welcome.steps;
  const last = n === steps.length - 1;
  return (
    <StartFrame>
      <div className="space-y-8">
        <Steps steps={steps.map((x, i) => ({ key: String(i), label: x.title }))} current={n} tone={TONES[n] ?? "violet"} compact />
        <div key={n} className="anim-rise space-y-5">
          <Tile icon={ICONS[n] ?? "sparkles"} tone={TONES[n] ?? "violet"} />
          <div className="space-y-2">
            <h1 className="text-title font-bold leading-tight tracking-tight">{steps[n]!.title}</h1>
            <p className="max-w-[48ch] text-body text-ink-2">{steps[n]!.text}</p>
          </div>
        </div>
        <div className="flex items-center justify-between gap-3">
          <span className="text-small font-semibold text-ink-3">{o.stepOf(n + 1, steps.length)}</span>
          <div className="flex gap-2">
            {n > 0 && <Button onClick={() => setN(n - 1)}>{o.welcome.back}</Button>}
            <Button variant="primary" disabled={busy} onClick={() => (last ? onDone() : setN(n + 1))}>
              {last ? o.welcome.start : o.welcome.next}
            </Button>
          </div>
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
    <StartFrame>
      <div className="space-y-6">
        <div className="space-y-4">
          <Tile icon="sliders" tone="ink" />
          <div className="space-y-1.5">
            <h1 className="text-subtitle font-bold leading-tight tracking-tight">{o.setup.title}</h1>
            <p className="text-ui text-ink-2">{o.setup.help}</p>
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
        <div className="flex flex-wrap justify-end gap-2">
          <Button onClick={onLater}>{o.setup.later}</Button>
          <Button variant="primary" onClick={onFill}>
            {o.setup.fillNow}
          </Button>
        </div>
        {examples && <Inset className="!p-4">{examples}</Inset>}
      </div>
    </StartFrame>
  );
}
