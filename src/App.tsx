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
import { AdminPage } from "./features/access/AdminPage";
import { useSession } from "./features/access/session";
import { SecurityPage } from "./features/security/SecurityPage";
import { es } from "./i18n/es-MX";
import { profileGet, projectList } from "./lib/tauri";

export default function App() {
  const [page, setPage] = useState<Page>("home");
  const [openId, setOpenId] = useState<string | null>(null);
  const [creating, setCreating] = useState(false);
  // the app is shown only with a person inside (AccessGate, ADR-028); what they may do shapes the menu
  const access = useSession();
  const profile = useQuery({ queryKey: ["profile"], queryFn: profileGet });
  const projects = useQuery({ queryKey: ["projects"], queryFn: projectList });

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
    <Shell page={page} onNavigate={setPage} institution={institution} focus={focus} onOpenFocus={openFocus} access={access}>
      {/* Projects stays mounted while the person is in another section (only hidden): what the AI is doing for a
          project, the project that was open and what they were writing are still there when they come back */}
      <div hidden={page !== "projects"}>
        <ProjectsPage openId={openId} onOpen={setOpenId} creating={creating} onCreating={setCreating} />
      </div>
      {page === "home" && <HomePage onOpenProject={openProject} onNewProject={newProject} onGoProjects={() => setPage("projects")} onGoProfile={() => setPage("profile")} />}
      {page === "profile" && <ProfilePage />}
      {page === "documents" && <DocumentsPage />}
      {page === "ai" && access?.can("settings") && <AiSettingsPage />}
      {page === "admin" && access?.can("administer") && <AdminPage />}
      {page === "security" && <SecurityPage />}
      {page === "help" && <HelpPage />}
    </Shell>
  );
}
