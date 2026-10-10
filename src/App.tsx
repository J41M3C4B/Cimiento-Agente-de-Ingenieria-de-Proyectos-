import { useQuery } from "@tanstack/react-query";
import { useState } from "react";
import { Shell } from "./components/Shell";
import type { Page } from "./components/Shell";
import { AiSettingsPage } from "./core/ai/AiSettingsPage";
import { HomePage } from "./core/home/HomePage";
import { DocumentsPage } from "./core/documents/DocumentsPage";
import { HelpPage } from "./core/help/HelpPage";
import { ProfilePage } from "./core/profile/ProfilePage";
import { ProjectsPage } from "./modules/projects/ProjectsPage";
import { StaffPage } from "./modules/hr/StaffPage";
import { CarePage } from "./modules/care/CarePage";
import { FacilitiesPage } from "./modules/facilities/FacilitiesPage";
import { FinancePage } from "./modules/finance/FinancePage";
import { AdminPage } from "./core/access/AdminPage";
import { useSession } from "./core/access/session";
import { SecurityPage } from "./core/security/SecurityPage";
import { es } from "./i18n/es-MX";
import { profileGet } from "./lib/tauri";

export default function App() {
  const [page, setPage] = useState<Page>("home");
  const [openId, setOpenId] = useState<string | null>(null);
  const [creating, setCreating] = useState(false);
  // the app is shown only with a person inside (AccessGate, ADR-028); what they may do shapes the menu
  const access = useSession();
  const profile = useQuery({ queryKey: ["profile"], queryFn: profileGet });

  const institution = profile.data?.input.institution.name?.trim() || es.nav.profile;

  const openProject = (id: string) => {
    setOpenId(id);
    setCreating(false);
    setPage("projects");
  };
  const goProfile = () => setPage("profile");
  const navigate = (p: Page) => setPage(p);
  const newProject = () => {
    setOpenId(null);
    setCreating(true);
    setPage("projects");
  };

  return (
    <Shell page={page} onNavigate={navigate} institution={institution} access={access}>
      {/* Projects stays mounted while the person is in another section (only hidden): what the AI is doing for a
          project, the project that was open and what they were writing are still there when they come back */}
      <div hidden={page !== "projects"}>
        <ProjectsPage openId={openId} onOpen={setOpenId} creating={creating} onCreating={setCreating} />
      </div>
      {page === "home" && <HomePage onOpenProject={openProject} onNewProject={newProject} onGoProjects={() => setPage("projects")} onGoProfile={goProfile} onGo={navigate} onGoAi={() => setPage("ai")} />}
      {page === "profile" && <ProfilePage onGo={navigate} />}
      {page === "staff" && <StaffPage />}
      {page === "people" && <CarePage />}
      {page === "facilities" && <FacilitiesPage />}
      {page === "finance" && <FinancePage />}
      {page === "documents" && <DocumentsPage />}
      {page === "ai" && access?.can("settings") && <AiSettingsPage />}
      {page === "admin" && access?.can("administer") && <AdminPage />}
      {page === "security" && <SecurityPage />}
      {page === "help" && <HelpPage />}
    </Shell>
  );
}
