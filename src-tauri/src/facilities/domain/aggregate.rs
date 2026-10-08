//! The indicators of the facilities (ADR-030), made by code over every site: how many spaces and how they are, by
//! kind; bathrooms and their grab bars; beds; what is not accessible; what fails most; the equipment. There is no
//! personal datum in a facility, so the whole summary may reach the AI.

use super::group::{EquipmentData, SpaceData, States};
use super::site::SiteData;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Count {
    pub code: String,
    pub count: i64,
}

/// One kind added up over its groups.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct KindTotal {
    pub kind: String,
    pub count: i64,
    #[serde(flatten)]
    pub states: States,
    /// Not checked yet: `count` minus the states.
    pub unchecked: i64,
}

/// A group named in a finding: its kind, its own name and its floor.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GroupRef {
    pub kind: String,
    pub label: Option<String>,
    pub floor: i64,
    pub count: i64,
    pub bad: i64,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct Indicators {
    pub sites: i64,
    pub land_m2: Option<i64>,
    pub built_m2: Option<i64>,
    pub floors: Option<i64>,
    /// A building of several floors with only stairs between them.
    pub only_stairs: bool,
    /// Spaces on an upper floor of such a building.
    pub spaces_upstairs: i64,
    pub spaces: i64,
    pub spaces_states: States,
    pub spaces_unchecked: i64,
    pub by_kind: Vec<KindTotal>,
    pub bathrooms: i64,
    /// Bathrooms in groups that said they have no grab bars.
    pub bathrooms_without_bars: i64,
    pub beds: Option<i64>,
    pub hospital_beds: Option<i64>,
    /// Groups that a person in a wheelchair cannot use.
    pub not_accessible: Vec<GroupRef>,
    /// Groups with something in poor state or unusable.
    pub broken: Vec<GroupRef>,
    /// How many groups of spaces have each problem, most frequent first.
    pub problems: Vec<Count>,
    /// Groups with cracks or electrical failures.
    pub structural: i64,
    pub equipment: i64,
    pub equipment_states: States,
    pub equipment_by_kind: Vec<KindTotal>,
    pub broken_equipment: Vec<GroupRef>,
    /// `Some(true)` with an emergency power plant that works; `Some(false)` when all of them fail.
    pub generator_works: Option<bool>,
    /// The key data still missing (codes).
    pub missing: Vec<&'static str>,
}

/// A site with its groups, as the AI and the guide read it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SiteSummary {
    pub site: SiteData,
    pub spaces: Vec<SpaceData>,
    pub equipment: Vec<EquipmentData>,
}

fn add_states(a: &mut States, b: &States) {
    a.good += b.good;
    a.fair += b.fair;
    a.poor += b.poor;
    a.unusable += b.unusable;
}

fn totals<'a>(groups: impl Iterator<Item = (&'a str, i64, &'a States)>) -> Vec<KindTotal> {
    let mut out: Vec<KindTotal> = Vec::new();
    for (kind, count, states) in groups {
        let i = match out.iter().position(|k| k.kind == kind) {
            Some(i) => i,
            None => {
                out.push(KindTotal { kind: kind.to_string(), count: 0, states: States::default(), unchecked: 0 });
                out.len() - 1
            }
        };
        out[i].count += count;
        add_states(&mut out[i].states, states);
        out[i].unchecked += (count - states.checked()).max(0);
    }
    out
}

fn sum_known(values: impl Iterator<Item = Option<i64>>) -> Option<i64> {
    values.flatten().fold(None, |acc, x| Some(acc.unwrap_or(0) + x))
}

pub fn indicators(sites: &[SiteSummary]) -> Indicators {
    let spaces: Vec<&SpaceData> = sites.iter().flat_map(|s| &s.spaces).collect();
    let equipment: Vec<&EquipmentData> = sites.iter().flat_map(|s| &s.equipment).collect();
    let mut i = Indicators {
        sites: sites.len() as i64,
        land_m2: sum_known(sites.iter().map(|s| s.site.land_m2)),
        built_m2: sum_known(sites.iter().map(|s| s.site.built_m2)),
        floors: sites.iter().filter_map(|s| s.site.floors).max(),
        only_stairs: sites.iter().any(|s| s.site.only_stairs()),
        spaces_upstairs: sites.iter().filter(|s| s.site.only_stairs()).flat_map(|s| &s.spaces).filter(|g| g.floor >= 1).map(|g| g.count).sum(),
        spaces: spaces.iter().map(|g| g.count).sum(),
        by_kind: totals(spaces.iter().map(|g| (g.kind.as_str(), g.count, &g.states))),
        bathrooms: spaces.iter().filter(|g| g.kind == "bathroom").map(|g| g.count).sum(),
        bathrooms_without_bars: spaces.iter().filter(|g| g.kind == "bathroom" && g.grab_bars == Some(false)).map(|g| g.count).sum(),
        beds: sum_known(spaces.iter().filter(|g| g.holds_beds()).map(|g| g.beds)),
        hospital_beds: sum_known(spaces.iter().filter(|g| g.holds_beds()).map(|g| g.hospital_beds)),
        structural: spaces.iter().filter(|g| g.problems.iter().any(|p| super::catalog::STRUCTURAL.contains(&p.as_str()))).count() as i64,
        equipment: equipment.iter().map(|g| g.count).sum(),
        equipment_by_kind: totals(equipment.iter().map(|g| (g.kind.as_str(), g.count, &g.states))),
        ..Default::default()
    };
    for g in &spaces {
        add_states(&mut i.spaces_states, &g.states);
        i.spaces_unchecked += (g.count - g.states.checked()).max(0);
        let r = GroupRef { kind: g.kind.clone(), label: g.label.clone(), floor: g.floor, count: g.count, bad: g.states.bad() };
        if g.accessible == Some(false) {
            i.not_accessible.push(r.clone());
        }
        if g.states.bad() > 0 {
            i.broken.push(r);
        }
        for p in &g.problems {
            match i.problems.iter_mut().find(|c| &c.code == p) {
                Some(c) => c.count += 1,
                None => i.problems.push(Count { code: p.clone(), count: 1 }),
            }
        }
    }
    // most frequent first; a tie keeps the order of the catalog
    let order = |c: &str| super::catalog::PROBLEMS.iter().position(|p| *p == c).unwrap_or(usize::MAX);
    i.problems.sort_by(|a, b| b.count.cmp(&a.count).then(order(&a.code).cmp(&order(&b.code))));
    for g in &equipment {
        add_states(&mut i.equipment_states, &g.states);
        if g.states.bad() > 0 {
            i.broken_equipment.push(GroupRef { kind: g.kind.clone(), label: g.label.clone(), floor: 0, count: g.count, bad: g.states.bad() });
        }
    }
    let generators: Vec<&&EquipmentData> = equipment.iter().filter(|g| g.kind == "generator").collect();
    if !generators.is_empty() {
        i.generator_works = Some(generators.iter().any(|g| g.states.good + g.states.fair > 0 || g.states.checked() < g.count));
    }

    match sites.first() {
        Some(s) => i.missing = s.site.missing(),
        None => i.missing = SiteData::default().missing(),
    }
    if spaces.is_empty() {
        i.missing.push("spaces");
    } else if i.spaces_unchecked > 0 {
        i.missing.push("unchecked");
    }
    i
}

#[cfg(test)]
mod tests {
    use super::*;

    fn space(kind: &str, floor: i64, count: i64, states: States) -> SpaceData {
        SpaceData { kind: kind.into(), floor, count, states, ..Default::default() }
    }

    fn st(good: i64, fair: i64, poor: i64, unusable: i64) -> States {
        States { good, fair, poor, unusable }
    }

    fn house() -> SiteSummary {
        let mut upstairs = space("bathroom", 1, 4, st(3, 0, 1, 0));
        upstairs.problems = vec!["leaks".into(), "grab_bars".into()];
        upstairs.grab_bars = Some(false);
        upstairs.accessible = Some(false);
        let mut downstairs = space("bathroom", 0, 2, st(2, 0, 0, 0));
        downstairs.grab_bars = Some(true);
        let mut bedrooms = space("bedroom", 0, 6, st(5, 1, 0, 0));
        bedrooms.beds = Some(18);
        bedrooms.hospital_beds = Some(4);
        let mut kitchen = space("kitchen", 0, 1, st(0, 0, 0, 1));
        kitchen.problems = vec!["electrical".into(), "leaks".into()];
        let rooms = space("bedroom", 1, 3, st(1, 0, 0, 0));
        SiteSummary {
            site: SiteData { name: "Casa".into(), land_m2: Some(900), built_m2: Some(650), floors: Some(2), floor_access: vec!["none".into()], ..Default::default() },
            spaces: vec![upstairs, downstairs, bedrooms, kitchen, rooms],
            equipment: vec![
                EquipmentData { kind: "washer".into(), count: 2, states: st(1, 0, 0, 1), ..Default::default() },
                EquipmentData { kind: "generator".into(), count: 1, states: st(0, 0, 0, 1), ..Default::default() },
            ],
        }
    }

    #[test]
    fn the_numbers_of_a_house_of_two_floors() {
        let i = indicators(&[house()]);
        assert_eq!((i.sites, i.built_m2, i.floors, i.only_stairs), (1, Some(650), Some(2), true));
        assert_eq!((i.spaces, i.spaces_upstairs, i.spaces_unchecked), (16, 7, 2));
        assert_eq!(i.spaces_states, st(11, 1, 1, 1));
        assert_eq!((i.bathrooms, i.bathrooms_without_bars), (6, 4));
        assert_eq!((i.beds, i.hospital_beds), (Some(18), Some(4)), "a group without beds said adds nothing, it is not zero");
        let bath = i.by_kind.iter().find(|k| k.kind == "bathroom").unwrap();
        assert_eq!((bath.count, bath.states.good, bath.states.poor, bath.unchecked), (6, 5, 1, 0));
        assert_eq!(i.by_kind.iter().find(|k| k.kind == "bedroom").unwrap().unchecked, 2);
        assert_eq!(i.not_accessible.len(), 1);
        assert_eq!(i.broken.iter().map(|g| (g.kind.as_str(), g.floor, g.bad)).collect::<Vec<_>>(), vec![("bathroom", 1, 1), ("kitchen", 0, 1)]);
        assert_eq!(i.problems.iter().map(|c| (c.code.as_str(), c.count)).collect::<Vec<_>>(), vec![("leaks", 2), ("electrical", 1), ("grab_bars", 1)]);
        assert_eq!(i.structural, 1);
        assert_eq!((i.equipment, i.broken_equipment.len(), i.generator_works), (3, 2, Some(false)));
        assert_eq!(i.missing, vec!["tenure", "water", "safety", "unchecked"]);
    }

    #[test]
    fn nothing_captured_says_what_is_missing_and_invents_nothing() {
        let i = indicators(&[]);
        assert_eq!((i.sites, i.spaces, i.beds, i.built_m2, i.generator_works), (0, 0, None, None, None));
        assert!(!i.only_stairs);
        assert_eq!(i.missing, vec!["built_m2", "floors", "tenure", "water", "safety", "spaces"]);
    }

    #[test]
    fn a_ramp_or_an_unsaid_access_is_not_only_stairs() {
        let mut h = house();
        h.site.floor_access = vec!["ramp".into()];
        assert!(!indicators(&[h.clone()]).only_stairs);
        h.site.floor_access.clear();
        let i = indicators(&[h]);
        assert!(!i.only_stairs && i.spaces_upstairs == 0);
    }
}
