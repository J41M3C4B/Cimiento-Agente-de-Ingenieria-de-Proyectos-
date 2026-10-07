import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import { Alert, Button, DropZone, Modal, Section, StatusDot, TextInput, PageHeader } from "../../components/ui";
import { extOf, fileSize } from "../documents/documentsModel";
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
      <p className="text-ui font-bold">
        <StatusDot tone={on ? "green" : "amber"}>{on ? t.pinOn : t.pinOff}</StatusDot>
      </p>
      <form
        className="space-y-4"
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
        className="space-y-4"
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
            <p className="mt-1 break-all text-small text-ink-2">{create.data.path}</p>
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
  // a new key empties the chosen file once it was restored
  const [picked, setPicked] = useState(0);
  const [file, setFile] = useState<File | null>(null);
  const [password, setPassword] = useState("");
  const [asking, setAsking] = useState(false);
  const restore = useMutation({
    mutationFn: async () => backupRestore(await toBase64(file as File), password),
    onSuccess: async () => {
      setAsking(false);
      setFile(null);
      setPassword("");
      setPicked((n) => n + 1);
      // everything on the screen came from the data that was just replaced
      await qc.invalidateQueries();
    },
    onError: () => setAsking(false),
  });

  return (
    <Section title={t.restoreTitle} help={t.restoreHelp}>
      <DropZone
        key={picked}
        label={t.restoreFile}
        accept=".cimiento"
        file={file ? { name: file.name, size: fileSize(file.size), ext: extOf(file.name) || "cimiento" } : null}
        changeHint={es.documents.form.change}
        onFile={setFile}
      />
      <TextInput label={t.restorePassword} type="password" autoComplete="off" value={password} onChange={(e) => setPassword(e.target.value)} />
      {restore.isError && <Alert tone="warn">{toAppError(restore.error).message}</Alert>}
      {restore.isSuccess && <Alert tone="ok">{t.restoreDone}</Alert>}
      <Button variant="danger" disabled={!file || password === "" || restore.isPending} onClick={() => setAsking(true)}>
        {restore.isPending ? t.restoring : t.restore}
      </Button>
      {asking && (
        <Modal
          title={t.restoreAsk}
          onClose={() => setAsking(false)}
          footer={
            <>
              <Button onClick={() => setAsking(false)}>{es.common.cancel}</Button>
              <Button variant="danger" onClick={() => restore.mutate()} disabled={restore.isPending}>
                {t.restoreYes}
              </Button>
            </>
          }
        >
          <p className="text-body">{t.restoreWarn}</p>
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
          <p className="font-bold">{t.scanFound(r.findings)}</p>
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
      <PageHeader title={t.title} intro={t.intro} />
      <div className="grid items-start gap-4 min-[1000px]:grid-cols-2">
        <div className="space-y-4">
          <PinSection />
          <BackupSection />
        </div>
        <div className="space-y-4">
          <RestoreSection />
          <ScanSection />
        </div>
      </div>
    </div>
  );
}
