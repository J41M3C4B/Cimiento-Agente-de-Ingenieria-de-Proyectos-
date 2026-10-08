import { useState } from "react";
import { Icon } from "../../components/icons";
import { Alert, Button, Choice, FactRow, Facts, Modal, TextArea, TextInput } from "../../components/ui";
import type { Tone } from "../../components/ui";
import { es } from "../../i18n/es-MX";
import { Many, YesNo } from "./GroupDialog";
import type { FacilityIssue, SiteData } from "./types";

const f = es.facilities;
const t = f.building;
type Section = "building" | "services" | "safety";

const num = (v: string) => (v.trim() === "" ? null : Number(v.replace(/[,\s]/g, "")));
const yes = (v: boolean | null) => (v === null ? null : v ? es.common.yes : es.common.no);
const word = (labels: Record<string, string>, v: string | null) => (v ? (labels[v] ?? v) : null);
const words = (labels: Record<string, string>, v: string[]) => (v.length ? v.map((x) => labels[x] ?? x).join(", ") : null);
const n = (v: number | null) => (v === null ? null : v.toLocaleString("es-MX"));

/** One question with a few short answers, as pills; a click on the chosen one clears it. */
function OneOf({ label, labels, value, onChange, name, tone }: { label: string; labels: Record<string, string>; value: string | null; onChange: (v: string | null) => void; name: string; tone?: Record<string, Tone> }) {
  return (
    <fieldset>
      <legend className="field-label">{label}</legend>
      <div className="flex flex-wrap gap-2">
        {Object.entries(labels).map(([code, text]) => (
          <Choice key={code} name={name} tone={tone?.[code]} checked={value === code} onChange={() => onChange(code)} onClick={() => value === code && onChange(null)}>
            {text}
          </Choice>
        ))}
      </div>
    </fieldset>
  );
}

const FREQ_TONE: Record<string, Tone> = { never: "green", sometimes: "amber", often: "red" };

/** The building, its services and its safety: three blocks to read, each one edited in its own window. */
export function BuildingPanel({ site, issues, onSave }: { site: SiteData; issues: FacilityIssue[]; onSave: (d: SiteData) => Promise<FacilityIssue[] | null> }) {
  const [open, setOpen] = useState<Section | null>(null);
  const edit = (s: Section) => (
    <Button size="sm" variant="plain" onClick={() => setOpen(s)}>
      <Icon name="pencil" size={16} />
      {f.edit}
    </Button>
  );
  const heads = issues.filter((i) => !i.blocking);
  return (
    <div>
      {heads.length > 0 && (
        <Alert tone="info">{heads.map((i) => f.issues[i.code] ?? i.code).join(" ")}</Alert>
      )}
      <FactRow title={t.sections.building} note={t.help.building} action={edit("building")}>
        <Facts
          items={[
            [t.name, site.name],
            [t.built_m2, site.built_m2 !== null ? t.m2(site.built_m2) : null],
            [t.land_m2, site.land_m2 !== null ? t.m2(site.land_m2) : null],
            [t.floors, n(site.floors)],
            [t.floor_access, site.floors !== null && site.floors > 1 ? words(t.floorAccess, site.floor_access) : null],
            [t.built_year, site.built_year?.toString() ?? null],
            [t.tenure, word(t.tenures, site.tenure) && `${word(t.tenures, site.tenure)}${site.tenure_until ? ` · ${site.tenure_until}` : ""}`],
            [t.tenure_documented, yes(site.tenure_documented)],
          ]}
        />
      </FactRow>
      <FactRow title={t.sections.services} note={t.help.services} action={edit("services")}>
        <Facts
          items={[
            [t.water_sources, words(t.waterSources, site.water_sources)],
            [t.water_shortage, word(t.frequency, site.water_shortage)],
            [t.water_storage_liters, site.water_storage_liters !== null ? t.liters(site.water_storage_liters) : null],
            [t.power_outages, word(t.frequency, site.power_outages)],
            [t.gas, word(t.gasKinds, site.gas)],
            [t.drainage, word(t.drainageKinds, site.drainage)],
            [t.internet, yes(site.internet)],
          ]}
        />
      </FactRow>
      <FactRow title={t.sections.safety} note={t.help.safety} action={edit("safety")}>
        <Facts
          items={[
            [t.extinguishers, n(site.extinguishers)],
            [t.extinguishers_current, yes(site.extinguishers_current)],
            [t.smoke_detectors, n(site.smoke_detectors)],
            [t.marked_exits, yes(site.marked_exits)],
            [t.emergency_lights, yes(site.emergency_lights)],
            [t.first_aid_kit, yes(site.first_aid_kit)],
            [t.internal_program, word(t.programs, site.internal_program)],
            [t.civil_protection_opinion, yes(site.civil_protection_opinion) && `${yes(site.civil_protection_opinion)}${site.opinion_year ? ` · ${site.opinion_year}` : ""}`],
            [t.drills_per_year, n(site.drills_per_year)],
          ]}
        />
      </FactRow>
      {open && <SiteDialog section={open} start={site} onSave={onSave} onClose={() => setOpen(null)} />}
    </div>
  );
}

function SiteDialog({ section, start, onSave, onClose }: { section: Section; start: SiteData; onSave: (d: SiteData) => Promise<FacilityIssue[] | null>; onClose: () => void }) {
  const [d, setD] = useState<SiteData>(start);
  const [issues, setIssues] = useState<FacilityIssue[]>([]);
  const [busy, setBusy] = useState(false);
  const set = (patch: Partial<SiteData>) => setD((x) => ({ ...x, ...patch }));
  const err = (field: string) => {
    const i = issues.find((x) => x.field === field);
    return i ? (f.issues[i.code] ?? es.errors.generic) : undefined;
  };

  async function save() {
    setBusy(true);
    const out = await onSave(d);
    setBusy(false);
    if (out) setIssues(out);
    else onClose();
  }

  return (
    <Modal
      title={t.sections[section]}
      size="lg"
      onClose={onClose}
      footer={
        <>
          <Button onClick={onClose}>{es.common.cancel}</Button>
          <Button variant="primary" onClick={save} disabled={busy}>
            {busy ? es.common.saving : es.common.save}
          </Button>
        </>
      }
    >
      <p className="text-ui text-ink-2">{t.help[section]}</p>
      {section === "building" && (
        <>
          <div className="grid grid-cols-1 gap-4 sm:grid-cols-2">
            <TextInput label={t.name} value={d.name} onChange={(e) => set({ name: e.target.value })} error={err("name")} />
            <TextInput label={t.built_year} inputMode="numeric" value={d.built_year?.toString() ?? ""} onChange={(e) => set({ built_year: num(e.target.value) })} error={err("built_year")} />
            <TextInput label={t.land_m2} inputMode="numeric" suffix="m²" value={d.land_m2?.toString() ?? ""} onChange={(e) => set({ land_m2: num(e.target.value) })} error={err("land_m2")} />
            <TextInput label={t.built_m2} inputMode="numeric" suffix="m²" value={d.built_m2?.toString() ?? ""} onChange={(e) => set({ built_m2: num(e.target.value) })} error={err("built_m2")} />
            <TextInput label={t.floors} inputMode="numeric" value={d.floors?.toString() ?? ""} onChange={(e) => set({ floors: num(e.target.value) })} error={err("floors")} />
          </div>
          {d.floors !== null && d.floors > 1 && <Many label={t.floor_access} labels={t.floorAccess} value={d.floor_access} onChange={(floor_access) => set({ floor_access })} />}
          <OneOf name="tenure" label={t.tenure} labels={t.tenures} value={d.tenure} onChange={(tenure) => set({ tenure })} />
          {(d.tenure === "loan" || d.tenure === "rent") && (
            <TextInput label={t.tenure_until} inputMode="numeric" value={d.tenure_until?.toString() ?? ""} onChange={(e) => set({ tenure_until: num(e.target.value) })} error={err("tenure_until")} />
          )}
          <YesNo name="tenure_documented" label={t.tenure_documented} value={d.tenure_documented} onChange={(tenure_documented) => set({ tenure_documented })} />
          <TextArea label={t.notes} hint={f.spaces.notesHint} rows={3} value={d.notes ?? ""} onChange={(e) => set({ notes: e.target.value || null })} />
        </>
      )}
      {section === "services" && (
        <>
          <Many label={t.water_sources} labels={t.waterSources} value={d.water_sources} onChange={(water_sources) => set({ water_sources })} />
          <OneOf name="water_shortage" label={t.water_shortage} labels={t.frequency} tone={FREQ_TONE} value={d.water_shortage} onChange={(water_shortage) => set({ water_shortage })} />
          <TextInput label={t.water_storage_liters} inputMode="numeric" suffix="L" value={d.water_storage_liters?.toString() ?? ""} onChange={(e) => set({ water_storage_liters: num(e.target.value) })} error={err("water_storage_liters")} />
          <OneOf name="power_outages" label={t.power_outages} labels={t.frequency} tone={FREQ_TONE} value={d.power_outages} onChange={(power_outages) => set({ power_outages })} />
          <OneOf name="gas" label={t.gas} labels={t.gasKinds} value={d.gas} onChange={(gas) => set({ gas })} />
          <OneOf name="drainage" label={t.drainage} labels={t.drainageKinds} value={d.drainage} onChange={(drainage) => set({ drainage })} />
          <YesNo name="internet" label={t.internet} value={d.internet} onChange={(internet) => set({ internet })} />
        </>
      )}
      {section === "safety" && (
        <>
          <div className="grid grid-cols-1 gap-4 sm:grid-cols-2">
            <TextInput label={t.extinguishers} inputMode="numeric" value={d.extinguishers?.toString() ?? ""} onChange={(e) => set({ extinguishers: num(e.target.value) })} error={err("extinguishers")} />
            <TextInput label={t.smoke_detectors} inputMode="numeric" value={d.smoke_detectors?.toString() ?? ""} onChange={(e) => set({ smoke_detectors: num(e.target.value) })} error={err("smoke_detectors")} />
          </div>
          {d.extinguishers !== 0 && <YesNo name="extinguishers_current" label={t.extinguishers_current} value={d.extinguishers_current} onChange={(extinguishers_current) => set({ extinguishers_current })} />}
          <YesNo name="marked_exits" label={t.marked_exits} value={d.marked_exits} onChange={(marked_exits) => set({ marked_exits })} />
          <YesNo name="emergency_lights" label={t.emergency_lights} value={d.emergency_lights} onChange={(emergency_lights) => set({ emergency_lights })} />
          <YesNo name="first_aid_kit" label={t.first_aid_kit} value={d.first_aid_kit} onChange={(first_aid_kit) => set({ first_aid_kit })} />
          <OneOf name="internal_program" label={t.internal_program} labels={t.programs} value={d.internal_program} onChange={(internal_program) => set({ internal_program })} />
          <YesNo name="civil_protection_opinion" label={t.civil_protection_opinion} value={d.civil_protection_opinion} onChange={(civil_protection_opinion) => set({ civil_protection_opinion })} />
          {d.civil_protection_opinion && (
            <TextInput label={t.opinion_year} inputMode="numeric" value={d.opinion_year?.toString() ?? ""} onChange={(e) => set({ opinion_year: num(e.target.value) })} error={err("opinion_year")} />
          )}
          <TextInput label={t.drills_per_year} inputMode="numeric" value={d.drills_per_year?.toString() ?? ""} onChange={(e) => set({ drills_per_year: num(e.target.value) })} error={err("drills_per_year")} />
        </>
      )}
      {issues.length > 0 && <Alert tone="error">{issues.map((i) => f.issues[i.code] ?? es.errors.generic).join(" ")}</Alert>}
    </Modal>
  );
}
