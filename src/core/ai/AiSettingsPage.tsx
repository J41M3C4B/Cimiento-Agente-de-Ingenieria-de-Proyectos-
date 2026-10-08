import { useQuery, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import { Alert, Bar, Button, Card, Disclosure, Inset, RadioCard, Section, Select, StatusDot, THead, Tag, TextInput, PageHeader } from "../../components/ui";
import type { Tone } from "../../components/ui";
import { es } from "../../i18n/es-MX";
import {
  aiCheck,
  aiClearKey,
  aiModels,
  aiSetCap,
  aiSetModels,
  aiSetKey,
  aiSetProvider,
  aiStatus,
  aiUsageReport,
  toAppError,
} from "../../lib/tauri";
import type { AiCheckView, AiModelsView, AiProvider, ModelUsage, RecentCall } from "../../lib/types";

const t = es.ai;
const money = (n: number) => n.toLocaleString("es-MX", { minimumFractionDigits: 2, maximumFractionDigits: 2 });
const number = (n: number) => n.toLocaleString("es-MX");
const seconds = (ms: number) => (ms / 1000).toLocaleString("es-MX", { minimumFractionDigits: 1, maximumFractionDigits: 1 });
const PROVIDERS: AiProvider[] = ["gemini", "anthropic"];

type Level = "ok" | "warn" | "full";
const toneFor = (percent: number): Level => (percent >= 100 ? "full" : percent >= 80 ? "warn" : "ok");
/** green says all is well, yellow says it is about to run out, red says it did: always with its words */
const LEVEL: Record<Level, { tone: Tone; word: string; icon: "check" | "warn" | "alert" }> = {
  ok: { tone: "green", word: t.levelOk, icon: "check" },
  warn: { tone: "amber", word: t.levelWarn, icon: "warn" },
  full: { tone: "red", word: t.levelFull, icon: "alert" },
};

function Usage({ label, percent }: { label: string; percent: number }) {
  const l = LEVEL[toneFor(percent)];
  return (
    <div className="space-y-2">
      <div className="flex flex-wrap items-center justify-between gap-x-3 gap-y-1">
        <p className="text-ui font-bold">{label}</p>
        <Tag tone={l.tone} icon={l.icon}>
          {l.word}
        </Tag>
      </div>
      <Bar percent={Math.min(100, percent)} tone={l.tone} label={label} />
    </div>
  );
}

function failureText(kind: string | null): string {
  if (!kind) return t.failureOther;
  return t.failures[kind] ?? t.failureOther;
}

function ModelCard({ m }: { m: ModelUsage }) {
  const day = m.limit && m.limit.per_day > 0 ? (m.calls_last_day * 100) / m.limit.per_day : 0;
  const minute = m.limit && m.limit.per_minute > 0 ? (m.calls_last_minute * 100) / m.limit.per_minute : 0;
  return (
    <Inset className="space-y-4">
      <div>
        <h3 className="text-body font-bold">
          {m.tier ? t.jobs[m.tier] : t.otherModel}
          {m.is_fallback && ` ${t.backup}`}
        </h3>
        <p className="text-small text-ink-3">{m.model}</p>
      </div>
      {m.limit && m.limit.per_day > 0 ? <Usage label={t.callsToday(m.calls_last_day, m.limit.per_day)} percent={day} /> : <p className="text-ui font-bold">{t.callsTodayNoLimit(m.calls_last_day)}</p>}
      {m.limit && m.limit.per_minute > 0 && <Usage label={t.callsMinute(m.calls_last_minute, m.limit.per_minute)} percent={minute} />}
      <div className="space-y-1 text-ui text-ink-2">
        {m.avg_latency_ms !== null && <p>{t.avgTime(seconds(m.avg_latency_ms))}</p>}
        {m.p95_latency_ms !== null && m.calls_total > 1 && <p className="text-small text-ink-3">{t.slowestTime(seconds(m.p95_latency_ms))}</p>}
        {m.calls_total > 0 && <p className="text-small text-ink-3">{t.equivalentCost(money(m.cost_mxn))}</p>}
      </div>
      {m.failed_total > 0 && (
        <Tag tone="amber" icon="warn">
          {t.failedCalls(m.failed_total)}
        </Tag>
      )}
      {m.calls_total > 0 && (
        <Disclosure title={t.detailTitle}>
          <dl className="grid grid-cols-[1fr_auto] gap-x-4 gap-y-1 text-ui">
            <dt>{t.detailSent}</dt>
            <dd className="tabular font-bold">{number(m.input_tokens)}</dd>
            <dt>{t.detailReceived}</dt>
            <dd className="tabular font-bold">{number(m.output_tokens)}</dd>
            <dt>{t.detailThinking}</dt>
            <dd className="tabular font-bold">{number(m.thought_tokens)}</dd>
            <dt>{t.detailReused}</dt>
            <dd className="tabular font-bold">{number(m.cached_tokens)}</dd>
          </dl>
          <p className="mt-2 text-small text-ink-3">{t.detailUnit}</p>
        </Disclosure>
      )}
    </Inset>
  );
}

function RecentTable({ calls }: { calls: RecentCall[] }) {
  return (
    <div>
      <h3 className="text-body font-bold">{t.recentTitle}</h3>
      <div className="mt-3 overflow-x-auto">
        <table className="table min-w-[520px] text-ui">
          <THead
            columns={[
              { key: "when", title: t.recentWhen },
              { key: "what", title: t.recentWhat },
              { key: "time", title: t.recentTime },
              { key: "result", title: t.recentResult },
            ]}
          />
          <tbody>
            {calls.slice(0, 8).map((c, i) => (
              <tr key={`${c.at}-${i}`}>
                <td className="tabular">{new Date(c.at).toLocaleTimeString("es-MX")}</td>
                <td className="!font-medium">{t.tasks[c.task] ?? c.task}</td>
                <td className="tabular !font-medium">{c.latency_ms === null ? "—" : `${seconds(c.latency_ms)} s`}</td>
                <td className="!font-medium">
                  {c.ok ? (
                    <Tag tone="green" variant="soft">
                      {t.recentOk}
                    </Tag>
                  ) : (
                    <Tag tone="red">{failureText(c.error_kind)}</Tag>
                  )}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}

const OTHER = "__other__";

/** Swap the light and strong model (and the thinking depth) without rebuilding the app. */
function ModelsSection({ m, onSave }: { m: AiModelsView; onSave: (light: string, strong: string, effort: string) => void }) {
  // `null` = not touched yet: the select shows what is saved
  const [light, setLight] = useState<string | null>(null);
  const [strong, setStrong] = useState<string | null>(null);
  const [lightOther, setLightOther] = useState("");
  const [strongOther, setStrongOther] = useState("");
  const [effort, setEffort] = useState<string | null>(null);

  const options = (current: string): [string, string][] => [
    ...(m.known.includes(current) ? m.known : [current, ...m.known]).map((k): [string, string] => [k, k]),
    [OTHER, t.modelOther],
  ];
  const lightPick = light ?? m.light;
  const strongPick = strong ?? m.strong;
  const lightName = lightPick === OTHER ? lightOther.trim() : lightPick;
  const strongName = strongPick === OTHER ? strongOther.trim() : strongPick;
  const effortPick = effort ?? m.effort;
  const changed = lightName !== m.light || strongName !== m.strong || effortPick !== m.effort;
  const incomplete = !lightName || !strongName;

  return (
    <Section title={t.modelsTitle} help={t.modelsHelp}>
      <div className="grid gap-4">
        <div className="space-y-2">
          <Select label={t.modelLight} value={lightPick} options={options(m.light)} onChange={(e) => setLight(e.target.value)} />
          {lightPick === OTHER && (
            <TextInput label={t.modelOtherLabel} value={lightOther} onChange={(e) => setLightOther(e.target.value)} />
          )}
        </div>
        <div className="space-y-2">
          <Select label={t.modelStrong} value={strongPick} options={options(m.strong)} onChange={(e) => setStrong(e.target.value)} />
          {strongPick === OTHER && (
            <TextInput label={t.modelOtherLabel} value={strongOther} onChange={(e) => setStrongOther(e.target.value)} />
          )}
        </div>
      </div>
      <Select label={t.effortLabel} value={effortPick} options={t.effortOptions} onChange={(e) => setEffort(e.target.value)} />
      <p className="text-small text-ink-3">{t.chainNow(m.strong_chain.join(" → "))}</p>
      <Button
        variant="primary"
        disabled={!changed || incomplete}
        onClick={() => {
          onSave(lightName, strongName, effortPick);
          setLight(null);
          setStrong(null);
          setEffort(null);
          setLightOther("");
          setStrongOther("");
        }}
      >
        {t.modelsSave}
      </Button>
    </Section>
  );
}

export function AiSettingsPage() {
  const qc = useQueryClient();
  const status = useQuery({ queryKey: ["ai-status"], queryFn: aiStatus });
  const models = useQuery({ queryKey: ["ai-models"], queryFn: aiModels });
  // refreshed while the page is open, so the bars follow what happens in the other screens
  const usage = useQuery({ queryKey: ["ai-usage"], queryFn: aiUsageReport, refetchInterval: 5000 });
  const [key, setKey] = useState("");
  const [cap, setCap] = useState("");
  const [notice, setNotice] = useState<{ tone: "ok" | "error"; text: string } | null>(null);
  const [check, setCheck] = useState<AiCheckView | "running" | null>(null);

  async function refresh() {
    await qc.invalidateQueries({ queryKey: ["ai-status"] });
    await qc.invalidateQueries({ queryKey: ["ai-models"] });
    await qc.invalidateQueries({ queryKey: ["ai-usage"] });
  }

  async function run(action: () => Promise<unknown>, doneText: string) {
    setNotice(null);
    try {
      await action();
      setNotice({ tone: "ok", text: doneText });
      await refresh();
    } catch (e) {
      setNotice({ tone: "error", text: toAppError(e).message });
    }
  }

  async function runCheck() {
    setCheck("running");
    setNotice(null);
    try {
      setCheck(await aiCheck());
    } catch (e) {
      setCheck(null);
      setNotice({ tone: "error", text: toAppError(e).message });
    }
  }

  const s = status.data;
  const u = usage.data;
  const provider = s?.provider;
  const level: Level = s ? (s.paused ? "full" : s.near_cap ? "warn" : "ok") : "ok";
  return (
    <div className="space-y-6">
      <PageHeader title={t.title} intro={t.intro} />

      {notice && <Alert tone={notice.tone}>{notice.text}</Alert>}

      {s && (
        <Card className="space-y-4">
          <div className="flex flex-wrap items-center justify-between gap-x-4 gap-y-2">
            <h2 className="text-heading font-bold">{t.spent(money(s.spent_mxn), money(s.cap_mxn))}</h2>
            <Tag tone={LEVEL[level].tone} icon={LEVEL[level].icon}>
              {LEVEL[level].word}
            </Tag>
          </div>
          <Bar percent={s.percent} tone={LEVEL[level].tone} label={t.spent(money(s.spent_mxn), money(s.cap_mxn))} />
          {s.paused && <Alert tone="warn">{t.paused}</Alert>}
          {!s.paused && s.near_cap && <Alert tone="warn">{t.nearCap}</Alert>}
        </Card>
      )}

      <div className="grid items-start gap-4 min-[1000px]:grid-cols-2">
        <div className="space-y-4">
          <Section title={t.providerTitle} help={t.providerHelp}>
            <fieldset className="space-y-2">
              <legend className="sr-only">{t.providerTitle}</legend>
              {PROVIDERS.map((p) => (
                <RadioCard
                  key={p}
                  name="provider"
                  title={t.providers[p]!}
                  checked={provider === p}
                  onChange={() => {
                    setCheck(null);
                    void run(() => aiSetProvider(p), t.providerChanged);
                  }}
                />
              ))}
            </fieldset>
          </Section>

          {models.data && (
            <ModelsSection
              key={`${models.data.provider}/${models.data.light}/${models.data.strong}/${models.data.effort}`}
              m={models.data}
              onSave={(light, strong, effort) => {
                setCheck(null);
                void run(() => aiSetModels(light, strong, effort), t.modelsSaved);
              }}
            />
          )}
        </div>

        <div className="space-y-4">
          <Section title={t.keyTitle} help={t.keyHelp}>
            <p className="text-ui font-bold">
              <StatusDot tone={s?.has_key ? "green" : "amber"}>{s?.has_key ? t.keySaved : t.keyMissing}</StatusDot>
            </p>
            <form
              className="space-y-4"
              onSubmit={(e) => {
                e.preventDefault();
                if (!provider) return;
                void run(async () => {
                  await aiSetKey(provider, key);
                  setKey("");
                  setCheck(null);
                }, es.common.saved);
              }}
            >
              <TextInput label={t.keyLabel} type="password" autoComplete="off" value={key} onChange={(e) => setKey(e.target.value)} />
              <div className="flex flex-wrap gap-3">
                <Button type="submit" variant="primary" disabled={!key.trim() || !provider}>
                  {t.keySave}
                </Button>
                {s?.has_key && provider && <Button onClick={() => run(() => aiClearKey(provider), es.common.saved)}>{t.keyRemove}</Button>}
              </div>
            </form>
            <p className="text-small text-ink-3">{t.privacy}</p>
          </Section>

          {s?.has_key && (
            <Section title={t.checkTitle} help={t.checkHelp}>
              <Button onClick={runCheck} disabled={check === "running"}>
                {check === "running" ? t.checking : t.checkButton}
              </Button>
              {check && check !== "running" && (
                <>
                  {check.ok && <Alert tone="ok">{t.checkOk}</Alert>}
                  {check.problem && <Alert tone="error">{es.aiNotice[check.problem]}</Alert>}
                  {check.models
                    .filter((m) => !m.exists || !m.can_generate)
                    .map((m) => (
                      <Alert key={m.model} tone="error">
                        {t.checkMissing(m.model)}
                      </Alert>
                    ))}
                </>
              )}
            </Section>
          )}

          <Section title={t.capTitle} help={t.capHelp}>
            <form
              className="flex flex-wrap items-end gap-3"
              onSubmit={(e) => {
                e.preventDefault();
                void run(() => aiSetCap(Number(cap)), es.common.saved);
              }}
            >
              <TextInput
                label={t.capLabel}
                prefix="$"
                className="min-w-[180px] flex-1"
                inputMode="decimal"
                placeholder={s ? String(s.cap_mxn) : ""}
                value={cap}
                onChange={(e) => setCap(e.target.value)}
              />
              <Button type="submit" variant="primary" disabled={cap.trim() === "" || Number.isNaN(Number(cap))}>
                {t.capSave}
              </Button>
            </form>
          </Section>
        </div>
      </div>

      <Section title={t.usageTitle} help={t.usageHelp}>
        {u && u.models.every((m) => m.calls_total === 0) && <p className="text-ui text-ink-2">{t.usageEmpty}</p>}
        {u && (
          <div className="grid gap-4 md:grid-cols-2">
            {u.models.map((m) => (
              <ModelCard key={`${m.provider}/${m.model}`} m={m} />
            ))}
          </div>
        )}
        {u && u.recent.length > 0 && <RecentTable calls={u.recent} />}
      </Section>
    </div>
  );
}
