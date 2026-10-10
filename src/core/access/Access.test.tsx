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
  const shell = (permissions: SessionView["permissions"], page: Parameters<typeof Shell>[0]["page"] = "home", onNavigate = vi.fn()) =>
    render(
      <Shell page={page} onNavigate={onNavigate} institution="Asilo" access={{ session: { ...session, permissions }, can: (p) => permissions.includes(p), lock: vi.fn(), logout: vi.fn() }}>
        <p>contenido</p>
      </Shell>,
    );
  const openSettings = () => userEvent.click(screen.getByRole("button", { name: "Configuración" }));

  it("the bar names every section of the work, with the page in view marked", () => {
    shell(["use"], "finance");
    for (const name of ["Inicio", "Mi institución", "Documentos", "Mis proyectos", "Personal", "Beneficiarios", "Instalaciones", "Finanzas"]) {
      expect(screen.getByRole("button", { name })).toBeInTheDocument();
    }
    expect(screen.getByRole("button", { name: "Finanzas" })).toHaveAttribute("aria-current", "page");
    expect(screen.getByRole("button", { name: "Inicio" })).not.toHaveAttribute("aria-current");
  });

  it("the window takes the accent of the page: the institution's, or the module's own", () => {
    const { container, unmount } = shell(["use"], "profile");
    expect(container.querySelector(".shell")).toHaveClass("accent-institution");
    unmount();
    const again = shell(["use"], "staff");
    expect(again.container.querySelector(".shell")).toHaveClass("accent-staff");
  });

  it("the administrator sees the administration and the AI settings in the gear", async () => {
    shell(["use", "delete", "settings", "administer"]);
    await openSettings();
    expect(screen.getByRole("menuitem", { name: "Administración" })).toBeInTheDocument();
    expect(screen.getByRole("menuitem", { name: "Ayuda automática" })).toBeInTheDocument();
  });

  it("direction and accounting do not", async () => {
    shell(["use"]);
    await openSettings();
    expect(screen.queryByRole("menuitem", { name: "Administración" })).not.toBeInTheDocument();
    expect(screen.queryByRole("menuitem", { name: "Ayuda automática" })).not.toBeInTheDocument();
    expect(screen.getByRole("menuitem", { name: "Seguridad" })).toBeInTheDocument();
    expect(screen.getByRole("menuitem", { name: "Ayuda" })).toBeInTheDocument();
  });

  it("an entry of the gear takes the person to its page and closes the menu", async () => {
    const onNavigate = vi.fn();
    shell(["use"], "home", onNavigate);
    await openSettings();
    await userEvent.click(screen.getByRole("menuitem", { name: "Seguridad" }));
    expect(onNavigate).toHaveBeenCalledWith("security");
    expect(screen.queryByRole("menu")).not.toBeInTheDocument();
  });

  it("each account keeps its own avatar color, the same on every page", async () => {
    window.localStorage.clear();
    const person = (page: Parameters<typeof Shell>[0]["page"]) => (
      <Shell page={page} onNavigate={vi.fn()} institution="Asilo" access={{ session, can: () => true, lock: vi.fn(), logout: vi.fn() }}>
        <p>contenido</p>
      </Shell>
    );
    const { container, rerender } = render(person("home"));
    const avatar = () => container.querySelector(".topbar-person .avatar");
    expect(avatar()).toHaveClass("tone-violet");
    await userEvent.click(screen.getByRole("button", { name: /Mi cuenta/ }));
    await userEvent.click(screen.getByRole("button", { name: "Turquesa" }));
    expect(avatar()).toHaveClass("tone-teal");
    rerender(person("staff"));
    expect(avatar()).toHaveClass("tone-teal");
    expect(window.localStorage.getItem(`cimiento.avatar.${session.username}`)).toBe("teal");
    window.localStorage.clear();
  });

  it("the person menu, at the right of the bar, locks and closes the session", async () => {
    const lock = vi.fn();
    const logout = vi.fn();
    render(
      <Shell page="home" onNavigate={vi.fn()} institution="Asilo" access={{ session, can: () => true, lock, logout }}>
        <p>contenido</p>
      </Shell>,
    );
    await userEvent.click(screen.getByRole("button", { name: /Mi cuenta/ }));
    await userEvent.click(screen.getByRole("menuitem", { name: "Bloquear" }));
    expect(lock).toHaveBeenCalled();
    await userEvent.click(screen.getByRole("button", { name: /Mi cuenta/ }));
    await userEvent.click(screen.getByRole("menuitem", { name: "Cerrar sesión" }));
    expect(logout).toHaveBeenCalled();
  });
});
