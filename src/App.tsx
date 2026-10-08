import { useQuery } from "@tanstack/react-query";
import { useState } from "react";
import { Shell } from "./components/Shell";
import type { Page } from "./components/Shell";
import { AiSettingsPage } from "./features/ai/AiSettingsPage";
import { HomePage } from "./features/home/HomePage";
import { DocumentsPage } from "./features/documents/DocumentsPage";
import { HelpPage } from "./features/help/HelpPage";
import { ProfilePage } from "./features/profile/ProfilePage";
import { ProjectsPage } from "./features/projects/ProjectsPage";
import { Onboarding } from "./features/onboarding/Onboarding";
import { LockScreen } from "./features/security/LockScreen";
import { SecurityPage } from "./features/security/SecurityPage";
import { es } from "./i18n/es-MX";
import { pinStatus, profileGet, projectList } from "./lib/tauri";

export default function App() {
  const [page, setPage] = useState<Page>("home");
  const [openId, setOpenId] = useState<string | null>(null);
  const [creating, setCreating] = useState(false);
  // the screen is not shown until the PIN, if there is one, is entered
  const pin = useQuery({ queryKey: ["pin-status"], queryFn: pinStatus });
  const profile = useQuery({ queryKey: ["profile"], queryFn: profileGet, enabled: pin.data !== true });
  const projects = useQuery({ queryKey: ["projects"], queryFn: projectList, enabled: pin.data !== true });
  const [unlocked, setUnlocked] = useState(false);

  if (pin.isLoading) return <p className="p-6 text-heading">{es.common.loading}</p>;
  if (pin.data === true && !unlocked) return <LockScreen onUnlock={() => setUnlocked(true)} />;

  // the first time, before the institution is registered, the person is welcomed instead of landing in an empty program
  if (profile.isSuccess && profile.data === null) return <Onboarding />;

  const institution = profile.data?.input.institution.name?.trim() || es.nav.profile;

  // the capsule talks about the project that is open or, failing that, the one in progress
  const focus = projects.data?.find((p) => p.id === openId) ?? projects.data?.find((p) => p.stage !== "READY") ?? projects.data?.[0];
  const openProject = (id: string) => {
    setOpenId(id);
    setCreating(false);
    setPage("projects");
  };
  const openFocus = () => focus && openProject(focus.id);
  const newProject = () => {
    setOpenId(null);
    setCreating(true);
    setPage("projects");
  };

  return (
    <Shell page={page} onNavigate={setPage} institution={institution} focus={focus} onOpenFocus={openFocus} locked={pin.data === true} onLock={() => setUnlocked(false)}>
      {/* Projects stays mounted while the person is in another section (only hidden): what the AI is doing for a
          project, the project that was open and what they were writing are still there when they come back */}
      <div hidden={page !== "projects"}>
        <ProjectsPage openId={openId} onOpen={setOpenId} creating={creating} onCreating={setCreating} />
      </div>
      {page === "home" && <HomePage onOpenProject={openProject} onNewProject={newProject} onGoProjects={() => setPage("projects")} onGoProfile={() => setPage("profile")} />}
      {page === "profile" && <ProfilePage />}
      {page === "documents" && <DocumentsPage />}
      {page === "ai" && <AiSettingsPage />}
      {page === "security" && <SecurityPage />}
      {page === "help" && <HelpPage />}
    </Shell>
  );
}
