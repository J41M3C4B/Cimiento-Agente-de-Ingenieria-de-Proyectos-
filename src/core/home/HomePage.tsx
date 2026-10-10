import { useQuery } from "@tanstack/react-query";
import { Icon } from "../../components/icons";
import { MODULE_META } from "../../components/modules";
import type { ModuleId } from "../../components/modules";
import { Alert, Button, Card, Segments, Tile, PageHeader } from "../../components/ui";
import type { Tone } from "../../components/ui";
import type { IconName } from "../../components/icons";
import { es } from "../../i18n/es-MX";
import { dayPartOf, firstName } from "../../lib/greeting";
import { projectList } from "../../lib/tauri";
import { useSession } from "../access/session";
import { facilitiesOverview } from "../../modules/facilities/api";
import { FACILITIES_KEY } from "../../modules/facilities/FacilitiesTab";
import { ONBOARDING_KEY, onboardingStatus } from "../onboarding/api";
import { resumeOnboarding } from "../onboarding/OnboardingGate";
import { GlobalFigures } from "./GlobalFigures";
import { InstitutionCard } from "./InstitutionCard";
import { Ongoing } from "./Ongoing";
import { TodoCard } from "./TodoCard";
import type { Page } from "../../components/Shell";

const t = es.home;

/**
 * Inicio (docs/13 §10): answers «¿qué hago ahora?» and «¿se me viene un plazo?». At the top the figures of the whole
 * institution, «Por hacer» and the dark card of «Mi institución»; below, the long folder with what is under way and the
 * notices; and, when it applies, the next steps to get to know the institution better.
 */
export function HomePage({
  onOpenProject,
  onNewProject,
  onGoProjects,
  onGoProfile,
  onGo,
  onGoAi,
}: {
  onOpenProject: (id: string) => void;
  onNewProject: () => void;
  onGoProjects: () => void;
  onGoProfile: () => void;
  /** opens a section or a module of the top bar */
  onGo: (page: Page) => void;
  onGoAi: () => void;
}) {
  const projects = useQuery({ queryKey: ["projects"], queryFn: projectList });

  const all = projects.data ?? [];
  const access = useSession();
  const name = firstName(access?.session.display_name);
  const greeting = t.greeting[dayPartOf(new Date().getHours())];

  return (
    <div className="space-y-6">
      <PageHeader
        title={name ? `${greeting}, ${name}` : greeting}
        intro={t.greetingIntro}
        action={
          <Button variant="primary" onClick={onNewProject}>
            <Icon name="plus" />
            {t.newProject}
          </Button>
        }
      />

      <PendingData />

      <GlobalFigures
        onGo={onGo}
        aside={
          <div className="fig-stack">
            <TodoCard />
            <InstitutionCard onOpen={onGoProfile} />
          </div>
        }
      />

      <Ongoing projects={all} loaded={projects.isSuccess} onOpenProject={onOpenProject} onNewProject={onNewProject} onGoProjects={onGoProjects} />

      <NextSteps onGo={onGo} onGoAi={onGoAi} />
    </div>
  );
}

/**
 * For the administrator who left the data of the institution to the direction: Inicio says they are still missing,
 * and a button takes them up again. Once the institution finishes them, it disappears.
 */
function PendingData() {
  const status = useQuery({ queryKey: ONBOARDING_KEY, queryFn: onboardingStatus });
  const s = status.data;
  if (!s || s.done || !s.can_postpone) return null;
  const n = s.steps.filter((x) => !x.complete).length;
  return (
    <Alert tone="warn">
      <div className="flex flex-wrap items-center justify-between gap-x-6 gap-y-3">
        <div className="min-w-[240px] flex-1">
          <b className="block font-bold">{t.pending.title}</b>
          <p className="text-ui text-ink-2">{t.pending.text(n)}</p>
        </div>
        <Button size="sm" variant="primary" onClick={resumeOnboarding}>
          {t.pending.action}
        </Button>
      </div>
    </Alert>
  );
}

/**
 * After the first start: what to do next so the assistant knows the institution better (ADR-031). Each step goes to
 * where it is done and goes away when it is done; with all of them done the card is not drawn.
 */
function NextSteps({ onGo, onGoAi }: { onGo: (page: Page) => void; onGoAi: () => void }) {
  const access = useSession();
  const status = useQuery({ queryKey: ONBOARDING_KEY, queryFn: onboardingStatus });
  const facilities = useQuery({ queryKey: FACILITIES_KEY, queryFn: facilitiesOverview });
  const s = status.data;
  if (!s || !s.done) return null;
  type Step = { key: string; icon: IconName; tone: Tone; todo: boolean; go: () => void };
  const of = (module: Exclude<ModuleId, "projects">, todo: boolean): Step => ({ key: module, icon: MODULE_META[module].icon, tone: "ink", todo, go: () => onGo(module) });
  const all: Step[] = [
    of("staff", s.records.staff === 0),
    of("people", s.records.served === 0),
    ...(facilities.isSuccess ? [of("facilities", facilities.data.indicators.spaces === 0)] : []),
    ...(s.setup && access?.can("settings") ? [{ key: "ai", icon: "sparkles" as IconName, tone: "ink" as const, todo: !s.setup.ai_ready, go: onGoAi }] : []),
  ];
  const pending = all.filter((x) => x.todo);
  if (pending.length === 0) return null;
  const n = t.nextSteps;
  return (
    <Card className="space-y-4">
      <div className="flex flex-wrap items-start justify-between gap-x-6 gap-y-2">
        <div className="min-w-[260px] flex-1">
          <h2 className="text-heading font-bold">{n.title}</h2>
          <p className="max-w-[70ch] text-ui text-ink-2">{n.help}</p>
        </div>
        <div className="flex min-w-[140px] flex-col gap-1.5">
          <span className="text-small font-semibold text-ink-2">{n.progress(all.length - pending.length, all.length)}</span>
          <Segments total={all.length} filled={all.length - pending.length} label={n.progress(all.length - pending.length, all.length)} tone="ac" />
        </div>
      </div>
      <ul className="grid gap-3 md:grid-cols-2">
        {pending.map((x) => {
          const [title, detail] = n.items[x.key]!;
          return (
            <li key={x.key} className="flex items-center gap-4 rounded-inset bg-inset p-4">
              <Tile icon={x.icon} tone={x.tone} />
              <div className="min-w-0 flex-1">
                <b className="block font-bold">{title}</b>
                <span className="block text-small text-ink-2">{detail}</span>
              </div>
              <Button size="sm" variant="secondary" onClick={x.go}>
                {n.go}
                <Icon name="next" size={14} />
              </Button>
            </li>
          );
        })}
      </ul>
    </Card>
  );
}
