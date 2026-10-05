import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useRef, useState } from "react";
import { Alert, Button, Modal, Section, TextInput } from "../../components/ui";
import { es } from "../../i18n/es-MX";
import { backupCreate, backupRestore, pinClear, pinSet, pinStatus, securityScan, toAppError } from "../../lib/tauri";

const t = es.security;

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

function PinSection() {
  const qc = useQueryClient();
  const status = useQuery({ queryKey: ["pin-status"], queryFn: pinStatus });
  const [current, setCurrent] = useState("");
  const [pin, setPin] = useState("");
  const [again, setAgain] = useState("");
  const [notice, setNotice] = useState<{ tone: "ok" | "warn"; text: string } | null>(null);
  const on = status.data === true;

  const done = async (text: string) => {
    setCurrent("");
    setPin("");
    setAgain("");
    setNotice({ tone: "ok", text });
    await qc.invalidateQueries({ queryKey: ["pin-status"] });
  };
  const fail = (e: unknown) => setNotice({ tone: "warn", text: toAppError(e).message });

  const save = useMutation({ mutationFn: () => pinSet(pin, on ? current : undefined), onSuccess: () => done(t.pinSaved), onError: fail });
  const clear = useMutation({ mutationFn: () => pinClear(current), onSuccess: () => done(t.pinCleared), onError: fail });
  const differ = again !== "" && pin !== again;
  const canSave = pin !== "" && pin === again && (!on || current !== "");

  return (
    <Section title={t.pinTitle} help={t.pinHelp}>
      <p className="font-semibold">{on ? t.pinOn : t.pinOff}</p>
      <form
        className="space-y-3"
        onSubmit={(e) => {
          e.preventDefault();
          if (canSave) save.mutate();
        }}
      >
        {on && <TextInput label={t.currentPin} type="password" inputMode="numeric" autoComplete="off" maxLength={8} value={current} onChange={(e) => setCurrent(e.target.value)} />}
        <TextInput label={t.newPin} hint={t.newPinHint} type="password" inputMode="numeric" autoComplete="off" maxLength={8} value={pin} onChange={(e) => setPin(e.target.value)} />
        <TextInput
          label={t.repeatPin}
          type="password"
          inputMode="numeric"
          autoComplete="off"
          maxLength={8}
          value={again}
          error={differ ? t.pinsDiffer : undefined}
          onChange={(e) => setAgain(e.target.value)}
        />
        {notice && <Alert tone={notice.tone}>{notice.text}</Alert>}
        <div className="flex flex-wrap gap-3">
          <Button type="submit" variant="primary" disabled={!canSave || save.isPending}>
            {on ? t.changePin : t.setPin}
          </Button>
          {on && (
            <Button variant="danger" disabled={current === "" || clear.isPending} onClick={() => clear.mutate()}>
              {t.clearPin}
            </Button>
          )}
        </div>
      </form>
    </Section>
  );
}

function BackupSection() {
  const [password, setPassword] = useState("");
  const [again, setAgain] = useState("");
  const create = useMutation({ mutationFn: () => backupCreate(password) });
  const differ = again !== "" && password !== again;

  return (
    <Section title={t.backupTitle} help={t.backupHelp}>
      <form
        className="space-y-3"
        onSubmit={(e) => {
          e.preventDefault();
          if (password !== "" && password === again) create.mutate();
        }}
      >
        <TextInput label={t.password} hint={t.passwordHint} type="password" autoComplete="new-password" value={password} onChange={(e) => setPassword(e.target.value)} />
        <TextInput label={t.repeatPassword} type="password" autoComplete="new-password" value={again} error={differ ? t.passwordsDiffer : undefined} onChange={(e) => setAgain(e.target.value)} />
        {create.isError && <Alert tone="warn">{toAppError(create.error).message}</Alert>}
        {create.data && (
          <Alert tone="ok">
            <p>{t.backupDone(create.data.file_name)}</p>
            <p className="mt-1 break-all text-[14px] text-stone-700">{create.data.path}</p>
          </Alert>
        )}
        <Button type="submit" variant="primary" disabled={create.isPending || password === "" || password !== again}>
          {create.isPending ? t.creating : t.createBackup}
        </Button>
      </form>
    </Section>
  );
}

function RestoreSection() {
  const qc = useQueryClient();
  const input = useRef<HTMLInputElement>(null);
  const [file, setFile] = useState<File | null>(null);
  const [password, setPassword] = useState("");
  const [asking, setAsking] = useState(false);
  const restore = useMutation({
    mutationFn: async () => backupRestore(await toBase64(file as File), password),
    onSuccess: async () => {
      setAsking(false);
      setFile(null);
      setPassword("");
      if (input.current) input.current.value = "";
      // everything on the screen came from the data that was just replaced
      await qc.invalidateQueries();
    },
    onError: () => setAsking(false),
  });

  return (
    <Section title={t.restoreTitle} help={t.restoreHelp}>
      <label className="block">
        <span className="mb-1 block font-semibold">{t.restoreFile}</span>
        <input ref={input} type="file" accept=".cimiento" onChange={(e) => setFile(e.target.files?.[0] ?? null)} className="block w-full rounded-lg border-[1.5px] border-stone-400 bg-white p-3 text-[14px] file:mr-3 file:rounded-full file:border-0 file:bg-blue-50 file:px-4 file:py-2 file:font-semibold file:text-blue-800" />
      </label>
      <TextInput label={t.restorePassword} type="password" autoComplete="off" value={password} onChange={(e) => setPassword(e.target.value)} />
      {restore.isError && <Alert tone="warn">{toAppError(restore.error).message}</Alert>}
      {restore.isSuccess && <Alert tone="ok">{t.restoreDone}</Alert>}
      <Button variant="danger" disabled={!file || password === "" || restore.isPending} onClick={() => setAsking(true)}>
        {restore.isPending ? t.restoring : t.restore}
      </Button>
      {asking && (
        <Modal title={t.restoreAsk} onClose={() => setAsking(false)}>
          <p className="text-[15px]">{t.restoreWarn}</p>
          <div className="flex flex-wrap gap-3">
            <Button variant="danger" onClick={() => restore.mutate()} disabled={restore.isPending}>
              {t.restoreYes}
            </Button>
            <Button onClick={() => setAsking(false)}>{es.common.cancel}</Button>
          </div>
        </Modal>
      )}
    </Section>
  );
}

function ScanSection() {
  const scan = useMutation({ mutationFn: securityScan });
  const r = scan.data;
  return (
    <Section title={t.scanTitle} help={t.scanHelp}>
      <Button onClick={() => scan.mutate()} disabled={scan.isPending}>
        {scan.isPending ? t.scanning : t.scan}
      </Button>
      {scan.isError && <Alert tone="warn">{toAppError(scan.error).message}</Alert>}
      {r && r.findings === 0 && <Alert tone="ok">{t.scanClean(r.texts)}</Alert>}
      {r && r.findings > 0 && (
        <Alert tone="warn">
          <p className="font-semibold">{t.scanFound(r.findings)}</p>
          <ul className="mt-2 list-disc pl-6">
            {r.tables
              .filter((x) => x.findings > 0)
              .map((x) => (
                <li key={x.table}>{t.scanWhere(t.scanTables[x.table] ?? x.table, x.findings)}</li>
              ))}
          </ul>
        </Alert>
      )}
    </Section>
  );
}

export function SecurityPage() {
  return (
    <div className="space-y-6">
      <header className="space-y-1.5">
        <h1 className="text-[24px] font-semibold leading-tight tracking-tight">{t.title}</h1>
        <p className="text-stone-700">{t.intro}</p>
      </header>
      <PinSection />
      <BackupSection />
      <RestoreSection />
      <ScanSection />
    </div>
  );
}
