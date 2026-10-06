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

  // The label of a link or block: on wide screens the bar is always open; on narrow ones it is a thin rail that
  // opens (hover or keyboard focus) over the page
  const reveal = "whitespace-nowrap opacity-0 transition-opacity duration-150 xl:opacity-100 group-hover/side:opacity-100 group-has-[:focus-visible]/side:opacity-100";

  // Mi institución uses the whole width
  const wide = page === "profile";

  const link = ([id, label, icon]: [Page, string, IconName]) => (
    <button
      key={id}
      type="button"
      title={label}
      aria-current={page === id ? "page" : undefined}
      onClick={() => setPage(id)}
      className={`flex h-10 w-full items-center gap-3 rounded-xl px-3 text-left text-[14px] transition-colors ${
        page === id ? "bg-stone-100 font-semibold text-stone-900" : "font-medium text-stone-600 hover:bg-stone-50 hover:text-stone-900"
      }`}
    >
      <Icon name={icon} size={18} className={page === id ? "shrink-0 text-stone-900" : "shrink-0 text-stone-500"} />
      <span className={reveal}>{label}</span>
    </button>
  );

  return (
    <div className="min-h-screen bg-canvas text-stone-900">
      <aside
        aria-label="Menú"
        className="peer/side group/side fixed bottom-3 left-3 top-3 z-40 flex w-[64px] flex-col overflow-hidden rounded-3xl bg-white p-3 shadow-panel transition-[width,box-shadow] duration-200 ease-out hover:w-[232px] hover:shadow-lift has-[:focus-visible]:w-[232px] has-[:focus-visible]:shadow-lift xl:w-[232px]"
      >
        <div className="flex items-center gap-2.5 pb-6 pl-0.5 pt-1">
          <span className="flex h-9 w-9 shrink-0 items-center justify-center rounded-[11px] bg-stone-900 text-white">
            <Icon name="logo" size={18} />
          </span>
          <p className={`text-[16px] font-semibold tracking-tight ${reveal}`}>{es.app.name}</p>
        </div>
        <nav aria-label="Secciones" className="flex flex-col gap-1">
          {MAIN.map(link)}
        </nav>
        <nav aria-label={es.nav.settings} className="mt-auto flex flex-col gap-1 pb-3">
          {MORE.map(link)}
        </nav>
        <div className="flex items-center gap-2.5 border-t border-stone-200 pt-3">
          <span className="flex h-9 w-9 shrink-0 items-center justify-center rounded-full bg-stone-100 text-[13px] font-semibold text-stone-800" aria-hidden="true">
            {institution.charAt(0).toUpperCase()}
          </span>
          <div className={`min-w-0 flex-1 ${reveal}`}>
            <p className="truncate text-[13px] font-semibold">{institution}</p>
            {pin.data === true && (
              <button type="button" onClick={() => setUnlocked(false)} className="flex items-center gap-1 text-[12px] font-medium text-stone-500 hover:text-stone-900">
                <Icon name="lock" size={13} />
                {es.nav.lock}
              </button>
            )}
          </div>
        </div>
      </aside>
      {/* when the thin bar opens over the page, the page steps back so what it covers does not look cut off */}
      <div
        aria-hidden="true"
        className="pointer-events-none fixed inset-0 z-30 bg-stone-900/20 opacity-0 transition-opacity duration-200 peer-hover/side:opacity-100 peer-has-[:focus-visible]/side:opacity-100 xl:hidden"
      />
      <main className="min-w-0 pl-[88px] xl:pl-[256px]">
        {/* Projects stays mounted while the person is in another section (only hidden): what the AI is doing for a
            project, the project that was open and what they were writing are still there when they come back */}
        <div hidden={page !== "projects"} className="px-6 py-6">
          <ProjectsPage />
        </div>
        {wide ? (
          <>{page === "profile" && <ProfilePage />}</>
        ) : (
          page !== "projects" && (
            <div className="mx-auto max-w-5xl px-6 py-8">
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
