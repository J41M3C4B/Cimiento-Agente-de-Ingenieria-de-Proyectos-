//! The words of the facilities (ADR-030) for the sheet of the AI and the guide in Word: the module keeps codes, this
//! says them in plain Spanish. The screen has its own words in `es-MX.ts`.

use crate::facilities::api::{GroupRef, States};
use crate::facilities::domain::group::{EquipmentData, SpaceData};
use crate::facilities::domain::site::SiteData;

fn pick<'a>(n: i64, one: &'a str, many: &'a str) -> &'a str {
    if n == 1 {
        one
    } else {
        many
    }
}

pub fn space_kind(code: &str, n: i64) -> &'static str {
    let (one, many) = match code {
        "bedroom" => ("dormitorio", "dormitorios"),
        "bathroom" => ("baño", "baños"),
        "kitchen" => ("cocina", "cocinas"),
        "dining" => ("comedor", "comedores"),
        "laundry" => ("lavandería", "lavanderías"),
        "infirmary" => ("enfermería o consultorio", "enfermerías o consultorios"),
        "therapy" => ("área de terapia o rehabilitación", "áreas de terapia o rehabilitación"),
        "living" => ("sala o salón de usos múltiples", "salas o salones de usos múltiples"),
        "classroom" => ("salón de tareas o aula", "salones de tareas o aulas"),
        "play" => ("área de juegos", "áreas de juegos"),
        "chapel" => ("capilla", "capillas"),
        "office" => ("oficina", "oficinas"),
        "storage" => ("bodega o almacén", "bodegas o almacenes"),
        "yard" => ("patio o jardín", "patios o jardines"),
        "roof" => ("azotea", "azoteas"),
        "parking" => ("estacionamiento", "estacionamientos"),
        _ => ("otro espacio", "otros espacios"),
    };
    pick(n, one, many)
}

pub fn equipment_kind(code: &str, n: i64) -> &'static str {
    let (one, many) = match code {
        "wheelchair" => ("silla de ruedas", "sillas de ruedas"),
        "patient_lift" => ("grúa para pacientes", "grúas para pacientes"),
        "pressure_mattress" => ("colchón antiescaras", "colchones antiescaras"),
        "oxygen" => ("concentrador de oxígeno", "concentradores de oxígeno"),
        "washer" => ("lavadora", "lavadoras"),
        "dryer" => ("secadora", "secadoras"),
        "fridge" => ("refrigerador", "refrigeradores"),
        "freezer" => ("congelador", "congeladores"),
        "stove" => ("estufa", "estufas"),
        "water_heater" => ("calentador de agua", "calentadores de agua"),
        "generator" => ("planta de luz de emergencia", "plantas de luz de emergencia"),
        "solar_panels" => ("sistema de paneles solares", "sistemas de paneles solares"),
        "water_pump" => ("bomba de agua", "bombas de agua"),
        "computer" => ("computadora", "computadoras"),
        "vehicle" => ("vehículo", "vehículos"),
        _ => ("otro equipo", "otros equipos"),
    };
    pick(n, one, many)
}

pub fn problem(code: &str) -> &'static str {
    match code {
        "leaks" => "fugas de agua",
        "damp" => "humedad o salitre",
        "roof" => "goteras o techo dañado",
        "electrical" => "fallas en la instalación eléctrica",
        "drainage" => "drenaje tapado o con mal olor",
        "floor" => "piso dañado o resbaloso",
        "cracks" => "grietas en muros o techo",
        "doors_windows" => "puertas o ventanas dañadas",
        "grab_bars" => "faltan barras de apoyo o pasamanos",
        "ventilation" => "poca ventilación o luz",
        "pests" => "plagas",
        "furniture" => "muebles o equipo gastado",
        "paint" => "pintura deteriorada",
        _ => "otra falla",
    }
}

/// «planta baja», «primer piso», «sótano».
pub fn floor(n: i64) -> String {
    match n {
        i64::MIN..=-1 => "sótano".into(),
        0 => "planta baja".into(),
        1 => "primer piso".into(),
        2 => "segundo piso".into(),
        3 => "tercer piso".into(),
        4 => "cuarto piso".into(),
        n => format!("piso {n}"),
    }
}

/// The name of a group: its own name when it has one, the kind otherwise.
pub fn group_name(kind: &str, label: Option<&str>, n: i64, space: bool) -> String {
    let word = if space { space_kind(kind, n) } else { equipment_kind(kind, n) };
    match label.map(str::trim).filter(|l| !l.is_empty()) {
        Some(l) if kind == "other" => l.to_string(),
        Some(l) => format!("{word} «{l}»"),
        None => word.to_string(),
    }
}

/// «3 bien, 1 mal, 2 sin revisar».
pub fn states(s: &States, count: i64) -> String {
    let mut parts = Vec::new();
    for (n, one, many) in [
        (s.good, "bien", "bien"),
        (s.fair, "regular", "regular"),
        (s.poor, "mal", "mal"),
        (s.unusable, "no se puede usar", "no se pueden usar"),
    ] {
        if n > 0 {
            parts.push(format!("{n} {}", pick(n, one, many)));
        }
    }
    let unchecked = count - s.checked();
    if unchecked > 0 {
        parts.push(format!("{unchecked} sin revisar"));
    }
    parts.join(", ")
}

fn capital(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        None => String::new(),
    }
}

/// «Baños, primer piso: 4 (3 bien, 1 mal). Fallas: fugas de agua. No se pueden usar en silla de ruedas. Sin barras de apoyo.»
pub fn space_line(g: &SpaceData) -> String {
    let mut s = format!("{}, {}: {} ({}).", capital(&group_name(&g.kind, g.label.as_deref(), g.count, true)), floor(g.floor), g.count, states(&g.states, g.count));
    if !g.problems.is_empty() {
        s.push_str(&format!(" Fallas: {}.", g.problems.iter().map(|p| problem(p)).collect::<Vec<_>>().join(", ")));
    }
    if let Some(beds) = g.beds {
        s.push_str(&format!(" {beds} {}", pick(beds, "cama", "camas")));
        match g.hospital_beds {
            Some(h) if h > 0 => s.push_str(&format!(", {h} de hospital.")),
            _ => s.push('.'),
        }
    }
    match g.accessible {
        Some(true) => s.push_str(" Se pueden usar en silla de ruedas."),
        Some(false) => s.push_str(" No se pueden usar en silla de ruedas."),
        None => {}
    }
    match g.grab_bars {
        Some(true) => s.push_str(" Con barras de apoyo."),
        Some(false) => s.push_str(" Sin barras de apoyo."),
        None => {}
    }
    match g.accessible_shower {
        Some(true) => s.push_str(" Con regadera accesible."),
        Some(false) => s.push_str(" Sin regadera accesible."),
        None => {}
    }
    if let Some(n) = g.notes.as_deref().map(str::trim).filter(|n| !n.is_empty()) {
        s.push_str(&format!(" Nota: {n}"));
    }
    s
}

/// «Lavadoras: 2 (1 bien, 1 no se puede usar).»
pub fn equipment_line(g: &EquipmentData) -> String {
    let mut s = format!("{}: {} ({}).", capital(&group_name(&g.kind, g.label.as_deref(), g.count, false)), g.count, states(&g.states, g.count));
    if let Some(n) = g.notes.as_deref().map(str::trim).filter(|n| !n.is_empty()) {
        s.push_str(&format!(" Nota: {n}"));
    }
    s
}

/// «1 de 4 baños (primer piso)».
pub fn group_ref(g: &GroupRef, space: bool) -> String {
    let name = group_name(&g.kind, g.label.as_deref(), g.count, space);
    if space {
        format!("{} de {} {name} ({})", g.bad, g.count, floor(g.floor))
    } else {
        format!("{} de {} {name}", g.bad, g.count)
    }
}

fn tenure(code: &str) -> &'static str {
    match code {
        "own" => "es propio",
        "loan" => "está en comodato",
        "rent" => "es rentado",
        "borrowed" => "está prestado, sin papeles",
        _ => "se tiene de otra forma",
    }
}

fn frequency(code: &str) -> &'static str {
    match code {
        "never" => "nunca",
        "sometimes" => "a veces",
        _ => "seguido",
    }
}

fn yes_no(b: bool) -> &'static str {
    if b {
        "sí"
    } else {
        "no"
    }
}

/// The building, its services and its safety, one line each; only what was captured.
pub fn site_lines(d: &SiteData) -> Vec<String> {
    let mut out = Vec::new();

    let mut b: Vec<String> = Vec::new();
    if let Some(f) = d.floors {
        b.push(if f == 1 { "1 piso (planta baja)".into() } else { format!("{f} pisos contando la planta baja") });
    }
    if d.floors.is_some_and(|f| f > 1) && !d.floor_access.is_empty() {
        let ways: Vec<&str> = d
            .floor_access
            .iter()
            .map(|a| match a.as_str() {
                "ramp" => "rampa",
                "elevator" => "elevador",
                "stair_lift" => "silla salvaescaleras",
                _ => "solo escaleras",
            })
            .collect();
        b.push(format!("entre pisos: {}", ways.join(", ")));
    }
    if let Some(m) = d.land_m2 {
        b.push(format!("terreno de {m} m²"));
    }
    if let Some(m) = d.built_m2 {
        b.push(format!("{m} m² construidos"));
    }
    if let Some(y) = d.built_year {
        b.push(format!("construido hacia {y}"));
    }
    if !b.is_empty() {
        out.push(format!("Inmueble «{}»: {}.", d.name, b.join("; ")));
    }

    if let Some(t) = d.tenure.as_deref() {
        let mut s = format!("El inmueble {}", tenure(t));
        if let Some(y) = d.tenure_until {
            s.push_str(&format!(" hasta {y}"));
        }
        match d.tenure_documented {
            Some(true) => s.push_str("; tiene papeles que lo acreditan (escritura o contrato)"),
            Some(false) => s.push_str("; no tiene papeles que lo acrediten"),
            None => {}
        }
        out.push(format!("{s}."));
    }

    let mut sv: Vec<String> = Vec::new();
    if !d.water_sources.is_empty() {
        let w: Vec<&str> = d
            .water_sources
            .iter()
            .map(|x| match x.as_str() {
                "network" => "red municipal",
                "truck" => "pipas",
                "well" => "pozo",
                _ => "captación de lluvia",
            })
            .collect();
        sv.push(format!("agua de {}", w.join(" y ")));
    }
    if let Some(f) = d.water_shortage.as_deref() {
        sv.push(format!("falta el agua: {}", frequency(f)));
    }
    if let Some(l) = d.water_storage_liters {
        sv.push(format!("guarda {l} litros de agua (cisterna y tinacos)"));
    }
    if let Some(f) = d.power_outages.as_deref() {
        sv.push(format!("se va la luz: {}", frequency(f)));
    }
    if let Some(g) = d.gas.as_deref() {
        sv.push(match g {
            "lp_tank" => "gas LP en tanque estacionario".into(),
            "lp_cylinders" => "gas LP en cilindros".into(),
            "natural" => "gas natural".into(),
            _ => "sin gas".to_string(),
        });
    }
    if let Some(x) = d.drainage.as_deref() {
        sv.push(match x {
            "sewer" => "drenaje a la red municipal".into(),
            "septic" => "fosa séptica".into(),
            _ => "sin drenaje".to_string(),
        });
    }
    if let Some(i) = d.internet {
        sv.push(format!("internet: {}", yes_no(i)));
    }
    if !sv.is_empty() {
        out.push(format!("Servicios: {}.", sv.join("; ")));
    }

    let mut sf: Vec<String> = Vec::new();
    if let Some(n) = d.extinguishers {
        let mut s = format!("{n} {}", pick(n, "extintor", "extintores"));
        if let Some(c) = d.extinguishers_current {
            s.push_str(if c { " con la recarga al día" } else { " sin la recarga al día" });
        }
        sf.push(s);
    }
    if let Some(n) = d.smoke_detectors {
        sf.push(format!("{n} {}", pick(n, "detector de humo", "detectores de humo")));
    }
    for (v, text) in [(d.marked_exits, "salidas y rutas de evacuación señaladas"), (d.emergency_lights, "luces de emergencia"), (d.first_aid_kit, "botiquín")] {
        if let Some(b) = v {
            sf.push(format!("{text}: {}", yes_no(b)));
        }
    }
    if let Some(p) = d.internal_program.as_deref() {
        sf.push(format!(
            "programa interno de protección civil: {}",
            match p {
                "yes" => "sí",
                "in_progress" => "en trámite",
                _ => "no",
            }
        ));
    }
    if let Some(o) = d.civil_protection_opinion {
        let mut s = format!("dictamen o visto bueno de protección civil: {}", yes_no(o));
        if let Some(y) = d.opinion_year {
            s.push_str(&format!(" (de {y})"));
        }
        sf.push(s);
    }
    if let Some(n) = d.drills_per_year {
        sf.push(format!("{n} {} al año", pick(n, "simulacro", "simulacros")));
    }
    if !sf.is_empty() {
        out.push(format!("Seguridad y protección civil: {}.", sf.join("; ")));
    }
    if let Some(n) = d.notes.as_deref().map(str::trim).filter(|n| !n.is_empty()) {
        out.push(format!("Nota del inmueble: {n}"));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_group_of_bathrooms_in_words() {
        let g = SpaceData {
            kind: "bathroom".into(),
            floor: 1,
            count: 4,
            states: States { good: 3, poor: 1, ..Default::default() },
            problems: vec!["leaks".into(), "grab_bars".into()],
            accessible: Some(false),
            grab_bars: Some(false),
            ..Default::default()
        };
        assert_eq!(
            space_line(&g),
            "Baños, primer piso: 4 (3 bien, 1 mal). Fallas: fugas de agua, faltan barras de apoyo o pasamanos. No se pueden usar en silla de ruedas. Sin barras de apoyo."
        );
        let r = GroupRef { kind: "bathroom".into(), label: None, floor: 1, count: 4, bad: 1 };
        assert_eq!(group_ref(&r, true), "1 de 4 baños (primer piso)");
    }

    #[test]
    fn own_names_unchecked_spaces_beds_and_equipment() {
        let g = SpaceData { kind: "bedroom".into(), label: Some("Dormitorio de mujeres".into()), count: 6, states: States { good: 5, ..Default::default() }, beds: Some(18), hospital_beds: Some(4), ..Default::default() };
        assert_eq!(space_line(&g), "Dormitorios «Dormitorio de mujeres», planta baja: 6 (5 bien, 1 sin revisar). 18 camas, 4 de hospital.");
        let o = SpaceData { kind: "other".into(), label: Some("Taller de costura".into()), count: 1, states: States { unusable: 1, ..Default::default() }, ..Default::default() };
        assert_eq!(space_line(&o), "Taller de costura, planta baja: 1 (1 no se puede usar).");
        let e = EquipmentData { kind: "washer".into(), count: 2, states: States { good: 1, unusable: 1, ..Default::default() }, ..Default::default() };
        assert_eq!(equipment_line(&e), "Lavadoras: 2 (1 bien, 1 no se puede usar).");
    }

    #[test]
    fn the_site_says_only_what_was_captured() {
        assert!(site_lines(&SiteData { name: "Casa".into(), ..Default::default() }).is_empty());
        let d = SiteData {
            name: "Casa principal".into(),
            land_m2: Some(900),
            built_m2: Some(650),
            floors: Some(2),
            floor_access: vec!["none".into()],
            tenure: Some("loan".into()),
            tenure_until: Some(2031),
            tenure_documented: Some(true),
            water_sources: vec!["network".into(), "truck".into()],
            water_shortage: Some("often".into()),
            extinguishers: Some(4),
            extinguishers_current: Some(false),
            internal_program: Some("in_progress".into()),
            ..Default::default()
        };
        assert_eq!(
            site_lines(&d),
            vec![
                "Inmueble «Casa principal»: 2 pisos contando la planta baja; entre pisos: solo escaleras; terreno de 900 m²; 650 m² construidos.",
                "El inmueble está en comodato hasta 2031; tiene papeles que lo acreditan (escritura o contrato).",
                "Servicios: agua de red municipal y pipas; falta el agua: seguido.",
                "Seguridad y protección civil: 4 extintores sin la recarga al día; programa interno de protección civil: en trámite.",
            ]
        );
    }
}
