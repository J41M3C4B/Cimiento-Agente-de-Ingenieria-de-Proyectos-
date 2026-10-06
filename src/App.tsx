import { useQuery } from "@tanstack/react-query";
import { useState } from "react";
import { Icon } from "./components/icons";
import type { IconName } from "./components/icons";
import { AiSettingsPage } from "./features/ai/AiSettingsPage";
import { DocumentsPage } from "./features/documents/DocumentsPage";
import { HelpPage } from "./features/help/HelpPage";
import { ProfilePage } from "./features/profile/ProfilePage";
import { ProjectsPage } from "./features/projects/ProjectsPage";
import { LockScreen } from "./features/security/LockScreen";
import { SecurityPage } from "./features/security/SecurityPage";
import { es } from "./i18n/es-MX";
import { pinStatus, profileGet } from "./lib/tauri";

type Page = "projects" | "profile" | "documents" | "ai" | "security" | "help";

const MAIN: [Page, string, IconName][] = [
  ["projects", es.nav.projects, "folder"],
  ["profile", es.nav.profile, "building"],
  ["documents", es.nav.documents, "file"],
];
const MORE: [Page, string, IconName][] = [
  ["ai", es.nav.ai, "sparkles"],
  ["security", es.nav.security, "shield"],
  ["help", es.nav.help, "help"],
];

export default function App() {
  const [page, setPage] = useState<Page>("projects");
  // the screen is not shown until the PIN, if there is one, is entered
  const pin = useQuery({ queryKey: ["pin-status"], queryFn: pinStatus });
  const profile = useQuery({ queryKey: ["profile"], queryFn: profileGet, enabled: pin.data !== true });
  const [unlocked, setUnlocked] = useState(false);

  if (pin.isLoading) return <p className="p-6 text-lg">{es.common.loading}</p>;
  if (pin.data === true && !unlocked) return <LockScreen onUnlock={() => setUnlocked(true)} />;

  const institution = profile.data?.input.institution.name?.trim() || es.nav.profile;

  // The label of a link or block: hidden while the bar is a thin rail, shown when it opens (hover or keyboard focus)
  const reveal = "whitespace-nowrap opacity-0 transition-opacity duration-150 group-hover/side:opacity-100 group-has-[:focus-visible]/side:opacity-100";

  // Mi institución uses the whole width, on a white surface
  const wide = page === "profile";

  const link = ([id, label, icon]: [Page, string, IconName]) => (
    <button
      key={id}
      type="button"
      title={label}
      aria-current={page === id ? "page" : undefined}
      onClick={() => setPage(id)}
      className={`relative flex h-11 w-full items-center gap-3 rounded-xl px-4 text-left text-[14px] font-medium transition-colors ${
        page === id ? "bg-blue-50 text-navy-900" : "text-stone-700 hover:bg-stone-100 hover:text-stone-900"
      }`}
    >
      {page === id && <span aria-hidden="true" className="absolute -left-3 top-1/2 h-6 w-[3px] -translate-y-1/2 rounded-r-full bg-brass-500" />}
      <Icon name={icon} className={page === id ? "shrink-0 text-navy-800" : "shrink-0"} />
      <span className={reveal}>{label}</span>
    </button>
  );

  return (
    <div className="min-h-screen bg-white text-stone-900">
      <aside
        aria-label="Menú"
        className="peer/side group/side fixed inset-y-0 left-0 z-40 flex w-[76px] flex-col gap-1 overflow-hidden border-r border-stone-200 bg-stone-50 p-3 transition-[width,box-shadow] duration-200 ease-out hover:w-64 hover:shadow-lift has-[:focus-visible]:w-64 has-[:focus-visible]:shadow-lift"
      >
        <div className="flex items-center gap-3 pb-5 pl-1.5 pt-2">
          <span className="flex h-10 w-10 shrink-0 items-center justify-center rounded-xl bg-gradient-to-br from-navy-700 to-navy-950 text-brass-300 shadow-brand">
            <Icon name="logo" />
          </span>
          <p className={`font-display text-[22px] font-semibold leading-tight tracking-tight text-navy-900 ${reveal}`}>{es.app.name}</p>
        </div>
        <nav aria-label="Secciones" className="flex flex-col gap-1">
          {MAIN.map(link)}
          <div className="relative h-10">
            <span aria-hidden="true" className="absolute inset-x-3 top-1/2 h-px bg-stone-300 transition-opacity group-hover/side:opacity-0 group-has-[:focus-visible]/side:opacity-0" />
            <p className={`absolute inset-x-4 bottom-1.5 text-[11px] font-semibold uppercase tracking-[0.1em] text-stone-600 ${reveal}`}>{es.nav.settings}</p>
          </div>
          {MORE.map(link)}
        </nav>
        <div className="mt-auto flex items-center gap-3 rounded-xl border border-stone-200 bg-white p-[7px]">
          <span className="flex h-9 w-9 shrink-0 items-center justify-center rounded-lg bg-gradient-to-br from-navy-700 to-navy-900 font-display text-[16px] font-semibold text-white" aria-hidden="true">
            {institution.charAt(0).toUpperCase()}
          </span>
          <div className={`min-w-0 flex-1 ${reveal}`}>
            <p className="truncate text-[13px] font-semibold">{institution}</p>
            {pin.data === true && (
              <button type="button" onClick={() => setUnlocked(false)} className="flex items-center gap-1 text-[12px] font-medium text-blue-800 hover:underline">
                <Icon name="lock" size={13} />
                {es.nav.lock}
              </button>
            )}
          </div>
        </div>
      </aside>
      {/* when the bar opens over the page, the page steps back so what it covers does not look cut off */}
      <div
        aria-hidden="true"
        className="pointer-events-none fixed inset-0 z-30 bg-navy-950/25 opacity-0 transition-opacity duration-200 peer-hover/side:opacity-100 peer-has-[:focus-visible]/side:opacity-100"
      />
      <main className="min-w-0 pl-[76px]">
        {/* Projects stays mounted while the person is in another section (only hidden): what the AI is doing for a
            project, the project that was open and what they were writing are still there when they come back */}
        <div hidden={page !== "projects"} className="px-8 py-7">
          <ProjectsPage />
        </div>
        {wide ? (
          <>{page === "profile" && <ProfilePage />}</>
        ) : (
          page !== "projects" && (
            <div className="mx-auto max-w-5xl px-8 py-8">
              {page === "documents" && <DocumentsPage />}
              {page === "ai" && <AiSettingsPage />}
              {page === "security" && <SecurityPage />}
              {page === "help" && <HelpPage />}
            </div>
          )
        )}
      </main>
    </div>
  );
}
