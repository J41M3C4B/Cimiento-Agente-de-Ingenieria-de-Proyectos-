// Access profiles (ADR-028).
export type Role = "admin" | "manager";
export type Permission = "use" | "delete" | "settings" | "administer";

export interface SessionView {
  user_id: string;
  username: string;
  display_name: string;
  role: Role;
  permissions: Permission[];
  must_change_password: boolean;
  locked: boolean;
}
export interface AccessStatus {
  needs_setup: boolean;
  setup_needs_pin: boolean;
  session: SessionView | null;
  idle_minutes: number;
}
export interface SetupOutcome {
  session: SessionView;
  /** Shown once, to write it down. */
  recovery_code: string;
}
export type LoginOutcome =
  | { status: "ok"; session: SessionView }
  | { status: "wrong" }
  | { status: "waiting"; wait_secs: number }
  | { status: "disabled" };

/** Someone who has access, as Inicio shows them. */
export interface TeamMember {
  display_name: string;
  role: Role;
}
export interface UserRow {
  id: string;
  username: string;
  display_name: string;
  person_id: string | null;
  role: Role;
  active: boolean;
  must_change_password: boolean;
  last_login_at: string | null;
  waiting: boolean;
}
export interface PersonAccess {
  person_id: string;
  full_name: string;
  position_id: string | null;
  status: string;
  user_id: string | null;
  suggested_username: string;
}
export type DeletionKind = "document" | "hr_person" | "beneficiary" | "project" | "roster_field" | "hr_field";
export interface RequestRow {
  id: string;
  kind: DeletionKind;
  target_id: string;
  target_label: string;
  requested_by: string;
  requested_by_name: string;
  requested_at: string;
  status: "pending" | "approved" | "rejected";
  resolved_by_name: string | null;
  resolved_at: string | null;
}
export interface AdminOverview {
  users: UserRow[];
  people: PersonAccess[];
  pending: RequestRow[];
  resolved: RequestRow[];
}
export interface AuditRow {
  at: string;
  event: string;
  entity: string | null;
  entity_id: string | null;
  actor: string | null;
  details: unknown;
}
