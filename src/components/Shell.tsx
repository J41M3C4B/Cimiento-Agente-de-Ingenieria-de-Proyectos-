import type { ReactNode } from "react";
import { es } from "../i18n/es-MX";
import { projectTone } from "../lib/palette";
import { useTheme } from "../lib/theme";
import type { ProjectRow } from "../lib/types";
import { PersonMenu } from "../core/access/PersonMenu";
import type { SessionApi } from "../core/access/session";
import { PROJECT_STEPS, stepIndex } from "../modules/projects/steps";
import type { IconName } from "./icons";
import { MODULE_META } from "./modules";
import type { ModuleId } from "./modules";
import { Avatar, IconButton, Logo, StepDots } from "./ui";

/** The sections of the rail: the core (Inicio, Mi institución, Documentos), the modules (ADR-032) and the settings. */
export type Page = "home" | "profile" | "documents" | "projects" | "staff" | "people" | "facilities" | "finance" | "ai" | "security" | "admin" | "help";

const CORE: [Page, string, IconName][] = [
  ["home", es.nav.home, "home"],
  ["profile", es.nav.profile, "idcard"],
  ["documents", es.nav.documents, "file"],
];
/** One button per module of the institution (ADR-032), each in its own color (components/modules.ts). */
const MODULES: [ModuleId, string][] = [
  ["projects", es.nav.projects],
  ["staff", es.nav.staff],
  ["people", es.nav.people],
  ["facilities", es.nav.facilities],
  ["finance", es.nav.finance],
];
/** The sections below the line; some only for whoever may use them (ADR-028). */
const MORE: [Page, string, IconName, "settings" | "administer" | null][] = [
  ["ai", es.nav.ai, "sparkles", "settings"],
  ["security", es.nav.security, "shield", null],
  ["admin", es.nav.admin, "users", "administer"],
];

/**
 * The layout of the whole program (docs/13 §6): a rail of round buttons on the left, a
 * capsule on top that says where the person is, and the page in the middle.
 */
export function Shell({
  page,
  onNavigate,
  institution,
  focus,
  onOpenFocus,
  access,
  children,
}: {
  page: Page;
  onNavigate: (page: Page) => void;
  institution: string;
  /** the project the capsule talks about: the one open, or else the one in progress */
  focus: ProjectRow | undefined;
  onOpenFocus: () => void;
  /** the person inside: the menu shows what they may use, and the rail locks or closes the session */
  access: SessionApi | null;
  children: ReactNode;
}) {
  const [theme, toggleTheme] = useTheme();
  const rail = ([id, label, icon]: [Page, string, IconName]) => (
    <IconButton key={id} icon={icon} label={label} tip={label} aria-current={page === id ? "page" : undefined} onClick={() => onNavigate(id)} />
  );
  const moduleButton = ([id, label]: [ModuleId, string]) => (
    <IconButton key={id} icon={MODULE_META[id].icon} label={label} tip={label} tone={MODULE_META[id].tone} aria-current={page === id ? "page" : undefined} onClick={() => onNavigate(id)} />
  );
  const done = focus?.stage === "READY";
  const at = focus ? stepIndex(focus.stage) : -1;
  const tone = focus ? projectTone(focus.color, focus.id) : "sky";

  return (
    <div className="shell">
      <div className="frame">
        <header className="topbar">
          <Logo className="h-8" />
          <div className="capsule" aria-label={es.nav.openProject}>
            {focus ? (
              <>
                <b className="min-w-0 max-w-[40%] truncate text-ui font-bold" title={focus.title}>
                  {focus.title}
                </b>
                <StepDots total={PROJECT_STEPS.length} at={at} done={done} tone={tone} className="hidden shrink-0 sm:flex" />
                <span className="min-w-0 flex-1 truncate text-small font-semibold text-ink-2">
                  {done ? es.projects.guideReady : at >= 0 ? es.projects.stepLabel(at + 1, PROJECT_STEPS.length, es.steps[focus.stage]) : es.steps[focus.stage]}
                </span>
                <IconButton icon="arrowUpRight" label={es.nav.openProject} size="sm" onClick={onOpenFocus} />
              </>
            ) : (
              <b className="text-ui font-bold">{es.app.name}</b>
            )}
          </div>
          <div className="ml-auto flex items-center gap-3">
            <IconButton icon={theme === "dark" ? "sun" : "moon"} label={theme === "dark" ? es.nav.themeLight : es.nav.themeDark} onClick={toggleTheme} />
            {access ? (
              <PersonMenu access={access} />
            ) : (
              <button type="button" onClick={() => onNavigate("profile")} title={es.nav.profile} className="flex items-center gap-2.5 border-l border-line pl-4 text-ui font-bold">
                <span className="hidden max-w-[220px] truncate md:inline">{institution}</span>
                <Avatar name={institution} tone="violet" />
              </button>
            )}
          </div>
        </header>
        <nav aria-label={es.nav.mainSections} className="rail">
          <div role="group" aria-label={es.nav.coreGroup} className="rail-core">
            {CORE.map(rail)}
          </div>
          <div role="group" aria-label={es.nav.modulesGroup} className="rail-group">
            {MODULES.map(moduleButton)}
          </div>
          <span className="rail-sep" aria-hidden="true" />
          {MORE.filter(([, , , needs]) => !needs || access?.can(needs)).map(([id, label, icon]) => rail([id, label, icon]))}
          <span className="rail-gap" aria-hidden="true" />
          {rail(["help", es.nav.help, "help"])}
          {access && (
            <>
              <IconButton icon="lock" label={es.access.menu.lock} tip={es.access.menu.lock} onClick={access.lock} />
              <IconButton icon="x" label={`${es.access.menu.logout} (${access.session.display_name})`} tip={es.access.menu.logout} variant="danger" onClick={access.logout} />
            </>
          )}
        </nav>
        <main className="frame-main min-w-0">{children}</main>
      </div>
    </div>
  );
}
