//! The canonical schema and its two views: what the model is asked to fill, and what only the code fills.
//!
//! The file `schemas/canonical_call.schema.json` is the contract. Annotations in it:
//! * `x-llena: "codigo"` — the field is filled by the code (normalised values, source file, metadata);
//!   the model never sees it.
//! * `x-tipo` — which normaliser reads the field's value.
//!
//! Providers accept only part of JSON Schema, so the model's view drops what they may not take
//! (`if/then`, formats, patterns, bounds, `x-*`) and turns `const`, `oneOf` and `type: [x, null]` into
//! forms they do. Dropping loses nothing: the full schema validates the assembled result afterwards.

use serde_json::{json, Map, Value};
use std::collections::BTreeSet;

pub const CANONICAL_SCHEMA_JSON: &str = include_str!("../../../../schemas/canonical_call.schema.json");
pub const SCHEMA_VERSION: &str = "1.1.0";

/// Blocks of the schema the model fills, in reading order.
pub const SECTIONS: [&str; 8] =
    ["identidad", "temporalidad", "elegibilidad", "financiamiento", "proyecto", "documentacion", "evaluacion", "entrega"];

/// Root arrays that travel with every block: contradictions, what no field takes, and doubts.
pub const EXTRAS: [&str; 3] = ["conflictos", "otros_hallazgos", "dudas"];

pub fn canonical_schema() -> Value {
    serde_json::from_str(CANONICAL_SCHEMA_JSON).expect("the canonical schema is valid JSON")
}

/// Keys that say nothing a provider needs, or that not every provider accepts.
const DROPPED_KEYS: [&str; 15] = [
    "$schema",
    "$id",
    "$comment",
    "title",
    "if",
    "then",
    "else",
    "allOf",
    "format",
    "pattern",
    "minLength",
    "maxLength",
    "minimum",
    "maximum",
    "default",
];
const DROPPED_BOUNDS: [&str; 4] = ["exclusiveMinimum", "exclusiveMaximum", "minItems", "maxItems"];

fn filled_by_code(schema: &Value) -> bool {
    schema.get("x-llena").and_then(Value::as_str) == Some("codigo")
}

/// One schema node as the model sees it. `properties` and `$defs` hold names, not keywords, so their
/// keys are kept as they are and only their values are simplified.
fn simplify(node: &Value) -> Value {
    let Value::Object(map) = node else { return node.clone() };
    let mut out = Map::new();
    for (key, value) in map {
        let k = key.as_str();
        if k.starts_with("x-") || DROPPED_KEYS.contains(&k) || DROPPED_BOUNDS.contains(&k) {
            continue;
        }
        match k {
            "properties" => {
                let props = value.as_object().cloned().unwrap_or_default();
                let kept: Map<String, Value> =
                    props.iter().filter(|(_, v)| !filled_by_code(v)).map(|(n, v)| (n.clone(), simplify(v))).collect();
                out.insert("properties".into(), Value::Object(kept));
            }
            "$defs" => {
                let defs = value.as_object().cloned().unwrap_or_default();
                out.insert(k.into(), Value::Object(defs.iter().map(|(n, v)| (n.clone(), simplify(v))).collect()));
            }
            "required" => {} // rebuilt below, once the properties that stay are known
            "const" => {
                out.insert("enum".into(), json!([value]));
            }
            "oneOf" => {
                out.insert("anyOf".into(), simplify_list(value));
            }
            "anyOf" => {
                out.insert("anyOf".into(), simplify_list(value));
            }
            "items" | "additionalProperties" => {
                out.insert(k.into(), simplify(value));
            }
            "type" => match value {
                Value::Array(types) => {
                    out.insert("anyOf".into(), Value::Array(types.iter().map(|t| json!({ "type": t })).collect()));
                }
                other => {
                    out.insert("type".into(), other.clone());
                }
            },
            _ => {
                out.insert(k.into(), value.clone());
            }
        }
    }
    if let (Some(Value::Array(required)), Some(Value::Object(props))) = (map.get("required"), out.get("properties")) {
        let kept: Vec<Value> = required.iter().filter(|r| r.as_str().is_some_and(|n| props.contains_key(n))).cloned().collect();
        out.insert("required".into(), Value::Array(kept));
    }
    Value::Object(out)
}

fn simplify_list(value: &Value) -> Value {
    Value::Array(value.as_array().map(|a| a.iter().map(simplify).collect()).unwrap_or_default())
}

fn ref_name(node: &Value) -> Option<&str> {
    node.get("$ref").and_then(Value::as_str).and_then(|r| r.strip_prefix("#/$defs/"))
}

/// Every `$defs` name a node points at, directly or through other definitions.
fn reachable_defs(root: &Value, defs: &Map<String, Value>) -> BTreeSet<String> {
    fn walk(node: &Value, out: &mut Vec<String>) {
        match node {
            Value::Object(m) => {
                if let Some(n) = ref_name(node) {
                    out.push(n.to_string());
                }
                for (k, v) in m {
                    if k != "$defs" {
                        walk(v, out);
                    }
                }
            }
            Value::Array(a) => a.iter().for_each(|v| walk(v, out)),
            _ => {}
        }
    }
    let mut seen = BTreeSet::new();
    let mut todo = Vec::new();
    walk(root, &mut todo);
    while let Some(name) = todo.pop() {
        if seen.insert(name.clone()) {
            if let Some(d) = defs.get(&name) {
                walk(d, &mut todo);
            }
        }
    }
    seen
}

/// Replaces each `$ref` by the definition it names (a field's own description wins over the generic one).
fn inline_refs(node: &Value, defs: &Map<String, Value>) -> Value {
    match node {
        Value::Object(m) => {
            if let Some(name) = ref_name(node) {
                let mut target = inline_refs(defs.get(name).unwrap_or(&Value::Null), defs);
                if let Value::Object(t) = &mut target {
                    for (k, v) in m {
                        if k != "$ref" {
                            t.insert(k.clone(), inline_refs(v, defs));
                        }
                    }
                }
                return target;
            }
            Value::Object(m.iter().map(|(k, v)| (k.clone(), inline_refs(v, defs))).collect())
        }
        Value::Array(a) => Value::Array(a.iter().map(|v| inline_refs(v, defs)).collect()),
        other => other.clone(),
    }
}

/// The schema the model fills: the whole canonical schema (`section: None`) or one block of it plus the
/// arrays that travel with every block. With `inline` there are no `$ref`: larger, but accepted by any
/// provider.
pub fn model_schema(canon: &Value, section: Option<&str>, inline: bool) -> Value {
    match section {
        Some(s) => model_schema_of(canon, &[s], inline),
        None => model_schema_of(canon, &SECTIONS, inline),
    }
}

/// The schema for a group of blocks (and the arrays that travel with them).
pub fn model_schema_of(canon: &Value, sections: &[&str], inline: bool) -> Value {
    let mut root = canon.clone();
    let props = canon["properties"].as_object().cloned().unwrap_or_default();
    let keep: Vec<&str> = sections.iter().copied().chain(EXTRAS).collect();
    let kept: Map<String, Value> = keep.iter().filter_map(|k| props.get(*k).map(|v| (k.to_string(), v.clone()))).collect();
    root["required"] = json!(keep);
    root["properties"] = Value::Object(kept);
    // the root description of the file talks to whoever maintains the contract, not to the model
    root["description"] = match sections {
        [s] => json!(format!("Bloque «{s}» de la ficha canónica de una convocatoria.")),
        all if all.len() == SECTIONS.len() => json!("Ficha canónica completa de una convocatoria."),
        some => json!(format!("Bloques {} de la ficha canónica de una convocatoria.", some.join(", "))),
    };
    let mut root = simplify(&root);
    let defs = root.get("$defs").and_then(Value::as_object).cloned().unwrap_or_default();
    root.as_object_mut().map(|m| m.remove("$defs"));
    if inline {
        return inline_refs(&root, &defs);
    }
    let used = reachable_defs(&root, &defs);
    let kept_defs: Map<String, Value> = defs.into_iter().filter(|(n, _)| used.contains(n)).collect();
    if !kept_defs.is_empty() {
        root["$defs"] = Value::Object(kept_defs);
    }
    root
}

fn humanize(field: &str) -> String {
    field.replace('_', " ")
}

fn def_of<'a>(canon: &'a Value, node: &'a Value) -> Option<&'a Value> {
    ref_name(node).and_then(|n| canon["$defs"].get(n))
}

/// One text per field of a block, to look for the pages that talk about it: the field's own words
/// (name and description) taken from the schema, never from a list written by hand.
pub fn field_queries(canon: &Value, section: &str) -> Vec<(String, String)> {
    let Some(props) = canon["properties"][section]["properties"].as_object() else { return Vec::new() };
    props
        .iter()
        .map(|(field, node)| {
            let mut text = humanize(field);
            let own = node.get("description").and_then(Value::as_str);
            let def = def_of(canon, node);
            if let Some(d) = own {
                text = format!("{text}: {d}");
            }
            // a list's items carry the useful description one level down («TODAS las fechas del calendario…»)
            if let Some(Value::Object(inner)) = def.and_then(|d| d.get("properties")) {
                for (name, p) in inner {
                    if let Some(d) = p.get("description").and_then(Value::as_str) {
                        if name != "estado" && name != "valor_texto" && name != "evidencias" && name != "normalizado" && !d.starts_with("TODOS los elementos") {
                            text = format!("{text}. {d}");
                        }
                    }
                }
            }
            (field.clone(), text)
        })
        .collect()
}

/// What the block as a whole is about, for a score over the block rather than over a field.
pub fn section_query(canon: &Value, section: &str) -> String {
    let fields: Vec<String> = field_queries(canon, section).into_iter().map(|(_, q)| q).collect();
    format!("{}. {}", humanize(section), fields.join(". "))
}

/// The whole block with every field `no_aparece` and every list empty: what a block that could not be
/// read looks like, and the base for a simulated answer in tests.
pub fn blank_section(canon: &Value, section: &str) -> Value {
    let props = canon["properties"][section]["properties"].as_object().cloned().unwrap_or_default();
    let mut out = Map::new();
    for (field, node) in props {
        let name = ref_name(&node).unwrap_or("");
        let v = match name {
            "Lista" => json!({ "estado": "no_aparece", "elementos": [] }),
            "ListaHitos" => json!({ "estado": "no_aparece", "hitos": [] }),
            "ListaCriterios" => json!({ "estado": "no_aparece", "criterios": [] }),
            "ListaModalidades" => json!({ "estado": "no_aparece", "modalidades": [] }),
            n if n.starts_with("Dato") => json!({ "estado": "no_aparece", "valor_texto": null, "evidencias": [], "normalizado": null }),
            other => panic!("unknown definition {other} in {section}.{field}"),
        };
        out.insert(field, v);
    }
    Value::Object(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn blank_answer(section: Option<&str>) -> Value {
        let canon = canonical_schema();
        let mut m = Map::new();
        for s in SECTIONS.iter().filter(|s| section.map_or(true, |x| x == **s)) {
            let mut b = blank_section(&canon, s);
            // the model never sends what the code fills
            fn strip(v: &mut Value) {
                match v {
                    Value::Object(m) => {
                        m.remove("normalizado");
                        m.values_mut().for_each(strip);
                    }
                    Value::Array(a) => a.iter_mut().for_each(strip),
                    _ => {}
                }
            }
            strip(&mut b);
            m.insert(s.to_string(), b);
        }
        for e in EXTRAS {
            m.insert(e.to_string(), json!([]));
        }
        Value::Object(m)
    }

    #[test]
    fn the_sections_are_exactly_the_blocks_of_the_schema() {
        let canon = canonical_schema();
        let props = canon["properties"].as_object().unwrap();
        let mut blocks: Vec<&str> = props.iter().filter(|(_, v)| !filled_by_code(v) && v.get("properties").is_some()).map(|(k, _)| k.as_str()).collect();
        blocks.sort();
        let mut want: Vec<&str> = SECTIONS.to_vec();
        want.sort();
        assert_eq!(blocks, want);
        for e in EXTRAS {
            assert!(props.contains_key(e), "{e}");
        }
        assert_eq!(canon["x-version"], SCHEMA_VERSION);
    }

    #[test]
    fn the_model_never_sees_what_the_code_fills() {
        let canon = canonical_schema();
        for inline in [false, true] {
            let whole = model_schema(&canon, None, inline).to_string();
            for forbidden in ["normalizado", "metadatos", "\"documento\"", "x-llena", "x-tipo", "\"if\"", "\"then\"", "\"format\"", "\"pattern\"", "oneOf", "\"const\"", "$schema"] {
                assert!(!whole.contains(forbidden), "inline={inline}: {forbidden} reached the model");
            }
            assert!(whole.contains("\"cita\"") && whole.contains("\"pagina\"") && whole.contains("valor_texto"));
            assert_eq!(whole.contains("$ref"), !inline, "inline={inline}");
        }
    }

    #[test]
    fn a_block_schema_holds_its_section_and_the_extras_and_is_far_smaller_than_the_whole() {
        let canon = canonical_schema();
        let whole = model_schema(&canon, None, false);
        assert_eq!(whole["required"].as_array().unwrap().len(), SECTIONS.len() + EXTRAS.len());
        for s in SECTIONS {
            let b = model_schema(&canon, Some(s), false);
            let names: Vec<&str> = b["properties"].as_object().unwrap().keys().map(String::as_str).collect();
            assert!(names.contains(&s) && EXTRAS.iter().all(|e| names.contains(e)) && names.len() == 4, "{s}: {names:?}");
            assert!(b.to_string().len() < whole.to_string().len(), "{s}");
        }
        for (label, schema) in [("completo", model_schema(&canon, None, true)), ("completo con $ref", whole.clone()), ("financiamiento en línea", model_schema(&canon, Some("financiamiento"), true))] {
            println!("esquema {label}: {} bytes", schema.to_string().len());
        }
    }

    #[test]
    fn a_blank_answer_satisfies_the_model_schema_and_what_the_code_adds_satisfies_the_full_one() {
        let canon = canonical_schema();
        for inline in [false, true] {
            let v = jsonschema::validator_for(&model_schema(&canon, None, inline)).unwrap();
            let answer = blank_answer(None);
            assert!(v.is_valid(&answer), "inline={inline}: {:?}", v.iter_errors(&answer).map(|e| e.to_string()).take(3).collect::<Vec<_>>());
            // it asks for everything: a missing field is refused
            let mut missing = answer.clone();
            missing["entrega"].as_object_mut().unwrap().remove("contacto");
            assert!(!v.is_valid(&missing));
            // and a state outside the three is refused
            let mut bad = answer.clone();
            bad["identidad"]["nombre"]["estado"] = json!("quizas");
            assert!(!v.is_valid(&bad));
        }
        for s in SECTIONS {
            let v = jsonschema::validator_for(&model_schema(&canon, Some(s), false)).unwrap();
            assert!(v.is_valid(&blank_answer(Some(s))), "{s}");
        }
    }

    #[test]
    fn every_field_of_every_block_has_a_query_made_of_its_own_words() {
        let canon = canonical_schema();
        for s in SECTIONS {
            let q = field_queries(&canon, s);
            assert!(!q.is_empty(), "{s}");
            for (field, text) in &q {
                assert!(text.starts_with(&humanize(field)), "{text}");
            }
            assert!(section_query(&canon, s).len() > 100, "{s}");
        }
        let tiempo = field_queries(&canon, "temporalidad");
        assert!(tiempo.iter().any(|(f, t)| f == "hitos" && t.contains("calendario")), "{tiempo:?}");
    }
}
