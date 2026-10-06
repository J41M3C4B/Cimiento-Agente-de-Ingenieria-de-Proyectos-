import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import { Icon } from "../../components/icons";
import { Alert, Button, Modal } from "../../components/ui";
import { es } from "../../i18n/es-MX";
import { projectDelete, projectList, projectSetColor, projectSetDonorKind, toAppError } from "../../lib/tauri";
import type { DonorKind, ProjectColor, ProjectRow } from "../../lib/types";
import { ProjectFolder } from "./ProjectFolder";
import { NewProjectForm } from "./NewProjectForm";
import { ProjectPage } from "./ProjectPage";

const t = es.projects;

export function ProjectsPage() {
  const qc = useQueryClient();
  const projects = useQuery({ queryKey: ["projects"], queryFn: projectList });
  const [openId, setOpenId] = useState<string | null>(null);
  const [creating, setCreating] = useState(false);
  const [toDelete, setToDelete] = useState<ProjectRow | null>(null);
  const [notice, setNotice] = useState<{ tone: "ok" | "error"; text: string } | null>(null);

  const remove = useMutation({
    mutationFn: (id: string) => projectDelete(id),
    onSuccess: async () => {
      setToDelete(null);
      setNotice({ tone: "ok", text: t.deleted });
      await qc.invalidateQueries({ queryKey: ["projects"] });
      // the call of the project went with it
      await qc.invalidateQueries({ queryKey: ["call-reading"] });
    },
    onError: (e) => {
      setToDelete(null);
      setNotice({ tone: "error", text: toAppError(e).message });
    },
  });

  const paint = useMutation({
    mutationFn: ({ id, color }: { id: string; color: ProjectColor }) => projectSetColor(id, color),
    onMutate: ({ id, color }) => {
      qc.setQueryData<ProjectRow[]>(["projects"], (rows) => rows?.map((p) => (p.id === id ? { ...p, color } : p)));
    },
    onError: async (e) => {
      setNotice({ tone: "error", text: toAppError(e).message });
      await qc.invalidateQueries({ queryKey: ["projects"] });
    },
  });

  const classify = useMutation({
    mutationFn: ({ id, kind }: { id: string; kind: DonorKind }) => projectSetDonorKind(id, kind),
    onMutate: ({ id, kind }) => {
      qc.setQueryData<ProjectRow[]>(["projects"], (rows) => rows?.map((p) => (p.id === id ? { ...p, donor_kind: kind } : p)));
    },
    onError: async (e) => {
      setNotice({ tone: "error", text: toAppError(e).message });
      await qc.invalidateQueries({ queryKey: ["projects"] });
    },
  });

  if (openId) {
    return (
      <div>
        <ProjectPage
          projectId={openId}
          onBack={() => {
            setOpenId(null);
            void qc.invalidateQueries({ queryKey: ["projects"] });
          }}
        />
      </div>
    );
  }

  if (creating) {
    return (
      <div className="mx-auto max-w-5xl">
        <NewProjectForm
          onCancel={() => setCreating(false)}
          onCreated={(p) => {
            setCreating(false);
            setOpenId(p.id);
          }}
        />
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <header className="flex flex-wrap items-end justify-between gap-4">
        <div className="space-y-1.5">
          <h1 className="text-[34px] font-medium leading-tight">{t.title}</h1>
          <p className="text-stone-600">{t.intro}</p>
        </div>
        <Button variant="primary" onClick={() => setCreating(true)}>
          <Icon name="plus" />
          {t.newProject}
        </Button>
      </header>

      {notice && <Alert tone={notice.tone}>{notice.text}</Alert>}

      {projects.data?.length === 0 && (
        <div className="flex flex-col items-center gap-3 rounded-2xl bg-white px-6 py-14 shadow-card text-center">
          <span className="flex h-14 w-14 items-center justify-center rounded-full bg-stone-100 text-stone-800">
            <Icon name="folder" size={26} />
          </span>
          <p className="text-[15px] font-semibold">{t.empty}</p>
          <p className="max-w-md text-stone-700">{t.emptyHelp}</p>
          <Button variant="primary" onClick={() => setCreating(true)}>
            <Icon name="plus" />
            {t.newProject}
          </Button>
        </div>
      )}

      <ul className="grid gap-x-5 gap-y-6 sm:grid-cols-2 lg:grid-cols-3">
        {projects.data?.map((p) => (
          <ProjectFolder
            key={p.id}
            project={p}
            busy={remove.isPending}
            onOpen={() => setOpenId(p.id)}
            onDelete={() => setToDelete(p)}
            onColor={(color) => paint.mutate({ id: p.id, color })}
            onKind={(kind) => classify.mutate({ id: p.id, kind })}
          />
        ))}
      </ul>

      {toDelete && (
        <Modal title={t.deleteTitle} onClose={() => setToDelete(null)}>
          <p className="text-[15px]">
            <strong>{toDelete.title}</strong>
          </p>
          <p className="text-[15px]">{t.deleteBody}</p>
          <div className="flex flex-wrap gap-3">
            <Button variant="danger" onClick={() => remove.mutate(toDelete.id)} disabled={remove.isPending}>
              {t.deleteConfirm}
            </Button>
            <Button onClick={() => setToDelete(null)} disabled={remove.isPending}>
              {es.common.cancel}
            </Button>
          </div>
        </Modal>
      )}
    </div>
  );
}
