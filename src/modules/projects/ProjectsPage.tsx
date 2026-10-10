import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import { Icon } from "../../components/icons";
import { MODULE_META } from "../../components/modules";
import { Alert, Button, Folder, Modal, PageHeader } from "../../components/ui";
import { es } from "../../i18n/es-MX";
import { projectDelete, projectList, projectSetColor, projectSetDonorKind, toAppError } from "../../lib/tauri";
import type { DonorKind, ProjectColor, ProjectRow } from "../../lib/types";
import { ProjectFolder } from "./ProjectFolder";
import { NewProjectForm } from "./NewProjectForm";
import { ProjectPage } from "./ProjectPage";

const t = es.projects;

export function ProjectsPage({ openId, onOpen, creating, onCreating }: { openId: string | null; onOpen: (id: string | null) => void; creating: boolean; onCreating: (v: boolean) => void }) {
  const qc = useQueryClient();
  const projects = useQuery({ queryKey: ["projects"], queryFn: projectList });
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
            onOpen(null);
            void qc.invalidateQueries({ queryKey: ["projects"] });
          }}
        />
      </div>
    );
  }

  if (creating) {
    return (
      <NewProjectForm
        onCancel={() => onCreating(false)}
        onCreated={(p) => {
          onCreating(false);
          onOpen(p.id);
        }}
      />
    );
  }

  // the button of the project that is in progress (the most recent one not ready) is the main one
  const current = projects.data?.find((p) => p.stage !== "READY") ?? projects.data?.[0];

  return (
    <div className="space-y-6">
      <PageHeader
        title={t.title}
        intro={t.intro}
        action={
          <Button variant="primary" onClick={() => onCreating(true)}>
            <Icon name="plus" />
            {t.newProject}
          </Button>
        }
      />

      {notice && <Alert tone={notice.tone}>{notice.text}</Alert>}

      {projects.data?.length === 0 && (
        <Folder
          tone="ac"
          title={
            <span className="inline-flex items-center gap-2.5">
              <Icon name={MODULE_META.projects.icon} size={20} />
              {t.tab}
            </span>
          }
          bodyClassName="items-center !gap-3 py-12 text-center"
        >
          <span className="grid h-ctl w-ctl place-items-center rounded-pill bg-inset text-ink-2">
            <Icon name="folder" size={22} />
          </span>
          <p className="text-body font-bold">{t.empty}</p>
          <p className="max-w-md text-ui text-ink-2">{t.emptyHelp}</p>
          <Button variant="primary" onClick={() => onCreating(true)}>
            <Icon name="plus" />
            {t.newProject}
          </Button>
        </Folder>
      )}

      <ul className="grid gap-4 grid-cols-[repeat(auto-fill,minmax(min(100%,360px),1fr))]">
        {projects.data?.map((p) => (
          <ProjectFolder
            key={p.id}
            project={p}
            busy={remove.isPending}
            primary={p.id === current?.id}
            onOpen={() => onOpen(p.id)}
            onDelete={() => setToDelete(p)}
            onColor={(color) => paint.mutate({ id: p.id, color })}
            onKind={(kind) => classify.mutate({ id: p.id, kind })}
          />
        ))}
      </ul>

      {toDelete && (
        <Modal
          title={t.deleteTitle}
          onClose={() => setToDelete(null)}
          footer={
            <>
              <Button variant="ghost" onClick={() => setToDelete(null)} disabled={remove.isPending}>
                {es.common.cancel}
              </Button>
              <Button variant="danger" onClick={() => remove.mutate(toDelete.id)} disabled={remove.isPending}>
                {t.deleteConfirm}
              </Button>
            </>
          }
        >
          <p className="text-body font-bold">{toDelete.title}</p>
          <p className="text-body text-ink-2">{t.deleteBody}</p>
        </Modal>
      )}
    </div>
  );
}
