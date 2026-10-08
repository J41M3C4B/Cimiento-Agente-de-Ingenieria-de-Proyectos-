//! The move of the spaces of the old profile list (ADR-023) into this module (ADR-030). The written kind becomes a
//! code of the catalog («Baño» -> `bathroom`), and the words of their own stay as its name when they say more
//! («Baños de mujeres»). The one state of the old row is given to every space of the group; everything goes on the
//! ground floor of the main site; origin and source are kept.

use super::domain::group::{SpaceData, States};
use super::storage as store;
use super::FacilitiesError;
use rusqlite::Connection;

pub struct LegacyFacility {
    pub kind: String,
    pub count: i64,
    /// `good`, `fair`, `poor` or `critical`.
    pub condition: Option<String>,
    pub accessible: Option<bool>,
    pub notes: Option<String>,
    pub origin: String,
    pub source_ref: Option<String>,
}

/// Words that say each kind, checked in this order (the first that matches wins).
const WORDS: &[(&str, &[&str])] = &[
    ("bathroom", &["baño", "bano", "sanitario", "regadera", "wc"]),
    ("bedroom", &["dormitorio", "habitaci", "recámara", "recamara"]),
    ("kitchen", &["cocina"]),
    ("dining", &["comedor"]),
    ("laundry", &["lavander", "lavadero"]),
    ("infirmary", &["enfermer", "consultorio", "médic", "medic"]),
    ("therapy", &["terapia", "rehabilit", "fisio", "gimnasio"]),
    ("classroom", &["aula", "salón de tareas", "salon de tareas", "biblioteca", "estudio", "computo", "cómputo"]),
    ("play", &["juego", "ludoteca"]),
    ("chapel", &["capilla", "oratorio"]),
    ("office", &["oficina", "dirección", "direccion", "administraci", "recepci"]),
    ("storage", &["bodega", "almacén", "almacen", "despensa"]),
    ("yard", &["patio", "jardín", "jardin", "huerto"]),
    ("roof", &["azotea", "techo"]),
    ("parking", &["estacionamiento", "cochera"]),
    ("living", &["sala", "usos múltiples", "usos multiples", "estancia", "recreaci"]),
];

/// The plain words of each kind: a name equal to one of them is not kept as a name of its own.
const PLAIN: &[(&str, &[&str])] = &[
    ("bathroom", &["baño", "baños", "sanitario", "sanitarios"]),
    ("bedroom", &["dormitorio", "dormitorios", "habitación", "habitaciones", "cuarto", "cuartos", "recámara", "recámaras"]),
    ("kitchen", &["cocina", "cocinas"]),
    ("dining", &["comedor", "comedores"]),
    ("laundry", &["lavandería", "lavanderías"]),
    ("infirmary", &["enfermería", "enfermerías", "consultorio", "consultorios"]),
    ("therapy", &["terapia", "rehabilitación"]),
    ("classroom", &["aula", "aulas"]),
    ("play", &["ludoteca", "área de juegos"]),
    ("chapel", &["capilla", "capillas"]),
    ("office", &["oficina", "oficinas"]),
    ("storage", &["bodega", "bodegas", "almacén"]),
    ("yard", &["patio", "patios", "jardín", "jardines"]),
    ("roof", &["azotea"]),
    ("parking", &["estacionamiento"]),
    ("living", &["sala", "salas"]),
];

/// The kind a written name says, and the name to keep (if it says more than the kind).
pub fn kind_of(written: &str) -> (&'static str, Option<String>) {
    let w = written.trim().to_lowercase();
    let plain_kind = PLAIN.iter().find(|(_, words)| words.contains(&w.as_str())).map(|(k, _)| *k);
    let Some(kind) = plain_kind.or_else(|| WORDS.iter().find(|(_, words)| words.iter().any(|x| w.contains(x))).map(|(k, _)| *k)) else {
        return ("other", Some(written.trim().to_string()).filter(|s| !s.is_empty()).or_else(|| Some("Espacio".into())));
    };
    (kind, plain_kind.is_none().then(|| written.trim().to_string()))
}

/// Moves the old rows, inside the caller's transaction, into the main site. Returns how many groups moved.
pub fn import(conn: &Connection, rows: &[LegacyFacility]) -> Result<usize, FacilitiesError> {
    if rows.is_empty() {
        return Ok(0);
    }
    let site_id = store::main_site_id(conn)?;
    for r in rows {
        let (kind, label) = kind_of(&r.kind);
        let count = r.count.max(1);
        let state = match r.condition.as_deref() {
            Some("critical") => "unusable",
            Some(s) => s,
            None => "",
        };
        let mut data = SpaceData {
            kind: kind.into(),
            label,
            floor: 0,
            count,
            states: States::all(count, state),
            accessible: r.accessible,
            notes: r.notes.clone(),
            ..Default::default()
        };
        data.tidy();
        store::save_space(conn, None, &site_id, &data, &r.origin, r.source_ref.as_deref())?;
    }
    Ok(rows.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn written_kinds_become_codes_and_keep_what_they_say_more() {
        assert_eq!(kind_of("Baño"), ("bathroom", None));
        assert_eq!(kind_of("baños"), ("bathroom", None));
        assert_eq!(kind_of("Baños de mujeres"), ("bathroom", Some("Baños de mujeres".into())));
        assert_eq!(kind_of("Dormitorio"), ("bedroom", None));
        assert_eq!(kind_of("Enfermería"), ("infirmary", None));
        assert_eq!(kind_of("Sala de usos múltiples"), ("living", Some("Sala de usos múltiples".into())));
        assert_eq!(kind_of("Cuartos"), ("bedroom", None));
        assert_eq!(kind_of("Cuarto de máquinas"), ("other", Some("Cuarto de máquinas".into())));
        assert_eq!(kind_of("Taller de costura"), ("other", Some("Taller de costura".into())));
    }
}
