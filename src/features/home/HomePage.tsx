import { useQuery } from "@tanstack/react-query";
import { Icon } from "../../components/icons";
import { Avatar, Button, Calendar, Card, Folder, NextBox, Steps, Tag, PageHeader } from "../../components/ui";
import { es } from "../../i18n/es-MX";
import { projectTone } from "../../lib/palette";
import { profileGet, projectList } from "../../lib/tauri";
import type { ProfileView, ProjectRow } from "../../lib/types";
import { shortDate, stepInfo, useProjectCall } from "../projects/projectCall";
import { ProjectFolder } from "../projects/ProjectFolder";
import { stepsForView } from "../projects/steps";

const t = es.home;

const capitalize = (s: string) => s.charAt(0).toUpperCase() + s.slice(1);

/** The state of the institution's sheet, in words and a color. */
function sheetState(profile: ProfileView | null | undefined): { tone: "green" | "amber"; text: string } {
  if (!profile) return { tone: "amber", text: es.profile.status.none };
  return profile.is_draft ? { tone: "amber", text: es.profile.status.draft } : { tone: "green", text: es.profile.status.confirmed };
}

/**
 * Inicio (docs/13 §10): answers «¿qué hago ahora?» and «¿se me viene un plazo?». The project in progress as a big
 * folder with its one next step, the date it closes, the sheet of the institution in one small tray and the other
 * projects as small folders.
 */
export function HomePage({
  onOpenProject,
  onNewProject,
  onGoProjects,
  onGoProfile,
}: {
  onOpenProject: (id: string) => void;
  onNewProject: () => void;
  onGoProjects: () => void;
  onGoProfile: () => void;
}) {
  const projects = useQuery({ queryKey: ["projects"], queryFn: projectList });
  const profile = useQuery({ queryKey: ["profile"], queryFn: profileGet });

  const all = projects.data ?? [];
  // the project in progress is the most recent one that is not ready; if all are ready, the most recent
  const current = all.find((p) => p.stage !== "READY") ?? all[0];
  const others = all.filter((p) => p.id !== current?.id);
  const today = capitalize(new Date().toLocaleDateString("es-MX", { weekday: "long", day: "numeric", month: "long" }));
  const institution = profile.data?.input.institution.name?.trim() || es.nav.profile;
  const sheet = sheetState(profile.data);

  return (
    <div className="space-y-6">
      <PageHeader
        title={t.greeting}
        intro={today}
        action={
          <Button variant="primary" onClick={onNewProject}>
            <Icon name="plus" />
            {t.newProject}
          </Button>
        }
      />

      {projects.isSuccess && !current && (
        <Card className="flex flex-col items-center gap-3 py-12 text-center">
          <span className="grid h-ctl w-ctl place-items-center rounded-pill bg-inset text-ink-2">
            <Icon name="folder" size={22} />
          </span>
          <p className="text-body font-bold">{t.emptyTitle}</p>
          <p className="max-w-md text-ui text-ink-2">{t.emptyHelp}</p>
          <Button variant="primary" onClick={onNewProject}>
            <Icon name="plus" />
            {t.newProject}
          </Button>
        </Card>
      )}

      {current && (
        <div className="grid gap-4 lg:grid-cols-[minmax(0,3fr)_minmax(0,2fr)]">
          <CurrentProject project={current} onOpen={() => onOpenProject(current.id)} />
          <div className="flex min-w-0 flex-col gap-4">
            <DeadlineCard project={current} />
            <Card small className="space-y-3">
              <div className="flex flex-wrap items-center gap-3">
                <Avatar name={institution} />
                <div className="min-w-[10rem] flex-1">
                  <b className="block truncate font-bold leading-tight">{institution}</b>
                  <span className="block text-small text-ink-3">{t.institutionOwner}</span>
                </div>
                <Tag tone={sheet.tone} icon={sheet.tone === "green" ? "check" : undefined}>
                  {sheet.text}
                </Tag>
              </div>
              <p className="text-ui text-ink-2">{t.institutionNote}</p>
              <button type="button" onClick={onGoProfile} className="inline-flex items-center gap-1 text-ui font-bold text-ink underline underline-offset-4">
                {t.goProfile}
                <Icon name="next" size={16} />
              </button>
            </Card>
          </div>
        </div>
      )}

      {others.length > 0 && (
        <section className="space-y-3" aria-labelledby="home-others">
          <div className="flex items-center justify-between gap-3">
            <h2 id="home-others" className="text-heading font-bold">
              {t.others}
            </h2>
            <button type="button" onClick={onGoProjects} className="inline-flex items-center gap-1 text-ui font-bold text-ink underline underline-offset-4">
              {t.seeAll}
              <Icon name="next" size={16} />
            </button>
          </div>
          <ul className="grid gap-4 grid-cols-[repeat(auto-fill,minmax(min(100%,360px),1fr))]">
            {others.slice(0, 3).map((p) => (
              <ProjectFolder key={p.id} project={p} onOpen={() => onOpenProject(p.id)} />
            ))}
          </ul>
        </section>
      )}
    </div>
  );
}

/** The big folder: the project in progress, where it is and the one thing to do now. */
function CurrentProject({ project, onOpen }: { project: ProjectRow; onOpen: () => void }) {
  const { funder, closes } = useProjectCall(project);
  const step = stepInfo(project);
  const [title, detail] = t.nextByStage[project.stage] ?? t.nextByStage.CALL_SELECTION!;

  return (
    <Folder
      tone={projectTone(project.color, project.id)}
      title={<span title={project.title}>{project.title}</span>}
      chip={
        closes && (
          <span className="folder-chip">
            <Icon name="calendar" size={16} />
            <span>
              <span className="max-sm:hidden">{t.closes} </span>
              {shortDate(closes)}
            </span>
          </span>
        )
      }
    >
      <div className="flex flex-wrap items-center justify-between gap-x-3 gap-y-2">
        <Tag tone="pc" icon={step.done ? "check" : undefined}>
          {step.done ? es.steps.READY : step.tag}
        </Tag>
        {funder && <span className="min-w-0 truncate text-ui text-ink-3">{funder}</span>}
      </div>
      <Steps steps={stepsForView} current={step.done ? stepsForView.length - 1 : step.at} />
      <NextBox eyebrow={step.done ? t.nextDone : t.next} title={title} detail={detail} goLabel={step.done ? t.goReady : t.go} onGo={onOpen} />
    </Folder>
  );
}

/** When the call closes, as a calendar. Only when the call says so: nothing is made up. */
function DeadlineCard({ project }: { project: ProjectRow }) {
  const { closes, days } = useProjectCall(project);
  if (!closes || days === null) return null;
  return (
    <Card small className="space-y-3">
      <h2 className="text-heading font-bold">{t.deadlineTitle}</h2>
      <Calendar
        weekday={capitalize(closes.toLocaleDateString("es-MX", { weekday: "long" }))}
        day={String(closes.getDate())}
        month={`${capitalize(closes.toLocaleDateString("es-MX", { month: "long" }))} ${closes.getFullYear()}`}
        note={days < 0 ? t.closed : t.daysToDeliver(days)}
      />
    </Card>
  );
}
