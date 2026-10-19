// The facilities module (ADR-030): its own types, apart from the rest of the app.
import type { QuarantineReport } from "../../lib/types";

/** How many are in each state. They may add up to less than the count: the rest are not checked yet. */
export interface States {
  good: number;
  fair: number;
  poor: number;
  unusable: number;
}

export interface SiteData {
  name: string;
  land_m2: number | null;
  built_m2: number | null;
  floors: number | null;
  /** ramp, elevator, stair_lift; `none` = only stairs. Empty = not said. */
  floor_access: string[];
  built_year: number | null;
  tenure: string | null;
  tenure_until: number | null;
  tenure_documented: boolean | null;
  water_sources: string[];
  water_shortage: string | null;
  water_storage_liters: number | null;
  power_outages: string | null;
  gas: string | null;
  drainage: string | null;
  internet: boolean | null;
  extinguishers: number | null;
  extinguishers_current: boolean | null;
  smoke_detectors: number | null;
  marked_exits: boolean | null;
  emergency_lights: boolean | null;
  first_aid_kit: boolean | null;
  internal_program: string | null;
  civil_protection_opinion: boolean | null;
  opinion_year: number | null;
  drills_per_year: number | null;
  notes: string | null;
}

/** A group of spaces of one kind on one floor: «Baños · planta alta · 4 → 3 bien, 1 mal». */
export interface SpaceData extends States {
  kind: string;
  label: string | null;
  /** -1 basement, 0 ground floor, 1 first floor… */
  floor: number;
  count: number;
  problems: string[];
  accessible: boolean | null;
  beds: number | null;
  hospital_beds: number | null;
  grab_bars: boolean | null;
  accessible_shower: boolean | null;
  notes: string | null;
}

export interface EquipmentData extends States {
  kind: string;
  label: string | null;
  count: number;
  notes: string | null;
}

export interface FacilityIssue {
  code: string;
  field: string;
  blocking: boolean;
}

export interface SpaceRow extends SpaceData {
  id: string;
  site_id: string;
  issues: FacilityIssue[];
}

export interface EquipmentRow extends EquipmentData {
  id: string;
  site_id: string;
  issues: FacilityIssue[];
}

export interface GroupRef {
  kind: string;
  label: string | null;
  floor: number;
  count: number;
  bad: number;
}

export interface KindTotal extends States {
  kind: string;
  count: number;
  unchecked: number;
}

export interface FacilityIndicators {
  sites: number;
  land_m2: number | null;
  built_m2: number | null;
  floors: number | null;
  only_stairs: boolean;
  spaces_upstairs: number;
  spaces: number;
  spaces_states: States;
  spaces_unchecked: number;
  by_kind: KindTotal[];
  bathrooms: number;
  bathrooms_without_bars: number;
  beds: number | null;
  hospital_beds: number | null;
  not_accessible: GroupRef[];
  broken: GroupRef[];
  problems: { code: string; count: number }[];
  structural: number;
  equipment: number;
  equipment_states: States;
  equipment_by_kind: KindTotal[];
  broken_equipment: GroupRef[];
  generator_works: boolean | null;
  missing: string[];
}

export interface FacilityInsight {
  code: string;
  values: Record<string, number>;
  /** Already in words, made in Rust («1 de 4 baños (primer piso)»), or codes for the screen to name. */
  items: string[];
  for_ai: boolean;
}

export interface FacilityBoard {
  indicators: FacilityIndicators;
  served: number;
  capacity: number | null;
  built_m2_per_person: number | null;
  people_per_bathroom: number | null;
  insights: FacilityInsight[];
}

export interface FacilitiesOverview {
  site: { id: string | null; data: SiteData; issues: FacilityIssue[] };
  spaces: SpaceRow[];
  equipment: EquipmentRow[];
  indicators: FacilityIndicators;
  space_kinds: string[];
  equipment_kinds: string[];
  board: FacilityBoard;
}

export type FacilitiesOutcome =
  | { status: "saved"; overview: FacilitiesOverview }
  | { status: "invalid"; issues: FacilityIssue[] }
  | { status: "quarantine"; report: QuarantineReport };
