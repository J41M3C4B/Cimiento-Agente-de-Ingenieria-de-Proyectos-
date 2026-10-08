// The commands of the facilities module (ADR-030).
import { invoke } from "@tauri-apps/api/core";
import type { Decision } from "../../lib/types";
import type { EquipmentData, FacilitiesOutcome, FacilitiesOverview, SiteData, SpaceData } from "./types";

export const facilitiesOverview = () => invoke<FacilitiesOverview>("facilities_overview");
export const facilitiesSiteSave = (data: SiteData, decision?: Decision) => invoke<FacilitiesOutcome>("facilities_site_save", { data, decision: decision ?? null });
export const facilitiesSpaceSave = (id: string | null, data: SpaceData, decision?: Decision) =>
  invoke<FacilitiesOutcome>("facilities_space_save", { id, data, decision: decision ?? null });
export const facilitiesSpaceDelete = (id: string) => invoke<FacilitiesOverview>("facilities_space_delete", { id });
export const facilitiesEquipmentSave = (id: string | null, data: EquipmentData, decision?: Decision) =>
  invoke<FacilitiesOutcome>("facilities_equipment_save", { id, data, decision: decision ?? null });
export const facilitiesEquipmentDelete = (id: string) => invoke<FacilitiesOverview>("facilities_equipment_delete", { id });
