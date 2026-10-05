import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";

vi.mock("../../lib/tauri", () => ({
  pinStatus: vi.fn(),
  pinSet: vi.fn(),
  pinClear: vi.fn(),
  pinVerify: vi.fn(),
  securityScan: vi.fn(),
  backupCreate: vi.fn(),
  backupRestore: vi.fn(),
  toAppError: (e: unknown) => ({ code: "x", message: e instanceof Error ? e.message : String(e) }),
}));

import * as api from "../../lib/tauri";
import { LockScreen } from "./LockScreen";
import { SecurityPage } from "./SecurityPage";

function show(ui: React.ReactElement) {
  const qc = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  render(<QueryClientProvider client={qc}>{ui}</QueryClientProvider>);
}

beforeEach(() => {
  vi.resetAllMocks();
  vi.mocked(api.pinStatus).mockResolvedValue(false);
});

describe("LockScreen", () => {
  it("lets the person in with the right PIN", async () => {
    vi.mocked(api.pinVerify).mockResolvedValue({ status: "ok" });
    const onUnlock = vi.fn();
    render(<LockScreen onUnlock={onUnlock} />);
    await userEvent.type(screen.getByLabelText("PIN"), "4821{Enter}");
    await waitFor(() => expect(api.pinVerify).toHaveBeenCalledWith("4821"));
    expect(onUnlock).toHaveBeenCalled();
  });

  it("says a wrong PIN is wrong and clears it, and tells how long to wait after too many tries", async () => {
    vi.mocked(api.pinVerify).mockResolvedValueOnce({ status: "wrong" }).mockResolvedValueOnce({ status: "locked", wait_secs: 30 });
    const onUnlock = vi.fn();
    render(<LockScreen onUnlock={onUnlock} />);
    await userEvent.type(screen.getByLabelText("PIN"), "0000{Enter}");
    expect(await screen.findByText("Ese PIN no es el correcto.")).toBeInTheDocument();
    expect(screen.getByLabelText("PIN")).toHaveValue("");
    await userEvent.type(screen.getByLabelText("PIN"), "1111{Enter}");
    expect(await screen.findByText("Demasiados intentos. Espere 30 segundos e intente otra vez.")).toBeInTheDocument();
    expect(onUnlock).not.toHaveBeenCalled();
  });
});

describe("SecurityPage", () => {
  it("sets a PIN only when both are the same", async () => {
    vi.mocked(api.pinSet).mockResolvedValue(undefined);
    show(<SecurityPage />);
    expect(await screen.findByText("No hay PIN: el programa abre sin pedirlo.")).toBeInTheDocument();
    await userEvent.type(screen.getByLabelText(/^PIN nuevo/), "4821");
    await userEvent.type(screen.getByLabelText(/^Repita el PIN nuevo/), "4822");
    expect(screen.getByText("Los dos PIN no coinciden.")).toBeInTheDocument();
    expect(screen.getByText("Poner el PIN")).toBeDisabled();
    await userEvent.clear(screen.getByLabelText(/^Repita el PIN nuevo/));
    await userEvent.type(screen.getByLabelText(/^Repita el PIN nuevo/), "4821");
    await userEvent.click(screen.getByText("Poner el PIN"));
    await waitFor(() => expect(api.pinSet).toHaveBeenCalledWith("4821", undefined));
    expect(await screen.findByText("Listo, el PIN quedó guardado.")).toBeInTheDocument();
  });

  it("with a PIN already set, changing or clearing it asks for the current one", async () => {
    vi.mocked(api.pinStatus).mockResolvedValue(true);
    vi.mocked(api.pinSet).mockResolvedValue(undefined);
    vi.mocked(api.pinClear).mockRejectedValue(new Error("Ese no es el PIN actual."));
    show(<SecurityPage />);
    expect(await screen.findByText("El PIN está puesto.")).toBeInTheDocument();
    expect(screen.getByText("Cambiar el PIN")).toBeDisabled();
    expect(screen.getByText("Quitar el PIN")).toBeDisabled();
    await userEvent.type(screen.getByLabelText("PIN actual"), "0000");
    await userEvent.click(screen.getByText("Quitar el PIN"));
    expect(await screen.findByText("Ese no es el PIN actual.")).toBeInTheDocument();
    await userEvent.type(screen.getByLabelText(/^PIN nuevo/), "9999");
    await userEvent.type(screen.getByLabelText(/^Repita el PIN nuevo/), "9999");
    await userEvent.click(screen.getByText("Cambiar el PIN"));
    await waitFor(() => expect(api.pinSet).toHaveBeenCalledWith("9999", "0000"));
  });

  it("makes a backup with a password written twice and says where it was left", async () => {
    vi.mocked(api.backupCreate).mockResolvedValue({ file_name: "Respaldo Cimiento 2026-10-03.cimiento", path: "C:\\Users\\Ana\\Downloads\\Respaldo Cimiento 2026-10-03.cimiento" });
    show(<SecurityPage />);
    await userEvent.type(await screen.findByLabelText(/^Contraseña del respaldo/), "una contraseña larga");
    await userEvent.type(screen.getByLabelText(/^Repita la contraseña/), "otra distinta");
    expect(screen.getByText("Las dos contraseñas no coinciden.")).toBeInTheDocument();
    expect(screen.getByText("Crear el respaldo")).toBeDisabled();
    await userEvent.clear(screen.getByLabelText(/^Repita la contraseña/));
    await userEvent.type(screen.getByLabelText(/^Repita la contraseña/), "una contraseña larga");
    await userEvent.click(screen.getByText("Crear el respaldo"));
    await waitFor(() => expect(api.backupCreate).toHaveBeenCalledWith("una contraseña larga"));
    expect(await screen.findByText(/quedó en su carpeta de Descargas con el nombre «Respaldo Cimiento 2026-10-03.cimiento»/)).toBeInTheDocument();
  });

  it("restores a backup only after the person confirms that what is there will be replaced", async () => {
    vi.mocked(api.backupRestore).mockResolvedValue(undefined);
    show(<SecurityPage />);
    const file = new File(["datos"], "respaldo.cimiento");
    await userEvent.upload(await screen.findByLabelText("Archivo del respaldo"), file);
    await userEvent.type(screen.getByLabelText("Contraseña de ese respaldo"), "una contraseña larga");
    await userEvent.click(screen.getByText("Restaurar el respaldo"));
    expect(screen.getByText(/Todo lo que hay ahora en el programa se reemplaza/)).toBeInTheDocument();
    expect(api.backupRestore).not.toHaveBeenCalled();
    await userEvent.click(screen.getByText("Sí, restaurar"));
    await waitFor(() => expect(api.backupRestore).toHaveBeenCalledWith(btoa("datos"), "una contraseña larga"));
    expect(await screen.findByText("Listo, el respaldo quedó restaurado.")).toBeInTheDocument();
  });

  it("says plainly when the password does not open the backup", async () => {
    vi.mocked(api.backupRestore).mockRejectedValue(new Error("Esa contraseña no abre este respaldo, o el archivo no es un respaldo de Cimiento."));
    show(<SecurityPage />);
    await userEvent.upload(await screen.findByLabelText("Archivo del respaldo"), new File(["x"], "r.cimiento"));
    await userEvent.type(screen.getByLabelText("Contraseña de ese respaldo"), "mala contraseña");
    await userEvent.click(screen.getByText("Restaurar el respaldo"));
    await userEvent.click(screen.getByText("Sí, restaurar"));
    expect(await screen.findByText(/Esa contraseña no abre este respaldo/)).toBeInTheDocument();
  });

  it("scans for data of people and shows only counts by place", async () => {
    vi.mocked(api.securityScan).mockResolvedValueOnce({ tables: [{ table: "project_section", texts: 9, findings: 0 }], texts: 9, findings: 0 });
    show(<SecurityPage />);
    await userEvent.click(await screen.findByText("Revisar ahora"));
    expect(await screen.findByText("No encontramos datos de personas en 9 textos revisados.")).toBeInTheDocument();
    vi.mocked(api.securityScan).mockResolvedValueOnce({
      tables: [{ table: "project_section", texts: 9, findings: 2 }, { table: "need", texts: 3, findings: 0 }],
      texts: 12,
      findings: 2,
    });
    await userEvent.click(screen.getByText("Revisar ahora"));
    expect(await screen.findByText("Encontramos 2 posibles datos de personas. Estos son los lugares donde están:")).toBeInTheDocument();
    expect(screen.getByText("Textos del proyecto: 2")).toBeInTheDocument();
    expect(screen.queryByText(/Objetivos: /)).not.toBeInTheDocument();
  });
});
