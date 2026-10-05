import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useRef, useState } from "react";
import { Icon } from "../../components/icons";
import { QuarantineDialog } from "../../components/QuarantineDialog";
import { Alert, Button, Chip, Steps, TextInput } from "../../components/ui";
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

  return (
    <div className="mx-auto max-w-3xl space-y-6">
      <button type="button" onClick={step === 0 ? onCancel : () => setStep(0)} className="inline-flex items-center gap-1.5 text-[14px] font-semibold text-stone-700 hover:text-blue-800">
        <Icon name="back" size={16} />
        {step === 0 ? t.backToProjects : t.backToFiles}
      </button>

      <header className="space-y-5">
        <div className="space-y-1.5">
          <h1 className="text-[24px] font-semibold leading-tight tracking-tight">{t.newTitle}</h1>
          <p className="text-stone-700">{t.stepLabel(step + 1, 2, step === 0 ? t.step1 : t.step2)}</p>
        </div>
        <div className="max-w-sm">
          <Steps steps={[{ key: "files", label: t.filesStep }, { key: "data", label: t.dataStep }]} current={step} />
        </div>
      </header>

      {step === 0 && (
        <>
          <section className="space-y-5 rounded-xl border border-stone-200 bg-white p-7 shadow-card">
            <div className="space-y-1">
              <h2 className="text-[18px] font-semibold">{t.step1Title}</h2>
              <p className="text-stone-700">{t.step1Help}</p>
            </div>

            <div>
              <input
                id="call-files"
                ref={input}
                type="file"
                multiple={hasExtras}
                accept=".pdf,.docx,.xlsx,.xlsm"
                aria-label={inputLabel}
                onChange={(e) => choose(Array.from(e.target.files ?? []))}
                className="peer sr-only"
              />
              <label
                htmlFor="call-files"
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
                className={`flex cursor-pointer flex-col items-center gap-2.5 rounded-xl border-2 border-dashed px-6 py-10 text-center transition-colors peer-focus-visible:ring-4 peer-focus-visible:ring-blue-100 ${
                  dragging ? "border-blue-800 bg-blue-50" : "border-stone-500 bg-stone-50 hover:border-blue-800 hover:bg-blue-50"
                }`}
              >
                <span className="flex h-14 w-14 items-center justify-center rounded-full bg-blue-50 text-blue-800">
                  <Icon name="upload" size={26} />
                </span>
                <span className="text-[15px] font-semibold">{hasExtras ? t.dropTitleMany : t.dropTitle}</span>
                <span className="text-stone-700">
                  {t.dropOr} <span className="font-semibold text-blue-800">{t.dropPick}</span>
                </span>
              </label>
            </div>

            <label className="flex cursor-pointer items-center gap-3 font-semibold">
              <input type="checkbox" role="switch" className="peer sr-only" checked={hasExtras} onChange={(e) => toggleExtras(e.target.checked)} />
              <span
                aria-hidden="true"
                className={`relative h-7 w-12 shrink-0 rounded-full transition-colors peer-focus-visible:ring-4 peer-focus-visible:ring-blue-100 ${hasExtras ? "bg-blue-800" : "bg-stone-400"}`}
              >
                <span className={`absolute top-[3px] h-[22px] w-[22px] rounded-full bg-white shadow transition-all ${hasExtras ? "left-[23px]" : "left-[3px]"}`} />
              </span>
              {t.hasExtras}
            </label>
            {hasExtras && <p className="text-[14px] text-stone-700">{t.extrasHelp}</p>}

            {items.length > 0 && (
              <ul className="space-y-2.5">
                {items.map((it, i) => (
                  <li key={`${it.file.name}-${i}`} className="flex flex-wrap items-center gap-3.5 rounded-lg border border-stone-200 bg-white p-3.5">
                    <span className="flex h-10 w-10 shrink-0 items-center justify-center rounded-lg bg-blue-50 text-blue-800">
                      <Icon name="file" />
                    </span>
                    <div className="min-w-0 flex-1">
                      <p className="truncate font-semibold">{it.file.name}</p>
                      <p className="text-[13px] text-stone-700">{size(it.file)}</p>
                    </div>
                    {hasExtras && (
                      <div className="relative w-56">
                        <select
                          aria-label={t.roleLabel(it.file.name)}
                          value={it.role}
                          onChange={(e) => setItems(withRole(items, i, e.target.value as FileRole))}
                          className="min-h-11 w-full appearance-none rounded-lg border-[1.5px] border-stone-400 bg-white pl-4 pr-10 text-[14px] focus-visible:border-blue-800 focus-visible:outline-none focus-visible:ring-4 focus-visible:ring-blue-100"
                        >
                          <option value="main">{es.calls.roleOf("main")}</option>
                          {EXTRA_ROLES.map((r) => (
                            <option key={r} value={r}>
                              {es.calls.roleOf(r)}
                            </option>
                          ))}
                        </select>
                        <Icon name="down" size={16} className="pointer-events-none absolute right-3.5 top-1/2 -translate-y-1/2 text-stone-700" />
                      </div>
                    )}
                  </li>
                ))}
              </ul>
            )}
            {hasExtras && items.length > 0 && mains !== 1 && <Alert tone="warn">{t.pickOneMain}</Alert>}
            {notice && <Alert tone={notice.tone}>{notice.text}</Alert>}
          </section>

          <div className="flex flex-wrap items-center justify-between gap-3 rounded-xl border border-stone-200 bg-white px-6 py-4 shadow-lift">
            <Button variant="ghost" onClick={onCancel}>
              {es.common.cancel}
            </Button>
            <Button variant="primary" disabled={!filesReady} onClick={() => setStep(1)}>
              {t.next}
              <Icon name="next" size={16} />
            </Button>
          </div>
        </>
      )}

      {step === 1 && (
        <form
          className="space-y-6"
          onSubmit={(e) => {
            e.preventDefault();
            if (ready) create.mutate(undefined);
          }}
        >
          <section className="space-y-5 rounded-xl border border-stone-200 bg-white p-7 shadow-card">
            <div className="flex flex-wrap items-center gap-3.5 rounded-lg bg-stone-50 p-3.5">
              <span className="flex h-10 w-10 shrink-0 items-center justify-center rounded-lg bg-blue-50 text-blue-800">
                <Icon name="file" />
              </span>
              <div className="min-w-0 flex-1">
                <p className="font-semibold">{t.filesReady(items.length)}</p>
                <p className="truncate text-[13px] text-stone-700">{items[0]?.file.name}</p>
              </div>
              <Chip tone="blue">{es.calls.roleOf("main")}</Chip>
            </div>

            <TextInput label={t.callName} hint={t.callNameHint} value={name} onChange={(e) => setName(e.target.value)} />
            <div className="grid gap-5 sm:grid-cols-[1fr_auto]">
              <TextInput label={t.funder} hint={t.funderHint} value={funder} onChange={(e) => setFunder(e.target.value)} />
              <TextInput
                label={t.year}
                hint={t.yearHint}
                inputMode="numeric"
                maxLength={4}
                value={year}
                error={year !== "" && !yearOk ? t.yearInvalid : undefined}
                onChange={(e) => setYear(e.target.value)}
                className="sm:w-44"
              />
            </div>
            <fieldset>
              <legend className="mb-1.5 text-[13px] font-medium text-stone-700">{t.kindFieldLabel}</legend>
              <div className="grid gap-2 sm:grid-cols-3">
                {(Object.keys(t.kinds) as DonorKind[]).map((k) => (
                  <label
                    key={k}
                    className="flex cursor-pointer flex-col gap-0.5 rounded-lg border border-stone-300 px-3.5 py-3 transition-colors hover:border-stone-400 has-[:checked]:border-blue-800 has-[:checked]:bg-blue-50 has-[:focus-visible]:ring-[3px] has-[:focus-visible]:ring-blue-100"
                  >
                    <span className="flex items-center gap-2.5 text-[14px] font-semibold">
                      <input type="radio" name="donor-kind" className="h-4 w-4 shrink-0 accent-blue-800" checked={donorKind === k} onChange={() => setDonorKind(k)} />
                      {t.kinds[k]}
                    </span>
                    <span className="pl-[26px] text-[12.5px] text-stone-600">{t.kindHelp[k]}</span>
                  </label>
                ))}
              </div>
            </fieldset>
            {notice && <Alert tone={notice.tone}>{notice.text}</Alert>}
            <Alert tone="info">
              <p className="text-stone-700">{t.startNote}</p>
            </Alert>
          </section>

          <div className="flex flex-wrap items-center justify-between gap-3 rounded-xl border border-stone-200 bg-white px-6 py-4 shadow-lift">
            <Button variant="ghost" onClick={() => setStep(0)}>
              {t.wizardBack}
            </Button>
            <Button type="submit" variant="primary" size="lg" disabled={create.isPending || !ready}>
              {create.isPending ? t.starting : t.start}
            </Button>
          </div>
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
