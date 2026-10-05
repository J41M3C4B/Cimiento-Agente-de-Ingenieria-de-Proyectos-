//! From the model's answers to the canonical document: the part that is the code's.
//!
//! The model's answers are claims. Here each one is checked and only what survives enters:
//! * every quote must be in the page it cites (a quote found on another page is moved there, one that is
//!   nowhere is thrown away);
//! * a value must be written in its quote; dates, amounts, percentages and durations are READ from it;
//! * evidence beats declaration: a field with a verified quote is not «no_aparece», and a field whose
//!   quotes all failed is not «encontrado»;
//! * contradictions are kept, never resolved;
//! * the assembled document is validated against the full schema.
//!
//! Nothing here knows how a funder writes its calls.

use super::contract::{canonical_schema, SCHEMA_VERSION, SECTIONS};
use super::normalize::{self, Ctx};
use super::package::Package;
use crate::documents::text::{norm, same_statement};
use regex::Regex;
use serde::Serialize;
use serde_json::{json, Map, Value};
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::sync::OnceLock;

/// The text with the words cut by a hyphen at the end of a line put back together («ga- rantizar» is
/// «garantizar»): a model that reads a justified page writes the word whole, and the quote is still the page's.
fn dehyphen(s: &str) -> String {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r"(\p{L})-\s+(\p{Ll})").expect("valid regex")).replace_all(s, "$1$2").to_string()
}

/// Where a normalized quote sits in a normalized page: exactly, or, when the page cuts some of its words with a
/// hyphen, from its first words to its last ones that the page spells the same way.
fn locate(page: &str, q: &str) -> Option<(usize, usize)> {
    if let Some(start) = page.find(q) {
        return Some((start, start + q.len()));
    }
    let words: Vec<&str> = q.split(' ').collect();
    let start = (1..=3.min(words.len())).rev().find_map(|n| page.find(&words[..n].join(" ")))?;
    let end = (1..=3.min(words.len())).rev().find_map(|n| {
        let tail = words[words.len() - n..].join(" ");
        page[start..].rfind(&tail).map(|i| start + i + tail.len())
    })?;
    (end > start).then_some((start, end))
}

/// The lines of a page and the cells of its table rows, each one normalized: what a short quote may be.
fn whole_units(text: &str) -> HashSet<String> {
    let mut out = HashSet::new();
    for line in text.lines() {
        out.insert(norm(line));
        if line.contains('|') {
            out.extend(line.split('|').map(norm));
        }
    }
    out.remove("");
    out
}

/// A page whose verified quotes cover less than this share of its text is «thin»: it may still hold what
/// the reading missed, even if one quote came from it (a page with a calendar and a list of expenses).
pub const THIN_PAGE_COVERAGE: f64 = 0.6;
/// A quote shorter than this proves nothing, unless it is a whole line or a whole cell of its page (an item of a
/// list or of a table: «Software», «Seguros»).
const MIN_QUOTE_CHARS: usize = 12;
/// ...and not even a whole cell counts if it is shorter than this.
const MIN_CELL_CHARS: usize = 3;
/// How many rejections the report lists in full (the counts are always complete).
const MAX_LISTED: usize = 80;

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct FieldCounts {
    pub encontrado: usize,
    pub ambiguo: usize,
    pub no_aparece: usize,
    /// Items inside the lists (elements, milestones, criteria, modalities).
    pub items: usize,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct Note {
    pub path: String,
    pub why: String,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct AssemblyReport {
    pub citations_emitted: usize,
    pub citations_verified: usize,
    /// Quotes that were right but cited another page: moved to where they are.
    pub citations_relocated: usize,
    pub citations_rejected: usize,
    /// A value that was not in its quote and was replaced by the quote.
    pub values_adjusted: usize,
    /// Fields where a verified quote overruled the declared state, or the opposite.
    pub states_fixed: usize,
    pub items_dropped: usize,
    pub duplicates: usize,
    pub normalized: usize,
    pub not_normalized: usize,
    pub conflicts_kept: usize,
    pub conflicts_dropped: usize,
    pub findings_kept: usize,
    pub findings_dropped: usize,
    pub sections_missing: Vec<String>,
    pub schema_errors: Vec<String>,
    pub fields: FieldCounts,
    pub per_section: BTreeMap<String, FieldCounts>,
    pub rejected: Vec<Note>,
    pub unread: Vec<Note>,
    /// Pages with at least one verified quote anywhere in the document, and the ones with none. A page
    /// nobody quotes is not necessarily empty, but it is where what the reading missed is to be looked for.
    pub pages_cited: Vec<usize>,
    pub pages_uncited: Vec<usize>,
    /// Pages whose quotes cover less than `THIN_PAGE_COVERAGE` of their text, the uncited ones included:
    /// where the second reading looks.
    pub pages_thin: Vec<usize>,
    /// Share of the package's text inside a verified quote, 0 to 1.
    pub text_coverage: f64,
}

impl AssemblyReport {
    /// Share of emitted quotes that were found in the package, 0 to 1.
    #[cfg_attr(not(test), allow(dead_code))] // used by the measurement tests, not by the application
    pub fn verified_rate(&self) -> f64 {
        if self.citations_emitted == 0 {
            1.0
        } else {
            self.citations_verified as f64 / self.citations_emitted as f64
        }
    }

    /// Share of the package's pages that carry at least one verified quote, 0 to 1. It needs no hand-written
    /// reference, so it measures any package.
    #[cfg_attr(not(test), allow(dead_code))] // used by the measurement tests, not by the application
    pub fn page_coverage(&self) -> f64 {
        let total = self.pages_cited.len() + self.pages_uncited.len();
        if total == 0 {
            1.0
        } else {
            self.pages_cited.len() as f64 / total as f64
        }
    }
}

/// What the run knows about the reading, for the `metadatos` block.
#[derive(Debug, Clone, Default, Serialize)]
pub struct Reading {
    pub model: String,
    pub at: String,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cost_mxn: f64,
}

/// Answers of the model, whole or by block, in the shape of `model_schema`.
#[derive(Debug, Clone, Default)]
pub struct Answers {
    pub sections: BTreeMap<String, Value>,
    pub conflicts: Vec<Value>,
    pub findings: Vec<Value>,
    pub doubts: Vec<Value>,
}

impl Answers {
    /// Adds an answer that carries some sections (all of them, or one block) and the extras.
    pub fn add(&mut self, answer: &Value) {
        for s in SECTIONS {
            if answer[s].is_object() {
                self.sections.insert(s.to_string(), answer[s].clone());
            }
        }
        let arr = |k: &str| answer[k].as_array().cloned().unwrap_or_default();
        self.conflicts.extend(arr("conflictos"));
        self.findings.extend(arr("otros_hallazgos"));
        self.doubts.extend(arr("dudas"));
    }
}

/// The keys under which a list field holds its items.
const LIST_KEYS: [&str; 4] = ["elementos", "hitos", "criterios", "modalidades"];

impl Answers {
    /// Adds what a later reading found in pages the first one left uncited. Lists grow; a single value is
    /// taken only where the first reading left it empty (two different values are not for the code to settle).
    /// Whatever is added still goes through the same checks as everything else when the document is assembled.
    pub fn merge(&mut self, answer: &Value) {
        for s in SECTIONS {
            let Some(new) = answer[s].as_object() else { continue };
            let Some(old) = self.sections.get_mut(s).and_then(Value::as_object_mut) else {
                self.sections.insert(s.to_string(), answer[s].clone());
                continue;
            };
            for (field, nv) in new {
                let Some(ov) = old.get_mut(field).and_then(Value::as_object_mut) else {
                    old.insert(field.clone(), nv.clone());
                    continue;
                };
                if let Some(key) = LIST_KEYS.iter().find(|k| nv[**k].is_array()) {
                    let add = nv[*key].as_array().cloned().unwrap_or_default();
                    if add.is_empty() {
                        continue;
                    }
                    match ov.get_mut(*key).and_then(Value::as_array_mut) {
                        Some(list) => list.extend(add),
                        None => {
                            ov.insert(key.to_string(), Value::Array(add));
                        }
                    }
                    if ov.get("estado").and_then(Value::as_str) != Some("ambiguo") {
                        ov.insert("estado".into(), json!("encontrado"));
                    }
                } else {
                    let empty = ov.get("estado").and_then(Value::as_str).map_or(true, |e| e == "no_aparece") || ov.get("evidencias").and_then(Value::as_array).map_or(true, Vec::is_empty);
                    if empty && nv["estado"].as_str().is_some_and(|e| e != "no_aparece") {
                        ov.clear();
                        ov.extend(nv.as_object().cloned().unwrap_or_default());
                    }
                }
            }
        }
        let arr = |k: &str| answer[k].as_array().cloned().unwrap_or_default();
        self.conflicts.extend(arr("conflictos"));
        self.findings.extend(arr("otros_hallazgos"));
        self.doubts.extend(arr("dudas"));
    }
}

struct Assembler<'a> {
    pkg: &'a Package,
    canon: &'a Value,
    ctx: Ctx,
    pages: BTreeMap<usize, String>,
    /// The same pages with the words cut at the end of a line put back together.
    joined: BTreeMap<usize, String>,
    /// Whole lines and whole cells of each page, for the quotes too short to prove anything by themselves.
    units: BTreeMap<usize, HashSet<String>>,
    cited: BTreeSet<usize>,
    /// Where each verified quote sits in its page (byte range of the normalized text).
    spans: BTreeMap<usize, Vec<(usize, usize)>>,
    report: AssemblyReport,
}

fn text_or_null(v: &Value) -> Value {
    match v.as_str().map(str::trim) {
        Some(s) if !s.is_empty() => json!(s),
        _ => Value::Null,
    }
}

fn def_name(node: &Value) -> &str {
    node.get("$ref").and_then(Value::as_str).and_then(|r| r.strip_prefix("#/$defs/")).unwrap_or("")
}

impl<'a> Assembler<'a> {
    fn new(pkg: &'a Package, canon: &'a Value) -> Self {
        let pages = pkg.pages.iter().map(|p| (p.number, norm(&p.text))).collect();
        let joined = pkg.pages.iter().map(|p| (p.number, norm(&dehyphen(&p.text)))).collect();
        let units = pkg.pages.iter().map(|p| (p.number, whole_units(&p.text))).collect();
        Assembler { pkg, canon, ctx: Ctx { year: pkg.dominant_year(), pesos: pkg.mentions_pesos() }, pages, joined, units, cited: BTreeSet::new(), spans: BTreeMap::new(), report: AssemblyReport::default() }
    }

    fn reject(&mut self, path: &str, why: &str) {
        if self.report.rejected.len() < MAX_LISTED {
            self.report.rejected.push(Note { path: path.to_string(), why: why.to_string() });
        }
    }

    fn unread(&mut self, path: &str, why: &str) {
        self.report.not_normalized += 1;
        if self.report.unread.len() < MAX_LISTED {
            self.report.unread.push(Note { path: path.to_string(), why: why.to_string() });
        }
    }

    /// The quote is in the page: as it is written, or with the words cut at the end of a line put back together; a
    /// short one only if it is a whole line or a whole cell of the page.
    fn is_on(&self, page: usize, q: &str, joined: &str) -> bool {
        if q.chars().count() < MIN_QUOTE_CHARS {
            return q.chars().count() >= MIN_CELL_CHARS && self.units.get(&page).is_some_and(|u| u.contains(q));
        }
        self.pages.get(&page).is_some_and(|t| t.contains(q)) || self.joined.get(&page).is_some_and(|t| t.contains(joined))
    }

    /// `{cita, pagina, documento}` if the quote is in the package; the page is corrected if it is on another.
    fn evidence(&mut self, ev: &Value, path: &str) -> Option<Value> {
        self.report.citations_emitted += 1;
        let quote = ev["cita"].as_str().unwrap_or("").trim();
        let cited = ev["pagina"].as_u64().unwrap_or(0) as usize;
        let q = norm(quote);
        let joined = norm(&dehyphen(quote));
        if q.chars().count() < MIN_CELL_CHARS {
            self.report.citations_rejected += 1;
            self.reject(path, "la cita es demasiado corta o está vacía");
            return None;
        }
        let page = if self.is_on(cited, &q, &joined) {
            cited
        } else {
            let hit = self.pages.keys().copied().filter(|n| self.is_on(*n, &q, &joined)).min_by_key(|n| n.abs_diff(cited));
            match hit {
                Some(n) => {
                    self.report.citations_relocated += 1;
                    n
                }
                None => {
                    self.report.citations_rejected += 1;
                    self.reject(path, &format!("la cita no aparece en ninguna página: «{}»", quote.chars().take(70).collect::<String>()));
                    return None;
                }
            }
        };
        self.report.citations_verified += 1;
        self.cited.insert(page);
        // where the quote sits, for the share of the page it covers
        if let Some(span) = self.pages.get(&page).and_then(|t| locate(t, &q)) {
            self.spans.entry(page).or_default().push(span);
        }
        let document = self.pkg.page(page).map(|p| p.document.clone()).unwrap_or_default();
        Some(json!({ "cita": quote, "pagina": page, "documento": document }))
    }

    fn evidences(&mut self, v: &Value, path: &str) -> Vec<Value> {
        v.as_array().cloned().unwrap_or_default().iter().filter_map(|e| self.evidence(e, path)).collect()
    }

    fn state_of(declared: &str, has_evidence: bool) -> &'static str {
        match (has_evidence, declared) {
            (false, _) => "no_aparece",
            (true, "ambiguo") => "ambiguo",
            (true, _) => "encontrado",
        }
    }

    /// A single value: its state, its text as written, its proofs and, read by the code, its normal form.
    fn dato(&mut self, v: &Value, kind: &str, path: &str) -> Value {
        let declared = v["estado"].as_str().unwrap_or("no_aparece");
        let evidences = self.evidences(&v["evidencias"], path);
        let state = Self::state_of(declared, !evidences.is_empty());
        if (declared == "no_aparece") != (state == "no_aparece") {
            self.report.states_fixed += 1;
        }
        if evidences.is_empty() {
            return json!({ "estado": "no_aparece", "valor_texto": null, "evidencias": [], "normalizado": null });
        }
        let quotes: Vec<String> = evidences.iter().map(|e| norm(e["cita"].as_str().unwrap_or(""))).collect();
        let said = v["valor_texto"].as_str().map(str::trim).unwrap_or("");
        let value = if !said.is_empty() && quotes.iter().any(|q| q.contains(&norm(said))) {
            said.to_string()
        } else {
            if !said.is_empty() {
                self.report.values_adjusted += 1;
            }
            evidences[0]["cita"].as_str().unwrap_or("").to_string()
        };
        let page = evidences[0]["pagina"].as_u64().unwrap_or(0) as usize;
        let quote = evidences[0]["cita"].as_str().unwrap_or("");
        let normalized = if kind == "texto" { None } else { normalize::normalize(kind, &value, quote, page, &self.ctx) };
        match (&normalized, kind) {
            (Some(_), _) => self.report.normalized += 1,
            (None, "texto") => {}
            (None, _) => self.unread(path, &format!("no se pudo leer como {kind}: «{}»", value.chars().take(60).collect::<String>())),
        }
        json!({ "estado": state, "valor_texto": value, "evidencias": evidences, "normalizado": normalized })
    }

    fn element(&mut self, e: &Value, path: &str) -> Option<Value> {
        let evidences = self.evidences(&e["evidencias"], path);
        if evidences.is_empty() {
            self.report.items_dropped += 1;
            return None;
        }
        Some(json!({
            "titulo": text_or_null(&e["titulo"]),
            "evidencias": evidences,
            "aplica_a": text_or_null(&e["aplica_a"]),
            "obligatoriedad": e["obligatoriedad"].as_str().unwrap_or("no_indicado"),
            "nota": text_or_null(&e["nota"]),
        }))
    }

    /// Drops what repeats an earlier item of the same list (same quote, or the same statement).
    fn dedupe(&mut self, items: Vec<Value>, key: impl Fn(&Value) -> String) -> Vec<Value> {
        let mut kept: Vec<Value> = Vec::new();
        let mut seen: Vec<String> = Vec::new();
        for item in items {
            let k = norm(&key(&item));
            if seen.iter().any(|s| same_statement(s, &k)) {
                self.report.duplicates += 1;
                continue;
            }
            seen.push(k);
            kept.push(item);
        }
        kept
    }

    fn list_state(&mut self, declared: &str, items: usize) -> &'static str {
        let state = Self::state_of(declared, items > 0);
        if (declared == "no_aparece") != (state == "no_aparece") {
            self.report.states_fixed += 1;
        }
        state
    }

    fn lista(&mut self, v: &Value, path: &str) -> Value {
        let elements: Vec<Value> = v["elementos"].as_array().cloned().unwrap_or_default().iter().filter_map(|e| self.element(e, path)).collect();
        let elements = self.dedupe(elements, |e| e["evidencias"][0]["cita"].as_str().unwrap_or("").to_string());
        let state = self.list_state(v["estado"].as_str().unwrap_or("no_aparece"), elements.len());
        json!({ "estado": state, "elementos": elements })
    }

    fn hitos(&mut self, v: &Value, path: &str) -> Value {
        let mut out = Vec::new();
        for h in v["hitos"].as_array().cloned().unwrap_or_default() {
            let evidences = self.evidences(&h["evidencias"], path);
            if evidences.is_empty() {
                self.report.items_dropped += 1;
                continue;
            }
            let (start, end) = (h["inicio_texto"].as_str().unwrap_or("").trim(), h["fin_texto"].as_str().map(str::trim));
            let page = evidences[0]["pagina"].as_u64().unwrap_or(0) as usize;
            let quote = evidences[0]["cita"].as_str().unwrap_or("");
            let normalized = normalize::read_milestone(start, end, quote, page, &self.ctx);
            match &normalized {
                Some(_) => self.report.normalized += 1,
                None => self.unread(path, &format!("hito sin fecha legible: «{}»", quote.chars().take(60).collect::<String>())),
            }
            out.push(json!({
                "tipo": h["tipo"].as_str().unwrap_or("otro"),
                "etiqueta": text_or_null(&h["etiqueta"]),
                "inicio_texto": text_or_null(&h["inicio_texto"]),
                "fin_texto": text_or_null(&h["fin_texto"]),
                "evidencias": evidences,
                "normalizado": normalized,
            }));
        }
        let out = self.dedupe(out, |h| format!("{} {}", h["tipo"].as_str().unwrap_or(""), h["evidencias"][0]["cita"].as_str().unwrap_or("")));
        let state = self.list_state(v["estado"].as_str().unwrap_or("no_aparece"), out.len());
        json!({ "estado": state, "hitos": out })
    }

    fn criterios(&mut self, v: &Value, path: &str) -> Value {
        let mut out = Vec::new();
        for c in v["criterios"].as_array().cloned().unwrap_or_default() {
            let evidences = self.evidences(&c["evidencias"], path);
            if evidences.is_empty() {
                self.report.items_dropped += 1;
                continue;
            }
            let weight = if c["ponderacion"].is_object() {
                let text = format!("{} {}", c["ponderacion"]["valor_texto"].as_str().unwrap_or(""), evidences[0]["cita"].as_str().unwrap_or(""));
                let kind = if text.contains('%') || norm(&text).contains("por ciento") { "porcentaje" } else { "entero" };
                let d = self.dato(&c["ponderacion"], kind, path);
                (d["estado"] != "no_aparece").then_some(d)
            } else {
                None
            };
            out.push(json!({ "titulo": text_or_null(&c["titulo"]), "ponderacion": weight, "evidencias": evidences, "nota": text_or_null(&c["nota"]) }));
        }
        let out = self.dedupe(out, |c| c["evidencias"][0]["cita"].as_str().unwrap_or("").to_string());
        let state = self.list_state(v["estado"].as_str().unwrap_or("no_aparece"), out.len());
        json!({ "estado": state, "criterios": out })
    }

    fn modalidades(&mut self, v: &Value, path: &str) -> Value {
        let mut out = Vec::new();
        for m in v["modalidades"].as_array().cloned().unwrap_or_default() {
            let name = m["nombre"].as_str().unwrap_or("").trim().to_string();
            let evidences = self.evidences(&m["evidencias"], path);
            if evidences.is_empty() || name.is_empty() {
                self.report.items_dropped += 1;
                continue;
            }
            let p = |f: &str| format!("{path}[{name}].{f}");
            out.push(json!({
                "nombre": name,
                "evidencias": evidences,
                "monto_minimo": self.dato(&m["monto_minimo"], "monto", &p("monto_minimo")),
                "monto_maximo": self.dato(&m["monto_maximo"], "monto", &p("monto_maximo")),
                "porcentaje_cofinanciamiento": self.dato(&m["porcentaje_cofinanciamiento"], "porcentaje", &p("porcentaje_cofinanciamiento")),
                "duracion_maxima": self.dato(&m["duracion_maxima"], "duracion", &p("duracion_maxima")),
                "requisitos_propios": self.lista(&m["requisitos_propios"], &p("requisitos_propios")),
            }));
        }
        let state = self.list_state(v["estado"].as_str().unwrap_or("no_aparece"), out.len());
        json!({ "estado": state, "modalidades": out })
    }

    fn field(&mut self, def: &str, v: &Value, path: &str) -> Value {
        match def {
            "Lista" => self.lista(v, path),
            "ListaHitos" => self.hitos(v, path),
            "ListaCriterios" => self.criterios(v, path),
            "ListaModalidades" => self.modalidades(v, path),
            "DatoTexto" => self.dato(v, "texto", path),
            "DatoTipoDocumento" => self.dato(v, "tipo_documento", path),
            "DatoFecha" => self.dato(v, "fecha", path),
            "DatoMonto" => self.dato(v, "monto", path),
            "DatoPorcentaje" => self.dato(v, "porcentaje", path),
            "DatoDuracion" => self.dato(v, "duracion", path),
            "DatoEntero" => self.dato(v, "entero", path),
            other => panic!("the canonical schema has a field of an unknown definition: {other}"),
        }
    }

    fn section(&mut self, name: &str, answer: Option<&Value>) -> Value {
        let blank = super::contract::blank_section(self.canon, name);
        let props = self.canon["properties"][name]["properties"].as_object().cloned().unwrap_or_default();
        let mut out = Map::new();
        for (field, node) in props {
            let given = answer.and_then(|a| a.get(&field)).filter(|v| v.is_object()).unwrap_or(&blank[&field]);
            let v = self.field(def_name(&node), given, &format!("{name}.{field}"));
            out.insert(field, v);
        }
        Value::Object(out)
    }

    fn extras(&mut self, a: &Answers) -> (Value, Value, Value) {
        let mut conflicts = Vec::new();
        for c in &a.conflicts {
            let versions: Vec<Value> = c["versiones"]
                .as_array()
                .cloned()
                .unwrap_or_default()
                .iter()
                .filter_map(|x| {
                    let ev = self.evidence(&x["evidencia"], "conflictos")?;
                    Some(json!({ "valor_texto": x["valor_texto"].as_str().unwrap_or(""), "evidencia": ev }))
                })
                .collect();
            if versions.len() >= 2 && c["campo"].as_str().is_some_and(|s| !s.trim().is_empty()) {
                self.report.conflicts_kept += 1;
                conflicts.push(json!({ "campo": c["campo"].as_str().unwrap_or("").trim(), "versiones": versions, "nota": text_or_null(&c["nota"]) }));
            } else {
                self.report.conflicts_dropped += 1;
            }
        }
        // every block that sees the same contradiction reports it, under its own name for the field:
        // the same versions with the same quotes are one contradiction
        let mut seen: Vec<Vec<String>> = Vec::new();
        conflicts.retain(|c| {
            let mut key: Vec<String> = c["versiones"].as_array().into_iter().flatten().map(|v| norm(v["evidencia"]["cita"].as_str().unwrap_or("")).chars().take(60).collect()).collect();
            key.sort();
            let fresh = !seen.contains(&key);
            seen.push(key);
            fresh
        });
        self.report.duplicates += self.report.conflicts_kept - conflicts.len();
        self.report.conflicts_kept = conflicts.len();
        let mut findings = Vec::new();
        for f in &a.findings {
            let evidences = self.evidences(&f["evidencias"], "otros_hallazgos");
            let theme = f["tema"].as_str().unwrap_or("").trim();
            if evidences.is_empty() || theme.is_empty() {
                self.report.findings_dropped += 1;
                continue;
            }
            self.report.findings_kept += 1;
            findings.push(json!({ "tema": theme, "evidencias": evidences, "nota": text_or_null(&f["nota"]) }));
        }
        let findings = self.dedupe(findings, |f| f["evidencias"][0]["cita"].as_str().unwrap_or("").to_string());
        let mut doubts = Vec::new();
        for d in &a.doubts {
            let text = d["texto"].as_str().unwrap_or("").trim();
            if !text.is_empty() {
                let evidences = self.evidences(&d["evidencias"], "dudas");
                doubts.push(json!({ "texto": text.chars().take(400).collect::<String>(), "evidencias": evidences }));
            }
        }
        let doubts = self.dedupe(doubts, |d| d["texto"].as_str().unwrap_or("").to_string());
        (Value::Array(conflicts), Value::Array(findings), Value::Array(doubts))
    }
}

fn count(v: &Value, counts: &mut FieldCounts) {
    match v["estado"].as_str() {
        Some("encontrado") => counts.encontrado += 1,
        Some("ambiguo") => counts.ambiguo += 1,
        Some("no_aparece") => counts.no_aparece += 1,
        _ => {}
    }
    for key in ["elementos", "hitos", "criterios", "modalidades"] {
        counts.items += v[key].as_array().map_or(0, Vec::len);
    }
}

/// Every text the document quotes or copies, for scoring against facts written by hand. The model's own
/// notes are left out: only what the documents say, plus the contradictions and doubts the contract keeps.
#[cfg_attr(not(test), allow(dead_code))] // used by the measurement tests, not by the application
pub fn evidence_text(doc: &Value) -> String {
    fn walk(v: &Value, out: &mut Vec<String>) {
        match v {
            Value::Object(m) => {
                for (k, x) in m {
                    match (k.as_str(), x) {
                        ("metadatos" | "normalizado" | "nota", _) => {}
                        ("cita" | "valor_texto" | "titulo" | "etiqueta" | "aplica_a" | "inicio_texto" | "fin_texto" | "nombre" | "tema" | "texto", Value::String(s)) => out.push(s.clone()),
                        _ => walk(x, out),
                    }
                }
            }
            Value::Array(a) => a.iter().for_each(|x| walk(x, out)),
            _ => {}
        }
    }
    let mut out = Vec::new();
    walk(doc, &mut out);
    out.join("\n")
}

/// Builds the canonical document from the answers, and validates it against the full schema. A block
/// that was not answered is all `no_aparece` and is listed in `sections_missing`.
pub fn assemble(pkg: &Package, answers: &Answers, reading: &Reading) -> (Value, AssemblyReport) {
    let canon = canonical_schema();
    let mut a = Assembler::new(pkg, &canon);
    let mut doc = Map::new();
    doc.insert(
        "metadatos".into(),
        json!({
            "version_schema": SCHEMA_VERSION,
            "fuentes": pkg.documents().iter().map(|d| json!({ "documento": d.name, "paginas": d.pages })).collect::<Vec<_>>(),
            "lectura": if reading.model.is_empty() { Value::Null } else { json!({
                "modelo": reading.model,
                "fecha": reading.at,
                "tokens_entrada": reading.input_tokens,
                "tokens_salida": reading.output_tokens,
                "costo_mxn": reading.cost_mxn,
            }) },
        }),
    );
    for s in SECTIONS {
        let answer = answers.sections.get(s);
        if answer.is_none() {
            a.report.sections_missing.push(s.to_string());
        }
        let v = a.section(s, answer);
        let mut counts = FieldCounts::default();
        if let Value::Object(fields) = &v {
            for f in fields.values() {
                count(f, &mut counts);
            }
        }
        a.report.fields.encontrado += counts.encontrado;
        a.report.fields.ambiguo += counts.ambiguo;
        a.report.fields.no_aparece += counts.no_aparece;
        a.report.fields.items += counts.items;
        a.report.per_section.insert(s.to_string(), counts);
        doc.insert(s.to_string(), v);
    }
    let (conflicts, findings, doubts) = a.extras(answers);
    doc.insert("conflictos".into(), conflicts);
    doc.insert("otros_hallazgos".into(), findings);
    doc.insert("dudas".into(), doubts);
    let doc = Value::Object(doc);

    let mut report = a.report;
    report.pages_uncited = pkg.pages.iter().map(|p| p.number).filter(|n| !a.cited.contains(n)).collect();
    report.pages_cited = a.cited.iter().copied().collect();
    let (mut covered_total, mut text_total) = (0usize, 0usize);
    for p in &pkg.pages {
        let len = a.pages.get(&p.number).map_or(0, String::len);
        let mut spans = a.spans.get(&p.number).cloned().unwrap_or_default();
        spans.sort_unstable();
        let (mut covered, mut end) = (0usize, 0usize);
        for (s, e) in spans {
            let s = s.max(end);
            if e > s {
                covered += e - s;
                end = e;
            }
        }
        covered_total += covered;
        text_total += len;
        if len > 0 && (covered as f64 / len as f64) < THIN_PAGE_COVERAGE {
            report.pages_thin.push(p.number);
        }
    }
    report.text_coverage = if text_total == 0 { 1.0 } else { covered_total as f64 / text_total as f64 };
    match jsonschema::validator_for(&canon) {
        Ok(v) => report.schema_errors = v.iter_errors(&doc).map(|e| format!("{}: {}", e.instance_path(), e)).take(20).collect(),
        Err(e) => report.schema_errors = vec![format!("bad canonical schema: {e}")],
    }
    (doc, report)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::documents::canonical::package::tests::sample;

    fn ev(cita: &str, pagina: usize) -> Value {
        json!({ "cita": cita, "pagina": pagina })
    }

    fn dato(estado: &str, valor: Option<&str>, evidences: Vec<Value>) -> Value {
        json!({ "estado": estado, "valor_texto": valor, "evidencias": evidences })
    }

    fn lista(estado: &str, items: Vec<Value>) -> Value {
        json!({ "estado": estado, "elementos": items })
    }

    fn element(cita: &str, page: usize, aplica: Option<&str>) -> Value {
        json!({ "titulo": null, "evidencias": [ev(cita, page)], "aplica_a": aplica, "obligatoriedad": "obligatorio", "nota": null })
    }

    /// What a model would answer for the sample package: some right, some invented.
    fn answers() -> Answers {
        let mut a = Answers::default();
        let canon = canonical_schema();
        let strip = |mut v: Value| {
            fn go(v: &mut Value) {
                match v {
                    Value::Object(m) => {
                        m.remove("normalizado");
                        m.values_mut().for_each(go);
                    }
                    Value::Array(x) => x.iter_mut().for_each(go),
                    _ => {}
                }
            }
            go(&mut v);
            v
        };
        let mut identidad = strip(super::super::contract::blank_section(&canon, "identidad"));
        identidad["nombre"] = dato("encontrado", Some("CONVOCATORIA EJEMPLO 2027"), vec![ev("CONVOCATORIA EJEMPLO 2027", 1)]);
        let mut elegibilidad = strip(super::super::contract::blank_section(&canon, "elegibilidad"));
        elegibilidad["quienes_pueden_participar"] = lista(
            "encontrado",
            vec![
                element("Podrán participar las organizaciones registradas.", 2, None),
                // right words, wrong page: moved, not lost
                element("Podrán participar las organizaciones registradas", 3, None),
                // invented: thrown away
                element("Solo participan fundaciones con más de diez años de experiencia.", 2, None),
            ],
        );
        let mut financiamiento = strip(super::super::contract::blank_section(&canon, "financiamiento"));
        financiamiento["monto_maximo"] = dato("encontrado", Some("$250,000 pesos"), vec![ev("Monto máximo: $250,000 pesos por proyecto.", 2)]);
        // says «no_aparece» but brings a verified quote: the evidence wins
        financiamiento["porcentaje_cofinanciamiento"] = dato("no_aparece", None, vec![ev("Monto máximo: $250,000 pesos por proyecto.", 2)]);
        // says «encontrado» with a quote that is not in the package: it falls to «no_aparece»
        financiamiento["moneda"] = dato("encontrado", Some("pesos mexicanos"), vec![ev("Los montos se expresan en pesos mexicanos de curso legal.", 2)]);
        let mut temporalidad = strip(super::super::contract::blank_section(&canon, "temporalidad"));
        temporalidad["hitos"] = json!({ "estado": "encontrado", "hitos": [
            { "tipo": "cierre", "etiqueta": "Cierre de postulación", "inicio_texto": "23 de mayo de 2027",
              "fin_texto": null, "evidencias": [ev("Cierre de postulación: 23 de mayo de 2027.", 3)] }
        ]});
        a.sections.insert("identidad".into(), identidad);
        a.sections.insert("elegibilidad".into(), elegibilidad);
        a.sections.insert("financiamiento".into(), financiamiento);
        a.sections.insert("temporalidad".into(), temporalidad);
        a.conflicts.push(json!({ "campo": "financiamiento.monto_maximo", "versiones": [
            { "valor_texto": "$250,000", "evidencia": ev("Monto máximo: $250,000 pesos por proyecto.", 2) },
            { "valor_texto": "$100,000", "evidencia": ev("El monto máximo es de $100,000 pesos.", 4) }
        ], "nota": null }));
        a.findings.push(json!({ "tema": "plazo único", "evidencias": [ev("Cierre de postulación: 23 de mayo de 2027.", 3)], "nota": null }));
        a
    }

    /// A model that answers a single list of the block `financiamiento` with the given items.
    fn list_answer(items: Vec<Value>) -> Answers {
        let canon = canonical_schema();
        let mut blank = super::super::contract::blank_section(&canon, "financiamiento");
        fn go(v: &mut Value) {
            match v {
                Value::Object(m) => {
                    m.remove("normalizado");
                    m.values_mut().for_each(go);
                }
                Value::Array(x) => x.iter_mut().for_each(go),
                _ => {}
            }
        }
        go(&mut blank);
        blank["conceptos_financiables"] = lista("encontrado", items);
        let mut a = Answers::default();
        a.sections.insert("financiamiento".into(), blank);
        a
    }

    #[test]
    fn a_short_item_of_a_list_or_a_table_counts_when_it_is_a_whole_line_or_a_whole_cell_and_not_a_word_inside_a_sentence() {
        let pkg = Package::from_documents(
            "rubros",
            vec![("anexo.pdf".into(), vec!["Rubros que se pagan\n\nSoftware\nAmbulancias | Equipo especializado\nSeguros | Becas\n\nNo se paga el software libre de otras campañas.\n\nSi | No".into()])],
        );
        let items = vec![
            element("Software", 1, None),       // a whole line
            element("Ambulancias", 1, None),    // a whole cell
            element("Seguros", 1, None),        // a whole cell
            element("Becas", 1, None),          // a whole cell
            element("libre", 1, None),          // a word inside a sentence: proves nothing
            element("Si", 1, None),             // too short even as a cell
            element("Joyas", 1, None),          // not in the page
        ];
        let (doc, r) = assemble(&pkg, &list_answer(items), &Reading::default());
        let kept: Vec<String> = doc["financiamiento"]["conceptos_financiables"]["elementos"].as_array().unwrap().iter().map(|e| e["evidencias"][0]["cita"].as_str().unwrap().to_string()).collect();
        assert_eq!(kept, vec!["Software", "Ambulancias", "Seguros", "Becas"]);
        assert_eq!((r.citations_verified, r.citations_rejected), (4, 3));
    }

    #[test]
    fn a_quote_with_the_words_joined_that_the_page_cuts_with_a_hyphen_is_the_pages_own() {
        let pkg = Package::from_documents(
            "categorias",
            vec![("bases.pdf".into(), vec!["Programas que buscan ga-\nrantizar el acceso a derechos básicos de la población en si-\ntuación de vulnerabilidad.\n\nSe atiende a personas con estudios socio-\neconómicos vigentes.".into()])],
        );
        let items = vec![
            // the model writes the words whole
            element("Programas que buscan garantizar el acceso a derechos básicos de la población en situación de vulnerabilidad.", 1, None),
            // the page's own hyphen is kept as the page writes it
            element("personas con estudios socio-económicos vigentes", 1, None),
            // joining words that the page does not join is still invented
            element("Programas que buscan garantizar el acceso a derechos básicos de las comunidades indígenas", 1, None),
        ];
        let (doc, r) = assemble(&pkg, &list_answer(items), &Reading::default());
        assert_eq!(doc["financiamiento"]["conceptos_financiables"]["elementos"].as_array().unwrap().len(), 2, "{:?}", r.rejected);
        assert_eq!((r.citations_verified, r.citations_rejected), (2, 1));
        // the quote the person sees is the one the model wrote; what counts as covered is where it sits in the page
        assert!(r.text_coverage > 0.5, "{}", r.text_coverage);
    }

    #[test]
    fn the_report_says_which_pages_carry_a_verified_quote() {
        let pkg = sample();
        let (_, r) = assemble(&pkg, &answers(), &Reading::default());
        assert_eq!(r.pages_cited, vec![1, 2, 3]);
        assert_eq!(r.pages_uncited, vec![4]);
        assert!((r.page_coverage() - 0.75).abs() < 1e-9);
        // a page with one short quote out of a lot of text is cited but still thin; the quoted line alone is not
        assert!(r.pages_thin.contains(&4) && r.pages_thin.iter().all(|p| *p <= 4));
        assert!(r.text_coverage > 0.0 && r.text_coverage < 1.0, "{}", r.text_coverage);
        let (_, none) = assemble(&pkg, &Answers::default(), &Reading::default());
        assert_eq!((none.pages_cited.len(), none.pages_uncited), (0, vec![1, 2, 3, 4]));
    }

    #[test]
    fn a_later_reading_grows_the_lists_and_fills_only_the_empty_single_values() {
        let mut a = answers();
        let mut later = Answers::default();
        let canon = canonical_schema();
        let mut elegibilidad = super::super::contract::blank_section(&canon, "elegibilidad");
        elegibilidad["quienes_pueden_participar"] = lista("encontrado", vec![element("Podrán participar las organizaciones registradas.", 2, Some("las registradas"))]);
        elegibilidad["quienes_no_pueden"] = lista("encontrado", vec![element("Solo participan las organizaciones registradas.", 2, None)]);
        let mut financiamiento = super::super::contract::blank_section(&canon, "financiamiento");
        financiamiento["monto_maximo"] = dato("encontrado", Some("$999"), vec![ev("Monto máximo: $250,000 pesos por proyecto.", 2)]);
        financiamiento["tope_gastos_administrativos"] = dato("encontrado", Some("10%"), vec![ev("Monto máximo: $250,000 pesos por proyecto.", 2)]);
        later.sections.insert("elegibilidad".into(), elegibilidad);
        later.sections.insert("financiamiento".into(), financiamiento);
        let before = a.sections["elegibilidad"]["quienes_pueden_participar"]["elementos"].as_array().unwrap().len();
        let later_value = serde_json::to_value(later.sections.clone()).unwrap();
        a.merge(&json!({ "elegibilidad": later_value["elegibilidad"], "financiamiento": later_value["financiamiento"], "dudas": [] }));
        assert_eq!(a.sections["elegibilidad"]["quienes_pueden_participar"]["elementos"].as_array().unwrap().len(), before + 1);
        // an empty field of the first reading takes what the second found...
        assert_eq!(a.sections["elegibilidad"]["quienes_no_pueden"]["estado"], "encontrado");
        assert_eq!(a.sections["financiamiento"]["tope_gastos_administrativos"]["valor_texto"], "10%");
        // ...but a value the first reading already had is not replaced
        assert_eq!(a.sections["financiamiento"]["monto_maximo"]["valor_texto"], "$250,000 pesos");
        // a block the first reading lacks is taken whole
        let mut empty = Answers::default();
        empty.merge(&json!({ "financiamiento": later_value["financiamiento"] }));
        assert!(empty.sections.contains_key("financiamiento"));
    }

    #[test]
    fn what_the_model_claims_is_checked_and_what_is_read_comes_from_the_quote() {
        let pkg = sample();
        let (doc, r) = assemble(&pkg, &answers(), &Reading::default());
        assert!(r.schema_errors.is_empty(), "{:?}", r.schema_errors);

        // a right quote enters with its document, and the value is read, not copied from the model
        let max = &doc["financiamiento"]["monto_maximo"];
        assert_eq!(max["estado"], "encontrado");
        assert_eq!(max["evidencias"][0]["documento"], "bases.pdf");
        assert_eq!(max["normalizado"], json!({ "cantidad": 250000.0, "moneda": "MXN" }));

        // the quote on the wrong page is moved; the invented one is thrown away
        let ok = doc["elegibilidad"]["quienes_pueden_participar"]["elementos"].as_array().unwrap();
        assert_eq!(ok.len(), 1, "the relocated quote repeats the first one, the invented one is gone: {ok:?}");
        assert_eq!(ok[0]["evidencias"][0]["pagina"], 2);
        assert_eq!((r.citations_relocated, r.items_dropped, r.duplicates), (1, 1, 1), "{r:?}");

        // evidence beats declaration, in both directions
        assert_eq!(doc["financiamiento"]["porcentaje_cofinanciamiento"]["estado"], "encontrado");
        assert_eq!(doc["financiamiento"]["moneda"]["estado"], "no_aparece");
        assert_eq!(doc["financiamiento"]["moneda"]["valor_texto"], Value::Null);
        assert_eq!(r.states_fixed, 2, "{r:?}");

        // a milestone is read: the year is the quote's own, so it is not inferred
        let h = &doc["temporalidad"]["hitos"]["hitos"][0];
        assert_eq!(h["normalizado"]["inicio"]["fecha"], "2027-05-23");
        assert_eq!(h["normalizado"]["inicio"]["anio_inferido"], false);

        // a contradiction with a quote that is not in the package loses that version, so it is no contradiction
        assert_eq!((r.conflicts_kept, r.conflicts_dropped), (0, 1));
        assert!(doc["conflictos"].as_array().unwrap().is_empty());
        assert_eq!(doc["otros_hallazgos"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn blocks_not_answered_are_blank_and_listed_and_the_document_is_still_valid() {
        let (doc, r) = assemble(&sample(), &answers(), &Reading::default());
        assert_eq!(r.sections_missing, vec!["proyecto", "documentacion", "evaluacion", "entrega"]);
        assert_eq!(doc["entrega"]["contacto"], json!({ "estado": "no_aparece", "elementos": [] }));
        assert!(r.schema_errors.is_empty(), "{:?}", r.schema_errors);
        assert_eq!(r.fields.encontrado + r.fields.ambiguo + r.fields.no_aparece, r.per_section.values().map(|c| c.encontrado + c.ambiguo + c.no_aparece).sum::<usize>());
        assert!(r.verified_rate() > 0.5 && r.verified_rate() < 1.0);
        let (empty, r0) = assemble(&sample(), &Answers::default(), &Reading::default());
        assert!(r0.schema_errors.is_empty() && r0.fields.encontrado == 0 && r0.verified_rate() == 1.0);
        assert_eq!(empty["metadatos"]["fuentes"][0], json!({ "documento": "bases.pdf", "paginas": 3 }));
    }

    #[test]
    fn a_contradiction_with_both_versions_in_the_package_is_kept_and_not_resolved() {
        let pkg = Package::from_documents(
            "c",
            vec![
                ("bases.pdf".into(), vec!["La contrapartida mínima exigida es del 30% del costo total del proyecto.".into()]),
                ("formulario.docx".into(), vec!["La contrapartida mínima exigida es del 20% del costo del proyecto.".into()]),
            ],
        );
        let mut a = Answers::default();
        a.add(&json!({ "conflictos": [{ "campo": "financiamiento.porcentaje_cofinanciamiento", "versiones": [
            { "valor_texto": "30%", "evidencia": ev("La contrapartida mínima exigida es del 30% del costo total del proyecto.", 1) },
            { "valor_texto": "20%", "evidencia": ev("La contrapartida mínima exigida es del 20% del costo del proyecto.", 2) } ], "nota": null }],
            "otros_hallazgos": [], "dudas": [{ "texto": "Dos documentos dicen cosas distintas.", "evidencias": [] }] }));
        let (doc, r) = assemble(&pkg, &a, &Reading::default());
        assert_eq!(r.conflicts_kept, 1);
        assert_eq!(doc["conflictos"][0]["versiones"][1]["evidencia"]["documento"], "formulario.docx");
        assert_eq!(doc["dudas"].as_array().unwrap().len(), 1);
        assert!(r.schema_errors.is_empty(), "{:?}", r.schema_errors);
    }

    #[test]
    fn the_same_contradiction_reported_by_two_blocks_is_one() {
        let pkg = Package::from_documents(
            "c",
            vec![("bases.pdf".into(), vec!["La contrapartida mínima exigida es del 30% del costo total del proyecto.".into(), "La contrapartida mínima exigida es del 20% del costo del proyecto.".into()])],
        );
        let conflict = |campo: &str| json!({ "campo": campo, "versiones": [
            { "valor_texto": "30%", "evidencia": ev("La contrapartida mínima exigida es del 30% del costo total del proyecto.", 1) },
            { "valor_texto": "20%", "evidencia": ev("La contrapartida mínima exigida es del 20% del costo del proyecto.", 2) } ], "nota": null });
        let mut a = Answers::default();
        a.add(&json!({ "conflictos": [conflict("financiamiento.porcentaje_cofinanciamiento")], "otros_hallazgos": [], "dudas": [] }));
        a.add(&json!({ "conflictos": [conflict("proyecto.requisitos_del_proyecto")], "otros_hallazgos": [], "dudas": [] }));
        let (doc, r) = assemble(&pkg, &a, &Reading::default());
        assert_eq!(doc["conflictos"].as_array().unwrap().len(), 1);
        assert_eq!((r.conflicts_kept, r.duplicates), (1, 1));
    }

    #[test]
    fn the_same_answers_always_give_the_same_document() {
        let (a, ra) = assemble(&sample(), &answers(), &Reading::default());
        let (b, rb) = assemble(&sample(), &answers(), &Reading::default());
        assert_eq!(a, b);
        assert_eq!(ra, rb);
    }

    #[test]
    fn the_text_for_scoring_has_what_the_documents_say_and_not_the_models_notes() {
        let (mut doc, _) = assemble(&sample(), &answers(), &Reading::default());
        doc["identidad"]["que_apoya"] = json!({ "estado": "encontrado", "elementos": [{ "titulo": "T", "evidencias": [{ "cita": "una cita", "pagina": 1, "documento": "d" }], "aplica_a": null, "obligatoriedad": "informativo", "nota": "OPINIÓN DEL MODELO" }] });
        let t = evidence_text(&doc);
        assert!(t.contains("$250,000 pesos") && t.contains("una cita") && !t.contains("OPINIÓN DEL MODELO") && !t.contains("version_schema"));
    }
}
