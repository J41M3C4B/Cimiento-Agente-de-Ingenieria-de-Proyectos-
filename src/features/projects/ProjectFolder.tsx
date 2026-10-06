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
  const { fill, ink } = FOLDER[color];
  const i = stepIndex(project.stage);
  const done = project.stage === "READY";
  const filled = done ? PROJECT_STEPS.length : Math.max(i, 0) + 1;

  return (
    <li className="flex min-w-0 flex-col" style={{ color: ink }}>
      {/* the tab, with a small curve where it meets the body */}
      <div className="relative z-10 inline-flex max-w-[76%] items-center self-start rounded-t-2xl px-5 py-3" style={{ background: fill }}>
        <p className="truncate text-[14px] font-semibold leading-snug">{project.donor_kind ? t.kinds[project.donor_kind] : t.kindNone}</p>
        <span aria-hidden="true" className="absolute -right-3 bottom-0 h-3 w-3" style={{ background: `radial-gradient(circle at 100% 0, transparent 12px, ${fill} 12.5px)` }} />
      </div>

      <div className="flex aspect-[7/5] min-h-[210px] flex-col rounded-2xl rounded-tl-none p-6" style={{ background: fill }}>
        <div className="flex items-start justify-between gap-3">
          <h2 className="line-clamp-3 min-w-0 text-[20px] font-semibold leading-snug" title={project.title}>
            {project.title}
          </h2>
          <div className="relative shrink-0">
            <button
              type="button"
              aria-label={t.colorLabel}
              title={t.colorLabel}
              aria-expanded={picking}
              onClick={() => setPicking((v) => !v)}
              className="h-5 w-5 rounded-full border-2 transition-transform hover:scale-110"
              style={{ borderColor: ink, background: "rgba(255,255,255,0.28)" }}
            />
            {picking && (
              <>
                <div className="fixed inset-0 z-10" onClick={() => setPicking(false)} aria-hidden="true" />
                <div role="group" aria-label={t.colorLabel} className="absolute right-0 top-8 z-20 w-[236px] rounded-xl border border-stone-200 bg-white p-3 text-stone-900 shadow-lift">
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

        <div className="mt-auto">
          <div className="flex gap-1.5" role="img" aria-label={t.stepOf(filled, PROJECT_STEPS.length)}>
            {PROJECT_STEPS.map((s, n) => (
              <span key={s} className="h-1.5 flex-1 rounded-full" style={{ background: ink, opacity: n < filled ? 1 : 0.28 }} />
            ))}
          </div>
          <div className="mt-2.5 flex items-center justify-between gap-3 text-[12.5px] font-normal opacity-90">
            <span>{t.status(done ? t.guideReady : es.steps[project.stage]!)}</span>
            {project.needs_review && <span className="font-medium">{t.review}</span>}
          </div>
          <div className="mt-4 flex items-center justify-between gap-3">
            <button
              type="button"
              onClick={onDelete}
              disabled={busy}
              className="rounded-md py-1 text-[13px] font-medium opacity-80 transition-opacity hover:underline hover:opacity-100 disabled:opacity-40"
            >
              {t.delete}
            </button>
            <button
              type="button"
              onClick={onOpen}
              className="inline-flex min-h-[36px] items-center gap-1.5 rounded-lg bg-white px-4 text-[13px] font-semibold text-stone-900 transition-colors hover:bg-stone-100"
            >
              {done ? t.open : t.resume}
              {!done && <Icon name="next" size={15} />}
            </button>
          </div>
        </div>
      </div>
    </li>
  );
}
