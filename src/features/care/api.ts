// The commands of the module of the people served (ADR-029).
import { invoke } from "@tauri-apps/api/core";
import type { Decision } from "../../lib/types";
import type { BeneficiaryData, BeneficiaryOutcome, BeneficiaryView, CareChange, CareField, CareOverview, GroupOutcome, WaitlistInput, WaitlistOutcome } from "./types";

export const careOverview = () => invoke<CareOverview>("care_overview");
export const carePersonGet = (id: string) => invoke<BeneficiaryView>("care_person_get", { id });
export const carePersonSave = (id: string | null, data: BeneficiaryData) => invoke<BeneficiaryOutcome>("care_person_save", { id, data });
export const carePersonDelete = (id: string) => invoke<CareChange>("care_person_delete", { id });
/** Shows the covered CURP; the audit log keeps that it was looked at. */
export const carePersonReveal = (id: string) => invoke<string>("care_person_reveal", { id });
export const careGroupSave = (id: string | null, title: string, active: boolean, decision?: Decision) => invoke<GroupOutcome>("care_group_save", { id, title, active, decision: decision ?? null });
export const careFieldSave = (key: string | null, title: string, kind: string, options: string[]) => invoke<CareField[]>("care_field_save", { key, title, kind, options });
export const careFieldDelete = (key: string) => invoke<CareField[]>("care_field_delete", { key });
export const careWaitlistSave = (id: string | null, input: WaitlistInput) => invoke<WaitlistOutcome>("care_waitlist_save", { id, input });
export const careWaitlistAdmit = (id: string) => invoke<BeneficiaryOutcome>("care_waitlist_admit", { id });
export const careWaitlistDelete = (id: string) => invoke<CareChange>("care_waitlist_delete", { id });
