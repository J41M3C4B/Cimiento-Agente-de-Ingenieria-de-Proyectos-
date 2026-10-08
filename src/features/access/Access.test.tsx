import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";

vi.mock("./api", () => ({
  accessSetupAdmin: vi.fn(),
  accessLogin: vi.fn(),
  accessRecover: vi.fn(),
  accessUnlock: vi.fn(),
  accessLogout: vi.fn(),
  accessChangePassword: vi.fn(),
}));
vi.mock("../../lib/tauri", () => ({ toAppError: (e: unknown) => ({ code: "x", message: e instanceof Error ? e.message : String(e) }) }));

import { Shell } from "../../components/Shell";
import * as api from "./api";
import { AccessScreen } from "./AccessScreen";
import type { AccessStatus, SessionView } from "./types";

const session: SessionView = { user_id: "u1", username: "jaime", display_name: "Jaime", role: "admin", permissions: ["use", "delete", "settings", "administer"], must_change_password: false, locked: false };
const status = (patch: Partial<AccessStatus>): AccessStatus => ({ needs_setup: false, setup_needs_pin: false, session: null, idle_minutes: 15, ...patch });

describe("entering the app", () => {
  beforeEach(() => vi.clearAllMocks());

  it("the first time makes the administrator and shows the recovery code once", async () => {
    vi.mocked(api.accessSetupAdmin).mockResolvedValue({ session, recovery_code: "ABCD-EFGH-JKLM-NPQR" });
    const onSession = vi.fn();
    render(<AccessScreen status={status({ needs_setup: true, setup_needs_pin: true })} onSession={onSession} />);
    await userEvent.type(screen.getByLabelText("Su nombre"), "Jaime Caballero");
    await userEvent.type(screen.getByLabelText("Usuario"), "jaime");
    await userEvent.type(screen.getByLabelText(/^Contraseña/), "una clave larga");
    await userEvent.type(screen.getByLabelText(/^Escríbala otra vez/), "una clave larga");
    await userEvent.type(screen.getByLabelText(/^PIN actual/), "4821");
    await userEvent.click(screen.getByRole("button", { name: "Crear cuenta" }));
    expect(api.accessSetupAdmin).toHaveBeenCalledWith("Jaime Caballero", "jaime", "una clave larga", "4821");
    expect(await screen.findByText("ABCD-EFGH-JKLM-NPQR")).toBeInTheDocument();
    expect(onSession).not.toHaveBeenCalled();
    await userEvent.click(screen.getByRole("button", { name: "Ya lo guardé" }));
    expect(onSession).toHaveBeenCalledWith(session);
  });

  it("a wrong password says so without telling which part, and too many make it wait", async () => {
    vi.mocked(api.accessLogin).mockResolvedValueOnce({ status: "wrong" }).mockResolvedValueOnce({ status: "waiting", wait_secs: 30 });
    render(<AccessScreen status={status({})} onSession={vi.fn()} />);
    await userEvent.type(screen.getByLabelText("Usuario"), "rosa");
    await userEvent.type(screen.getByLabelText("Contraseña"), "mala{Enter}");
    expect(await screen.findByText("El usuario o la contraseña no son correctos.")).toBeInTheDocument();
    await userEvent.type(screen.getByLabelText("Contraseña"), "otra{Enter}");
    expect(await screen.findByText(/Espere 30 segundos/)).toBeInTheDocument();
  });

  it("a temporary password is changed before entering", async () => {
    vi.mocked(api.accessChangePassword).mockResolvedValue({ ...session, must_change_password: false });
    const onSession = vi.fn();
    render(<AccessScreen status={status({ session: { ...session, role: "manager", must_change_password: true } })} onSession={onSession} />);
    expect(screen.getByText("Cambie su contraseña")).toBeInTheDocument();
    await userEvent.type(screen.getByLabelText(/^Contraseña temporal/), "temporal123");
    await userEvent.type(screen.getByLabelText(/^Contraseña nueva/), "mi clave propia");
    await userEvent.type(screen.getByLabelText(/^Escríbala otra vez/), "mi clave propia");
    await userEvent.click(screen.getByRole("button", { name: "Guardar y seguir" }));
    await waitFor(() => expect(onSession).toHaveBeenCalled());
    expect(api.accessChangePassword).toHaveBeenCalledWith("temporal123", "mi clave propia");
  });

  it("the locked screen asks the password of who was inside", async () => {
    vi.mocked(api.accessUnlock).mockResolvedValue({ status: "ok", session });
    const onSession = vi.fn();
    render(<AccessScreen status={status({ session: { ...session, locked: true } })} onSession={onSession} />);
    expect(screen.getByText(/Jaime, escriba su contraseña/)).toBeInTheDocument();
    await userEvent.type(screen.getByLabelText("Contraseña"), "una clave larga{Enter}");
    await waitFor(() => expect(onSession).toHaveBeenCalledWith(session));
  });
});

describe("the menu shows what each person may use", () => {
  const shell = (permissions: SessionView["permissions"]) =>
    render(
      <Shell page="home" onNavigate={vi.fn()} institution="Asilo" focus={undefined} onOpenFocus={vi.fn()} access={{ session: { ...session, permissions }, can: (p) => permissions.includes(p), lock: vi.fn(), logout: vi.fn() }}>
        <p>contenido</p>
      </Shell>,
    );

  it("the administrator sees the administration and the AI settings", () => {
    shell(["use", "delete", "settings", "administer"]);
    expect(screen.getByRole("button", { name: "Administración" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Ayuda automática" })).toBeInTheDocument();
  });

  it("direction and accounting do not", () => {
    shell(["use"]);
    expect(screen.queryByRole("button", { name: "Administración" })).not.toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "Ayuda automática" })).not.toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Seguridad" })).toBeInTheDocument();
  });
});
