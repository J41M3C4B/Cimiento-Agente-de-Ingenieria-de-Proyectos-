// The commands of the access profiles (ADR-028).
import { invoke } from "@tauri-apps/api/core";
import type { AccessStatus, AdminOverview, AuditRow, LoginOutcome, Role, SessionView, SetupOutcome, TeamMember } from "./types";

export const accessStatus = () => invoke<AccessStatus>("access_status");
export const accessSetupAdmin = (displayName: string, username: string, password: string, pin: string | null) =>
  invoke<SetupOutcome>("access_setup_admin", { displayName, username, password, pin });
export const accessLogin = (username: string, password: string) => invoke<LoginOutcome>("access_login", { username, password });
export const accessRecover = (username: string, code: string, newPassword: string) => invoke<SetupOutcome>("access_recover", { username, code, newPassword });
export const accessUnlock = (password: string) => invoke<LoginOutcome>("access_unlock", { password });
export const accessLock = () => invoke<void>("access_lock");
export const accessLogout = () => invoke<void>("access_logout");
export const accessChangePassword = (current: string, newPassword: string) => invoke<SessionView>("access_change_password", { current, newPassword });

export const accessTeam = () => invoke<TeamMember[]>("access_team");
export const adminOverview = () => invoke<AdminOverview>("admin_overview");
export const adminUserCreate = (a: { personId: string | null; displayName: string | null; username: string; role: Role; temporaryPassword: string }) =>
  invoke<AdminOverview>("admin_user_create", a);
export const adminUserUpdate = (id: string, role: Role, active: boolean) => invoke<AdminOverview>("admin_user_update", { id, role, active });
export const adminUserResetPassword = (id: string, temporaryPassword: string) => invoke<AdminOverview>("admin_user_reset_password", { id, temporaryPassword });
export const adminRequestResolve = (id: string, approve: boolean) => invoke<AdminOverview>("admin_request_resolve", { id, approve });
export const adminAudit = (event: string | null, limit = 200) => invoke<AuditRow[]>("admin_audit", { event, limit });
export const adminRecoveryCodeNew = () => invoke<string>("admin_recovery_code_new");
