import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";

vi.mock("../../lib/tauri", () => ({
  securityScan: vi.fn(),
  backupCreate: vi.fn(),
  backupRestore: vi.fn(),
  toAppError: (e: unknown) => ({ code: "x", message: e instanceof Error ? e.message : String(e) }),
}));

import * as api from "../../lib/tauri";
import { SessionContext } from "../access/session";
import type { Permission, Role } from "../access/types";
import { SecurityPage } from "./SecurityPage";

/** The page as someone with this role sees it (ADR-028). */
function show(ui: React.ReactElement, role: Role = "admin") {
  const qc = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  const permissions: Permission[] = role === "admin" ? ["use", "delete", "settings", "administer"] : ["use"];
  const session = { user_id: "u", username: "u", display_name: "U", role, permissions, must_change_password: false, locked: false };
  render(
    <QueryClientProvider client={qc}>
      <SessionContext.Provider value={{ session, can: (p) => permissions.includes(p), lock: vi.fn(), logout: vi.fn() }}>{ui}</SessionContext.Provider>
    </QueryClientProvider>,
  );
}

beforeEach(() => {
  vi.resetAllMocks();
});

describe("SecurityPage", () => {
  it("direction and accounting make backups but only the administrator restores one", async () => {
    show(<SecurityPage />, "manager");
    expect(await screen.findByText("Crear el respaldo")).toBeInTheDocument();
    expect(screen.queryByText("Restaurar el respaldo")).not.toBeInTheDocument();
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
