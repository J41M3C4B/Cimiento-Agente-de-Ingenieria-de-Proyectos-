import { useEffect, useRef, useState } from "react";
import { Icon } from "../../components/icons";
import { Button, Card, Choice, Eyebrow, Folder, IconButton, Inset, Segments, Tag } from "../../components/ui";
import { es } from "../../i18n/es-MX";
import { PICKABLE_PROJECT_COLORS, projectTone, PROJECT_TONE } from "../../lib/palette";
import type { DonorKind, ProjectColor, ProjectRow } from "../../lib/types";
import { shortDate, stepInfo, useProjectCall } from "./projectCall";

const t = es.projects;
const h = es.home;

/**
 * A project as a folder, small: to know where it is without opening it (docs/13 §7, «Tarjeta de proyecto»). The tab
 * has its name in its color; the body, its step, who calls, the six steps, what comes next and, at the foot, when it
 * closes and what can be done. Choosing the color or the kind of donor, and deleting, are only offered when the
 * parent gives the way to do them (Inicio shows the same card without them).
 */
export function ProjectFolder({
  project, busy, primary, onOpen, onDelete, onColor, onKind,
}: {
  project: ProjectRow;
  busy?: boolean;
  /** the button is the main one of the page (the most urgent project) */
  primary?: boolean;
  onOpen: () => void;
  onDelete?: () => void;
  onColor?: (color: ProjectColor) => void;
  onKind?: (kind: DonorKind) => void;
}) {
  const [picking, setPicking] = useState(false);
  const menu = useRef<HTMLDivElement>(null);
  const { funder, closes, days } = useProjectCall(project);
  const tone = projectTone(project.color, project.id);
  const step = stepInfo(project);
  const [nextTitle] = h.nextByStage[project.stage] ?? h.nextByStage.CALL_SELECTION!;
  const canPick = Boolean(onColor || onKind);
  // who calls; when the call does not say, the kind of donor stands in
  const who = funder ?? (project.donor_kind ? t.kinds[project.donor_kind] : null);

  // the small menu closes with a click outside it or with Escape
  useEffect(() => {
    if (!picking) return;
    const away = (e: MouseEvent) => {
      if (!menu.current?.contains(e.target as Node)) setPicking(false);
    };
    const esc = (e: KeyboardEvent) => e.key === "Escape" && setPicking(false);
    document.addEventListener("mousedown", away);
    document.addEventListener("keydown", esc);
    return () => {
      document.removeEventListener("mousedown", away);
      document.removeEventListener("keydown", esc);
    };
  }, [picking]);

  return (
    <Folder as="li" tone={tone} className={picking ? "z-20" : ""} title={<span title={project.title}>{project.title}</span>}>
      <div className="flex flex-wrap items-center justify-between gap-x-3 gap-y-2">
        <div className="flex min-w-0 flex-wrap items-center gap-2">
          <Tag tone="pc" icon={step.done ? "check" : undefined}>
            {step.done ? es.steps.READY : step.tag}
          </Tag>
          {project.needs_review && <Tag tone="amber">{t.review}</Tag>}
        </div>
        {who && (
          <span className="min-w-0 max-w-full truncate text-small text-ink-3" title={who}>
            {who}
          </span>
        )}
      </div>

      <Segments total={6} filled={step.filled} tone="pc" label={t.stepOf(step.filled, 6)} />

      <Inset className="space-y-1">
        <Eyebrow>{step.done ? h.nextDone : h.next}</Eyebrow>
        <p className="text-ui font-bold">{nextTitle}</p>
      </Inset>

      <div className="mt-auto flex flex-wrap items-center justify-between gap-x-3 gap-y-3">
        {closes && days !== null ? (
          <div className="min-w-0">
            <b className="block text-ui font-bold">
              {h.closes} {shortDate(closes)}
            </b>
            <span className={`text-small font-semibold ${days >= 0 && days <= 10 ? "text-amber-ink" : "text-ink-3"}`}>{days < 0 ? h.closed : h.daysLeft(days)}</span>
          </div>
        ) : (
          <span />
        )}
        <div className="flex items-center gap-1">
          {onDelete && <IconButton icon="trash" label={t.delete} variant="plain" size="sm" disabled={busy} onClick={onDelete} />}
          {canPick && (
            <div ref={menu}>
              <IconButton icon="sliders" label={t.colorLabel} variant="plain" size="sm" aria-expanded={picking} onClick={() => setPicking((v) => !v)} />
              {picking && (
                <Card small className="absolute bottom-16 right-4 z-20 w-[min(20rem,calc(100%-2rem))] space-y-3 !shadow-float">
                  {onColor && (
                    <div role="group" aria-label={t.colorLabel} className="space-y-2">
                      <Eyebrow>{t.colorLabel}</Eyebrow>
                      <div className="flex flex-wrap gap-2">
                        {PICKABLE_PROJECT_COLORS.map((name) => (
                          <Choice
                            key={name}
                            tone={PROJECT_TONE[name]}
                            name={`color-${project.id}`}
                            checked={PROJECT_TONE[name] === tone}
                            onChange={() => {
                              onColor(name);
                              setPicking(false);
                            }}
                          >
                            {t.colors[name]}
                          </Choice>
                        ))}
                      </div>
                    </div>
                  )}
                  {onKind && (
                    <div role="group" aria-label={t.kindLabel} className="space-y-1 border-t border-line pt-3">
                      <Eyebrow>{t.kindLabel}</Eyebrow>
                      {(Object.keys(t.kinds) as DonorKind[]).map((k) => (
                        <button
                          key={k}
                          type="button"
                          aria-pressed={k === project.donor_kind}
                          onClick={() => {
                            onKind(k);
                            setPicking(false);
                          }}
                          className={`flex min-h-ctl-sm w-full items-center justify-between gap-2 rounded-field px-3 text-left text-ui transition-colors hover:bg-inset ${k === project.donor_kind ? "font-bold" : "font-semibold"}`}
                        >
                          {t.kinds[k]}
                          {k === project.donor_kind && <Icon name="check" size={16} strokeWidth={3} />}
                        </button>
                      ))}
                    </div>
                  )}
                </Card>
              )}
            </div>
          )}
          <Button size="sm" variant={primary ? "primary" : "secondary"} onClick={onOpen}>
            {step.done ? h.goReady : h.go}
            <Icon name="next" size={16} />
          </Button>
        </div>
      </div>
    </Folder>
  );
}
