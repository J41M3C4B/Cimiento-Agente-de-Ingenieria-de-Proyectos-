import { useState } from "react";
import { Icon } from "../../components/icons";
import { es } from "../../i18n/es-MX";
import type { DonorKind, ProjectColor, ProjectRow } from "../../lib/types";
import { PROJECT_STEPS, stepIndex } from "./steps";

const t = es.projects;

/** Each color of a folder: a flat, full color, and the ink that reads well over it. */
const FOLDER: Record<ProjectColor, { fill: string; ink: string }> = {
  blue: { fill: "#24459a", ink: "#ffffff" },
  violet: { fill: "#5a4aa8", ink: "#ffffff" },
  teal: { fill: "#1b7a74", ink: "#ffffff" },
  green: { fill: "#2f7a52", ink: "#ffffff" },
  amber: { fill: "#d9a13b", ink: "#2b1d00" },
  orange: { fill: "#c8693a", ink: "#ffffff" },
  pink: { fill: "#a8456f", ink: "#ffffff" },
  red: { fill: "#a93a3a", ink: "#ffffff" },
};
export const FOLDER_COLORS = Object.keys(FOLDER) as ProjectColor[];

/** The color a folder has when nobody picked one: always the same for the same project. */
export const defaultColor = (id: string): ProjectColor =>
  FOLDER_COLORS[[...id].reduce((h, c) => (h * 31 + c.charCodeAt(0)) >>> 0, 7) % FOLDER_COLORS.length]!;

/**
 * A project as a folder, all of one flat color: the kind of project on the tab; on the body its name, the bars that
 * show how far it is, the state below them (and «Por revisar» on the right) and what can be done. The body keeps a
 * 7:5 shape so it never looks stretched.
 */
export function ProjectFolder({
  project, busy, onOpen, onDelete, onColor, onKind,
}: {
  project: ProjectRow;
  busy?: boolean;
  onOpen: () => void;
  onDelete: () => void;
  onColor: (color: ProjectColor) => void;
  onKind: (kind: DonorKind) => void;
}) {
  const [picking, setPicking] = useState(false);
  const color = project.color ?? defaultColor(project.id);
  const { fill } = FOLDER[color];
  const i = stepIndex(project.stage);
  const done = project.stage === "READY";
  const filled = done ? PROJECT_STEPS.length : Math.max(i, 0) + 1;

  return (
    <li className="group flex min-w-0 flex-col rounded-2xl bg-white p-6 shadow-card transition-shadow hover:shadow-panel">
      <div className="flex items-center justify-between gap-3">
        <p className="truncate text-[12px] font-medium uppercase tracking-[0.1em] text-stone-500">{project.donor_kind ? t.kinds[project.donor_kind] : t.kindNone}</p>
        <div className="relative shrink-0">
          <button
            type="button"
            aria-label={t.colorLabel}
            title={t.colorLabel}
            aria-expanded={picking}
            onClick={() => setPicking((v) => !v)}
            className="flex h-6 w-6 items-center justify-center rounded-full transition-transform hover:scale-110"
          >
            <span className="h-3 w-3 rounded-full ring-4 ring-stone-100" style={{ background: fill }} />
          </button>
            {picking && (
              <>
                <div className="fixed inset-0 z-10" onClick={() => setPicking(false)} aria-hidden="true" />
                <div role="group" aria-label={t.colorLabel} className="absolute right-0 top-8 z-20 w-[236px] rounded-2xl bg-white p-3 text-stone-900 shadow-lift">
                  <p className="mb-2 text-[12px] font-semibold text-stone-600">{t.colorLabel}</p>
                  <div className="flex flex-wrap gap-2.5">
                    {FOLDER_COLORS.map((name) => (
                      <button
                        key={name}
                        type="button"
                        aria-label={t.colors[name]}
                        title={t.colors[name]}
                        aria-pressed={name === color}
                        onClick={() => {
                          onColor(name);
                          setPicking(false);
                        }}
                        className={`flex h-6 w-6 items-center justify-center rounded-full transition-transform hover:scale-110 ${name === color ? "ring-2 ring-stone-900 ring-offset-2" : ""}`}
                        style={{ background: FOLDER[name].fill, color: FOLDER[name].ink }}
                      >
                        {name === color && <Icon name="check" size={13} strokeWidth={3.2} />}
                      </button>
                    ))}
                  </div>
                  <p className="mb-1.5 mt-4 border-t border-stone-200 pt-3 text-[12px] font-semibold text-stone-600">{t.kindLabel}</p>
                  <div className="space-y-0.5">
                    {(Object.keys(t.kinds) as DonorKind[]).map((k) => (
                      <button
                        key={k}
                        type="button"
                        aria-pressed={k === project.donor_kind}
                        onClick={() => {
                          onKind(k);
                          setPicking(false);
                        }}
                        className={`flex w-full items-center justify-between rounded-md px-2 py-1.5 text-left text-[13px] transition-colors hover:bg-stone-100 ${k === project.donor_kind ? "font-semibold" : ""}`}
                      >
                        {t.kinds[k]}
                        {k === project.donor_kind && <Icon name="check" size={14} strokeWidth={3} className="text-blue-800" />}
                      </button>
                    ))}
                  </div>
                </div>
              </>
            )}
        </div>
      </div>

      <h2 className="mt-5 line-clamp-3 min-w-0 text-[20px] font-semibold leading-snug tracking-tight" title={project.title}>
        {project.title}
      </h2>

      <div className="mt-auto pt-10">
        <div className="flex gap-1.5" role="img" aria-label={t.stepOf(filled, PROJECT_STEPS.length)}>
          {PROJECT_STEPS.map((s, n) => (
            <span key={s} className="h-1.5 flex-1 rounded-full" style={{ background: n < filled ? fill : "var(--color-stone-200)" }} />
          ))}
        </div>
        <div className="mt-3 flex items-center justify-between gap-3 text-[13px] text-stone-600">
          <span>{t.status(done ? t.guideReady : es.steps[project.stage]!)}</span>
          {project.needs_review && <span className="font-medium text-amber-800">{t.review}</span>}
        </div>
        <div className="mt-5 flex items-center justify-between gap-3">
          <button
            type="button"
            onClick={onDelete}
            disabled={busy}
            className="rounded-md py-1 text-[13px] font-medium text-stone-500 transition-colors hover:text-red-800 disabled:opacity-40"
          >
            {t.delete}
          </button>
          <button
            type="button"
            onClick={onOpen}
            className="inline-flex min-h-[38px] items-center gap-1.5 rounded-xl bg-stone-900 px-4 text-[13px] font-semibold text-white transition-colors hover:bg-stone-700"
          >
            {done ? t.open : t.resume}
            {!done && <Icon name="next" size={15} />}
          </button>
        </div>
      </div>
    </li>
  );
}
