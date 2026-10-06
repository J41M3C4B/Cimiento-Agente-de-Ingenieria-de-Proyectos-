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
  const [menu, setMenu] = useState(false);
  // the screen is not shown until the PIN, if there is one, is entered
  const pin = useQuery({ queryKey: ["pin-status"], queryFn: pinStatus });
  const profile = useQuery({ queryKey: ["profile"], queryFn: profileGet, enabled: pin.data !== true });
  const [unlocked, setUnlocked] = useState(false);

  if (pin.isLoading) return <p className="p-6 text-lg">{es.common.loading}</p>;
  if (pin.data === true && !unlocked) return <LockScreen onUnlock={() => setUnlocked(true)} />;

  const institution = profile.data?.input.institution.name?.trim() || es.nav.profile;

  // Mi institución uses a wide sheet of its own
  const wide = page === "profile";

  const go = (id: Page) => {
    setPage(id);
    setMenu(false);
  };

  const tab = ([id, label]: [Page, string, IconName]) => (
    <button
      key={id}
      type="button"
      aria-current={page === id ? "page" : undefined}
      onClick={() => go(id)}
      className={`relative h-[52px] px-0.5 text-[14px] font-medium transition-colors ${page === id ? "text-stone-900" : "text-stone-500 hover:text-stone-900"}`}
    >
      {label}
      {page === id && <span aria-hidden="true" className="absolute inset-x-0 bottom-0 h-0.5 rounded-full bg-stone-900" />}
    </button>
  );

  return (
    <div className="min-h-screen bg-canvas text-stone-900">
      <header className="sticky top-0 z-40 border-b border-stone-300/50 bg-canvas/85 backdrop-blur">
        <div className="mx-auto flex h-[52px] max-w-[1440px] items-center gap-9 px-6">
          <div className="flex items-center gap-2.5">
            <span className="flex h-7 w-7 items-center justify-center rounded-lg bg-stone-900 text-white">
              <Icon name="logo" size={15} />
            </span>
            <p className="text-[15px] font-semibold tracking-tight">{es.app.name}</p>
          </div>
          <nav aria-label="Secciones" className="flex items-center gap-7">
            {MAIN.map(tab)}
          </nav>
          <div className="relative ml-auto">
            <button
              type="button"
              aria-label={es.nav.settings}
              aria-expanded={menu}
              title={institution}
              onClick={() => setMenu((v) => !v)}
              className="flex h-8 w-8 items-center justify-center rounded-full bg-white text-[13px] font-semibold text-stone-800 shadow-card transition-shadow hover:shadow-panel"
            >
              {institution.charAt(0).toUpperCase()}
            </button>
            {menu && (
              <>
                <div className="fixed inset-0 z-40" onClick={() => setMenu(false)} aria-hidden="true" />
                <div role="menu" className="absolute right-0 top-11 z-50 w-60 rounded-2xl bg-white p-1.5 shadow-lift">
                  <p className="truncate px-3 pb-2 pt-2.5 text-[12.5px] text-stone-500">{institution}</p>
                  {MORE.map(([id, label, icon]) => (
                    <button
                      key={id}
                      type="button"
                      role="menuitem"
                      onClick={() => go(id)}
                      className={`flex h-9 w-full items-center gap-3 rounded-lg px-3 text-left text-[14px] transition-colors hover:bg-stone-100 ${page === id ? "font-semibold" : ""}`}
                    >
                      <Icon name={icon} size={16} className="text-stone-500" />
                      {label}
                    </button>
                  ))}
                  {pin.data === true && (
                    <button
                      type="button"
                      role="menuitem"
                      onClick={() => {
                        setMenu(false);
                        setUnlocked(false);
                      }}
                      className="mt-1 flex h-9 w-full items-center gap-3 rounded-lg border-t border-stone-200 px-3 text-left text-[14px] transition-colors hover:bg-stone-100"
                    >
                      <Icon name="lock" size={16} className="text-stone-500" />
                      {es.nav.lock}
                    </button>
                  )}
                </div>
              </>
            )}
          </div>
        </div>
      </header>
      <main className="min-w-0">
        {/* Projects stays mounted while the person is in another section (only hidden): what the AI is doing for a
            project, the project that was open and what they were writing are still there when they come back */}
        <div hidden={page !== "projects"} className="mx-auto max-w-[1440px] px-6 py-6">
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
