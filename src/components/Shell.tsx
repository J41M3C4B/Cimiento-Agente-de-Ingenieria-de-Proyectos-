import { useEffect, useRef } from "react";
import type { ReactNode } from "react";
import { es } from "../i18n/es-MX";
import { useTheme } from "../lib/theme";
import { PersonMenu } from "../core/access/PersonMenu";
import type { SessionApi } from "../core/access/session";
import type { IconName } from "./icons";
import { accentOf } from "./modules";
import type { ModuleId } from "./modules";
import { SettingsMenu } from "./SettingsMenu";
import { Avatar, IconButton, LogoMark } from "./ui";

/** The sections of the bar: the core (Inicio, Mi institución, Documentos), the modules (ADR-032) and the settings. */
export type Page = "home" | "profile" | "documents" | "projects" | "staff" | "people" | "facilities" | "finance" | "ai" | "security" | "admin" | "help";

const CORE: [Page, string][] = [
  ["home", es.nav.home],
  ["profile", es.nav.profile],
  ["documents", es.nav.documents],
];
/** One tab per module of the institution (ADR-032). */
const MODULES: [ModuleId, string][] = [
  ["projects", es.nav.projects],
  ["staff", es.nav.staff],
  ["people", es.nav.people],
  ["facilities", es.nav.facilities],
  ["finance", es.nav.finance],
];
/** What the gear opens; some only for whoever may use them (ADR-028). */
const MORE: { page: Page; label: string; icon: IconName; needs: "settings" | "administer" | null }[] = [
  { page: "ai", label: es.nav.ai, icon: "sparkles", needs: "settings" },
  { page: "security", label: es.nav.security, icon: "shield", needs: null },
  { page: "admin", label: es.nav.admin, icon: "users", needs: "administer" },
  { page: "help", label: es.nav.help, icon: "help", needs: null },
];

/**
 * The layout of the whole program (docs/13 §6): a bar on top (the logo in the accent of the page, the sections with their names, and
 * on the right the theme, the settings and the person) and the page below it. The window carries the accent of the
 * page the person is in (docs/13 §3.7), which the pieces with tone="ac" take.
 */
export function Shell({
  page,
  onNavigate,
  institution,
  access,
  children,
}: {
  page: Page;
  onNavigate: (page: Page) => void;
  institution: string;
  /** the person inside: the menu shows what they may use, and lets them lock or close the session */
  access: SessionApi | null;
  children: ReactNode;
}) {
  const [theme, toggleTheme] = useTheme();
  const nav = useRef<HTMLElement>(null);

  // in a narrow window the tabs scroll: the one the person is in always stays in view
  useEffect(() => {
    nav.current?.querySelector<HTMLElement>('[aria-current="page"]')?.scrollIntoView?.({ inline: "nearest", block: "nearest" });
  }, [page]);

  const tab = ([id, label]: [Page, string]) => (
    <button key={id} type="button" className="topnav-tab" aria-current={page === id ? "page" : undefined} onClick={() => onNavigate(id)}>
      {label}
    </button>
  );
  const more = MORE.filter(({ needs }) => !needs || access?.can(needs));

  return (
    <div className={`shell accent-${accentOf(page)}`}>
      <div className="frame">
        <header className="topbar">
          <span className="topbar-mark">
            <LogoMark accent className="h-ctl-sm" />
          </span>
          <nav ref={nav} aria-label={es.nav.mainSections} className="topnav">
            <div role="group" aria-label={es.nav.coreGroup} className="topnav-group">
              {CORE.map(tab)}
            </div>
            <span className="topnav-sep" aria-hidden="true" />
            <div role="group" aria-label={es.nav.modulesGroup} className="topnav-group">
              {MODULES.map(tab)}
            </div>
          </nav>
          <div className="topbar-tools">
            <IconButton variant="plain" icon={theme === "dark" ? "sun" : "moon"} label={theme === "dark" ? es.nav.themeLight : es.nav.themeDark} onClick={toggleTheme} />
            <SettingsMenu items={more} current={page} onNavigate={onNavigate} />
            {access ? (
              <PersonMenu access={access} />
            ) : (
              <button type="button" onClick={() => onNavigate("profile")} aria-label={institution} title={institution} className="topbar-person">
                <Avatar name={institution} tone="violet" />
              </button>
            )}
          </div>
        </header>
        <main className="frame-main min-w-0">{children}</main>
      </div>
    </div>
  );
}
