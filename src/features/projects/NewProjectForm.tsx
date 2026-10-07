import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useRef, useState } from "react";
import { Icon } from "../../components/icons";
import { QuarantineDialog } from "../../components/QuarantineDialog";
import { Alert, Button, Card, FileTile, Inset, RadioCard, Select, Steps, Switch, Tag, TextInput } from "../../components/ui";
import { es } from "../../i18n/es-MX";
import { projectCreateFromCall, projectSetDonorKind, toAppError } from "../../lib/tauri";
import type { Decision, DonorKind, FileRole, ProjectRow, QuarantineReport } from "../../lib/types";

const t = es.projects;

export interface PackageItem {
  file: File;
  role: FileRole;
}

/** The roles a person can give to a file that is not the call itself. */
const EXTRA_ROLES: FileRole[] = ["annex", "guide", "form", "notice", "other"];

function toBase64(file: File): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => {
      const s = String(reader.result);
      resolve(s.slice(s.indexOf(",") + 1));
    };
    reader.onerror = () => reject(reader.error);
    reader.readAsDataURL(file);
  });
}

/** The files as the person picked them: the first one is taken as the call and the rest as annexes, until they say otherwise. */
export function itemsOf(files: File[]): PackageItem[] {
  return files.map((file, i) => ({ file, role: i === 0 ? "main" : "annex" }));
}

/** Marking a file as the call takes that mark off the one that had it: a package has exactly one. */
export function withRole(items: PackageItem[], index: number, role: FileRole): PackageItem[] {
  return items.map((it, i) => {
    if (i === index) return { ...it, role };
    return role === "main" && it.role === "main" ? { ...it, role: "annex" } : it;
  });
}

export function yearIsValid(text: string): boolean {
  return /^\d{4}$/.test(text.trim());
}

const size = (f: File) => (f.size >= 1024 * 1024 ? `${(f.size / 1024 / 1024).toFixed(1)} MB` : `${Math.max(1, Math.round(f.size / 1024))} KB`);

/**
 * Starts a project from its call, in two steps: the files (and what each one is), then the name, who gives it and
 * the year. One action at the end creates the project and starts reading the call in the background (ADR-016).
 */
export function NewProjectForm({ onCreated, onCancel }: { onCreated: (project: ProjectRow) => void; onCancel: () => void }) {
  const qc = useQueryClient();
  const input = useRef<HTMLInputElement>(null);
  const [step, setStep] = useState<0 | 1>(0);
  const [items, setItems] = useState<PackageItem[]>([]);
  const [hasExtras, setHasExtras] = useState(false);
  const [dragging, setDragging] = useState(false);
  const [name, setName] = useState("");
  const [funder, setFunder] = useState("");
  const [year, setYear] = useState("");
  const [donorKind, setDonorKind] = useState<DonorKind>("institutional");
  const [notice, setNotice] = useState<{ tone: "warn" | "error"; text: string } | null>(null);
  const [quarantine, setQuarantine] = useState<QuarantineReport | null>(null);

  const mains = items.filter((i) => i.role === "main").length;
  const yearOk = yearIsValid(year);
  const filesReady = items.length > 0 && mains === 1;
  const ready = filesReady && name.trim() !== "" && funder.trim() !== "" && yearOk;

  const create = useMutation({
    mutationFn: async (decision?: Decision) => {
      const files = await Promise.all(items.map(async (i) => ({ name: i.file.name, data: await toBase64(i.file), role: i.role })));
      return projectCreateFromCall(files, name, funder, Number(year), decision);
    },
    onSuccess: async (out) => {
      if (out.status === "created") {
        setQuarantine(null);
        setNotice(null);
        try {
          await projectSetDonorKind(out.project.id, donorKind);
        } catch {
          /* the project is made; the kind can be set later from its folder */
        }
        await qc.invalidateQueries({ queryKey: ["projects"] });
        onCreated(out.project);
      } else if (out.status === "quarantine") {
        setQuarantine(out.report);
      } else {
        // a file that cannot be read sends the person back to the files, where the problem is
        setQuarantine(null);
        setStep(0);
        const why = es.calls.unreadable[out.reason] ?? es.errors.generic;
        setNotice({ tone: "warn", text: es.calls.unreadableFile(out.file, why) });
      }
    },
    onError: (e) => {
      setQuarantine(null);
      setNotice({ tone: "error", text: toAppError(e).message });
    },
  });

  function choose(files: File[]) {
    if (files.length === 0) return;
    setItems(itemsOf(hasExtras ? files : files.slice(0, 1)));
    setNotice(null);
  }

  function toggleExtras(on: boolean) {
    setHasExtras(on);
    if (!on) {
      // without extras there is one file and it is the call
      setItems((prev) => itemsOf(prev.slice(0, 1).map((p) => p.file)));
      if (input.current) input.current.value = "";
    }
  }

  const inputLabel = hasExtras ? t.callFilesMore : t.callFiles;
  const ext = (f: File) => f.name.split(".").pop() ?? "";

  return (
    <div className="mx-auto w-full max-w-[768px] space-y-6">
      <Button variant="ghost" size="sm" onClick={step === 0 ? onCancel : () => setStep(0)}>
        <Icon name="back" size={16} />
        {step === 0 ? t.backToProjects : t.backToFiles}
      </Button>

      <header className="space-y-4">
        <div className="space-y-1">
          <h1 className="text-title font-bold tracking-tight">{t.newTitle}</h1>
          <p className="text-ui text-ink-3">{t.stepLabel(step + 1, 2, step === 0 ? t.step1 : t.step2)}</p>
        </div>
        <div className="max-w-sm">
          <Steps tone="ink" steps={[{ key: "files", label: t.filesStep }, { key: "data", label: t.dataStep }]} current={step} />
        </div>
      </header>

      {step === 0 && (
        <Card className="space-y-6">
          <div className="space-y-1">
            <h2 className="text-heading font-bold">{t.step1Title}</h2>
            <p className="text-ui text-ink-2">{t.step1Help}</p>
          </div>

          <label
            className={`dropzone ${dragging ? "dropzone--over" : ""}`}
            onDragOver={(e) => {
              e.preventDefault();
              setDragging(true);
            }}
            onDragLeave={() => setDragging(false)}
            onDrop={(e) => {
              e.preventDefault();
              setDragging(false);
              choose(Array.from(e.dataTransfer.files));
            }}
          >
            <input ref={input} id="call-files" type="file" multiple={hasExtras} accept=".pdf,.docx,.xlsx,.xlsm" aria-label={inputLabel} onChange={(e) => choose(Array.from(e.target.files ?? []))} />
            <span className="grid h-ctl w-ctl place-items-center rounded-pill bg-inset text-ink-2">
              <Icon name="upload" size={20} />
            </span>
            <b className="font-bold">{hasExtras ? t.dropTitleMany : t.dropTitle}</b>
            <span className="text-small text-ink-3">
              {t.dropOr} <span className="font-bold text-ink underline underline-offset-4">{t.dropPick}</span>
            </span>
          </label>

          <Switch label={t.hasExtras} role="switch" checked={hasExtras} onChange={(e) => toggleExtras(e.target.checked)} />
          {hasExtras && <p className="text-ui text-ink-2">{t.extrasHelp}</p>}

          {items.length > 0 && (
            <ul className="space-y-2">
              {items.map((it, i) => (
                <li key={`${it.file.name}-${i}`}>
                  <Inset className="flex flex-wrap items-center gap-3 !py-3">
                    <FileTile ext={ext(it.file)} />
                    <div className="min-w-0 flex-1">
                      <p className="truncate font-bold">{it.file.name}</p>
                      <p className="text-small text-ink-3">{size(it.file)}</p>
                    </div>
                    {hasExtras && (
                      <Select
                        label={t.roleLabel(it.file.name)}
                        hideLabel
                        className="w-full sm:w-56"
                        value={it.role}
                        onChange={(e) => setItems(withRole(items, i, e.target.value as FileRole))}
                        options={[["main", es.calls.roleOf("main")], ...EXTRA_ROLES.map((r): [string, string] => [r, es.calls.roleOf(r)])]}
                      />
                    )}
                  </Inset>
                </li>
              ))}
            </ul>
          )}
          {hasExtras && items.length > 0 && mains !== 1 && <Alert tone="warn">{t.pickOneMain}</Alert>}
          {notice && <Alert tone={notice.tone}>{notice.text}</Alert>}

          <div className="flex flex-wrap items-center justify-between gap-3 border-t border-line pt-6">
            <Button variant="ghost" onClick={onCancel}>
              {es.common.cancel}
            </Button>
            <Button variant="primary" disabled={!filesReady} onClick={() => setStep(1)}>
              {t.next}
              <Icon name="next" size={16} />
            </Button>
          </div>
        </Card>
      )}

      {step === 1 && (
        <form
          onSubmit={(e) => {
            e.preventDefault();
            if (ready) create.mutate(undefined);
          }}
        >
          <Card className="space-y-6">
            <Inset className="flex flex-wrap items-center gap-3 !py-3">
              <FileTile ext={ext(items[0]!.file)} />
              <div className="min-w-[9rem] flex-1">
                <p className="font-bold">{t.filesReady(items.length)}</p>
                <p className="truncate text-small text-ink-3">{items[0]?.file.name}</p>
              </div>
              <Tag tone="sky" variant="soft">
                {es.calls.roleOf("main")}
              </Tag>
            </Inset>

            <TextInput label={t.callName} hint={t.callNameHint} required value={name} onChange={(e) => setName(e.target.value)} />
            <div className="grid gap-6 sm:grid-cols-[1fr_auto]">
              <TextInput label={t.funder} hint={t.funderHint} required value={funder} onChange={(e) => setFunder(e.target.value)} />
              <TextInput
                label={t.year}
                hint={t.yearHint}
                required
                inputMode="numeric"
                maxLength={4}
                value={year}
                error={year !== "" && !yearOk ? t.yearInvalid : undefined}
                onChange={(e) => setYear(e.target.value)}
                className="sm:w-60"
              />
            </div>
            <fieldset>
              <legend className="field-label">{t.kindFieldLabel}</legend>
              <div className="grid gap-2 sm:grid-cols-3">
                {(Object.keys(t.kinds) as DonorKind[]).map((k) => (
                  <RadioCard key={k} name="donor-kind" title={t.kinds[k]} note={t.kindHelp[k]} checked={donorKind === k} onChange={() => setDonorKind(k)} />
                ))}
              </div>
            </fieldset>
            {notice && <Alert tone={notice.tone}>{notice.text}</Alert>}
            <Alert tone="info">
              <p className="text-ink-2">{t.startNote}</p>
            </Alert>

            <div className="flex flex-wrap items-center justify-between gap-3 border-t border-line pt-6">
              <Button variant="ghost" onClick={() => setStep(0)}>
                {t.wizardBack}
              </Button>
              <Button type="submit" variant="primary" disabled={create.isPending || !ready}>
                {create.isPending ? t.starting : t.start}
              </Button>
            </div>
          </Card>
        </form>
      )}

      {quarantine && (
        <QuarantineDialog
          report={quarantine}
          busy={create.isPending}
          onRedact={() => create.mutate("redact")}
          onNotPersonal={() => create.mutate("not_personal")}
          onCancel={() => setQuarantine(null)}
        />
      )}
    </div>
  );
}
