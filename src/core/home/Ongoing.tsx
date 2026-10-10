import { useState } from "react";
import { Icon } from "../../components/icons";
import { Button, Calendar, Folder, Inset, NextBox, Steps, Tag } from "../../components/ui";
import { es } from "../../i18n/es-MX";
import { projectTone } from "../../lib/palette";
import type { ProjectRow } from "../../lib/types";
import { shortDate, stepInfo, useProjectCall } from "../../modules/projects/projectCall";
import { stepsForView } from "../../modules/projects/steps";

const t = es.home;

type View = "ongoing" | "notices";

const capitalize = (s: string) => s.charAt(0).toUpperCase() + s.slice(1);

/**
 * The long folder of Inicio, with the structure of the modules: one tab per view. «En curso» is where the person
 * picks up what they were doing (today the projects; the work of the other modules comes here later) and
 * «Notificaciones» is where the notices will arrive (still empty).
 */
export function Ongoing({
  projects,
  loaded,
  onOpenProject,
  onNewProject,
  onGoProjects,
}: {
  projects: ProjectRow[];
  /** the projects have already been read, so «none» is true and not just «not yet» */
  loaded: boolean;
  onOpenProject: (id: string) => void;
  onNewProject: () => void;
  onGoProjects: () => void;
}) {
  const [view, setView] = useState<View>("ongoing");
  // the project in progress is the most recent one that is not ready; if all are ready, the most recent
  const current = projects.find((p) => p.stage !== "READY") ?? projects[0];
  const others = projects.filter((p) => p.id !== current?.id);
  const underway = projects.filter((p) => p.stage !== "READY").length;

  return (
    <Folder
      tone="ac"
      tabs={{
        label: t.ongoing.label,
        value: view,
        onChange: (id) => setView(id as View),
        items: [
          { id: "ongoing", label: t.ongoing.tab, count: underway },
          { id: "notices", label: t.ongoing.notices },
        ],
      }}
    >
      {view === "ongoing" ? (
        current ? (
          <div className="grid gap-6 lg:grid-cols-[minmax(0,3fr)_minmax(0,2fr)]">
            <CurrentProject project={current} onOpen={() => onOpenProject(current.id)} />
            <Side project={current} others={others} onOpenProject={onOpenProject} onGoProjects={onGoProjects} />
          </div>
        ) : (
          loaded && (
            <div className="flex flex-col items-center gap-3 py-10 text-center">
              <span className="grid h-ctl w-ctl place-items-center rounded-pill bg-inset text-ink-2">
                <Icon name="folder" size={22} />
              </span>
              <p className="text-body font-bold">{t.emptyTitle}</p>
              <p className="max-w-md text-ui text-ink-2">{t.emptyHelp}</p>
              <Button variant="primary" onClick={onNewProject}>
                <Icon name="plus" />
                {t.newProject}
              </Button>
            </div>
          )
        )
      ) : (
        <div className="flex flex-col items-center gap-3 py-10 text-center">
          <span className="grid h-ctl w-ctl place-items-center rounded-pill bg-inset text-ink-2">
            <Icon name="check" size={22} />
          </span>
          <p className="text-body font-bold">{t.ongoing.noticesTitle}</p>
          <p className="max-w-md text-ui text-ink-2">{t.ongoing.noticesHelp}</p>
        </div>
      )}
    </Folder>
  );
}

/** The project in progress, in its own color: where it is and the one thing to do now. */
function CurrentProject({ project, onOpen }: { project: ProjectRow; onOpen: () => void }) {
  const { funder, closes } = useProjectCall(project);
  const step = stepInfo(project);
  const [title, detail] = t.nextByStage[project.stage] ?? t.nextByStage.CALL_SELECTION!;

  return (
    <div className={`pc-scope tone-${projectTone(project.color, project.id)} flex min-w-0 flex-col gap-4`}>
      <div className="flex flex-wrap items-center justify-between gap-x-3 gap-y-1">
        <b className="min-w-0 truncate text-heading font-bold" title={project.title}>
          {project.title}
        </b>
        {closes && (
          <span className="inline-flex items-center gap-1.5 text-small text-ink-2">
            <Icon name="calendar" size={16} />
            {t.closes} {shortDate(closes)}
          </span>
        )}
      </div>
      <div className="flex flex-wrap items-center justify-between gap-x-3 gap-y-2">
        <Tag tone="pc" icon={step.done ? "check" : undefined}>
          {step.done ? es.steps.READY : step.tag}
        </Tag>
        {funder && <span className="min-w-0 truncate text-ui text-ink-3">{funder}</span>}
      </div>
      <Steps steps={stepsForView} current={step.done ? stepsForView.length - 1 : step.at} />
      <NextBox eyebrow={step.done ? t.nextDone : t.next} title={title} detail={detail} goLabel={step.done ? t.goReady : t.go} onGo={onOpen} />
    </div>
  );
}

/** What goes beside the project in progress: when its call closes, as a calendar, and the other projects. */
function Side({ project, others, onOpenProject, onGoProjects }: { project: ProjectRow; others: ProjectRow[]; onOpenProject: (id: string) => void; onGoProjects: () => void }) {
  const { closes, days } = useProjectCall(project);
  if ((!closes || days === null) && others.length === 0) return null;
  return (
    <div className="flex min-w-0 flex-col gap-4">
      {closes && days !== null && (
        <Inset className="space-y-3">
          <h2 className="text-ui font-bold">{t.deadlineTitle}</h2>
          <Calendar
            weekday={capitalize(closes.toLocaleDateString("es-MX", { weekday: "long" }))}
            day={String(closes.getDate())}
            month={`${capitalize(closes.toLocaleDateString("es-MX", { month: "long" }))} ${closes.getFullYear()}`}
            note={days < 0 ? t.closed : t.daysToDeliver(days)}
          />
        </Inset>
      )}
      {others.length > 0 && (
        <section className="space-y-2" aria-labelledby="home-others">
          <div className="flex items-center justify-between gap-3">
            <h2 id="home-others" className="text-ui font-bold">
              {t.others}
            </h2>
            <button type="button" onClick={onGoProjects} className="inline-flex items-center gap-1 text-small font-bold text-ink underline underline-offset-4">
              {t.seeAll}
              <Icon name="next" size={14} />
            </button>
          </div>
          <ul className="flex flex-col gap-2">
            {others.slice(0, 3).map((p) => (
              <li key={p.id}>
                <button type="button" onClick={() => onOpenProject(p.id)} className="flex w-full items-center gap-3 rounded-inset bg-inset px-4 py-3 text-left">
                  <span className="min-w-0 flex-1">
                    <b className="block truncate text-ui font-bold" title={p.title}>
                      {p.title}
                    </b>
                    <span className="block truncate text-small text-ink-2">{stepInfo(p).done ? es.steps.READY : stepInfo(p).tag}</span>
                  </span>
                  <Icon name="next" size={16} />
                </button>
              </li>
            ))}
          </ul>
        </section>
      )}
    </div>
  );
}
