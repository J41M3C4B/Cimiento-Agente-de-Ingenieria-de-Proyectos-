// The commands of the staff module (ADR-027).
import { invoke } from "@tauri-apps/api/core";
import type { Decision } from "../../lib/types";
import type { CustomField, ModalityInfo, PersonData, PersonOutcome, PersonView, PositionInput, PositionOutcome, SecretField, StaffChange, StaffOverview } from "./types";

export const hrOverview = () => invoke<StaffOverview>("hr_overview");
export const hrPersonGet = (id: string) => invoke<PersonView>("hr_person_get", { id });
export const hrPersonSave = (id: string | null, data: PersonData) => invoke<PersonOutcome>("hr_person_save", { id, data });
export const hrPersonDelete = (id: string) => invoke<StaffChange>("hr_person_delete", { id });
/** Shows a covered identifier; the audit log keeps that it was looked at. */
export const hrPersonReveal = (id: string, field: SecretField) => invoke<string>("hr_person_reveal", { id, field });
export const hrPositionSave = (id: string | null, input: PositionInput, decision?: Decision) =>
  invoke<PositionOutcome>("hr_position_save", { id, input, decision: decision ?? null });
export const hrPositionSetActive = (id: string, active: boolean) => invoke<StaffChange>("hr_position_set_active", { id, active });
export const hrModalityCreate = (title: string, behavesAs: string) => invoke<ModalityInfo[]>("hr_modality_create", { title, behavesAs });
export const hrFieldSave = (key: string | null, title: string, kind: string, options: string[]) => invoke<CustomField[]>("hr_field_save", { key, title, kind, options });
export const hrFieldDelete = (key: string) => invoke<CustomField[]>("hr_field_delete", { key });
