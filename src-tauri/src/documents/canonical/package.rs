//! A call is a package of documents, not a PDF (ADR-014): bases, rules, forms, annexes that refer to one
//! another. This turns the files into numbered pages (one numbering for the whole package) and a page
//! map, and makes no assumption about how a call is written.

use crate::documents::text::{clean_text, norm};
use regex::Regex;
use serde::Serialize;
use std::collections::{BTreeMap, HashMap};
use std::path::Path;

/// A Word or Excel file has no pages; it is cut into pieces of about this many characters, between lines.
const PIECE_CHARS: usize = 3_500;

#[derive(Debug, Clone, PartialEq)]
pub struct PackPage {
    /// Number in the package, from 1: the one the model cites.
    pub number: usize,
    pub document: String,
    /// Number inside its document, from 1.
    pub local: usize,
    /// Cleaned text (the model reads it and the quotes are checked against it).
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DocInfo {
    pub name: String,
    pub pages: usize,
    pub chars: usize,
}

/// One line of the page map: where a page is and what it opens with.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PageLabel {
    pub id: String,
    pub page: usize,
    pub document: String,
    pub label: String,
    pub chars: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Package {
    pub name: String,
    pub pages: Vec<PackPage>,
}

impl Package {
    /// Builds a package from texts already read: `(document name, pages of that document)`.
    pub fn from_documents(name: &str, docs: Vec<(String, Vec<String>)>) -> Package {
        let mut pages = Vec::new();
        for (document, texts) in docs {
            for (i, raw) in texts.iter().enumerate() {
                pages.push(PackPage { number: pages.len() + 1, document: document.clone(), local: i + 1, text: clean_text(raw).trim().to_string() });
            }
        }
        Package { name: name.to_string(), pages }
    }

    /// Reads PDF, Word and Excel files, in this order. A file that cannot be read is an error: a package
    /// with a silently missing document would give a wrong picture of the call.
    #[cfg_attr(not(test), allow(dead_code))] // used by the measurement tests, not by the application
    pub fn from_paths(name: &str, paths: &[String]) -> Result<Package, String> {
        let mut docs = Vec::new();
        for p in paths {
            let path = Path::new(p);
            let file = path.file_name().map(|f| f.to_string_lossy().to_string()).unwrap_or_else(|| p.clone());
            docs.push((file.clone(), read_pieces(path).map_err(|e| format!("{file}: {e}"))?));
        }
        Ok(Package::from_documents(name, docs))
    }

    pub fn total_chars(&self) -> usize {
        self.pages.iter().map(|p| p.text.chars().count()).sum()
    }

    pub fn page(&self, number: usize) -> Option<&PackPage> {
        number.checked_sub(1).and_then(|i| self.pages.get(i))
    }

    pub fn documents(&self) -> Vec<DocInfo> {
        let mut out: Vec<DocInfo> = Vec::new();
        for p in &self.pages {
            match out.last_mut() {
                Some(d) if d.name == p.document => {
                    d.pages += 1;
                    d.chars += p.text.chars().count();
                }
                _ => out.push(DocInfo { name: p.document.clone(), pages: 1, chars: p.text.chars().count() }),
            }
        }
        out
    }

    /// The pages as the model reads them: each one under `[Página N | archivo]`, in package order.
    pub fn render(&self, numbers: &[usize]) -> String {
        let mut sorted: Vec<usize> = numbers.to_vec();
        sorted.sort_unstable();
        sorted.dedup();
        sorted
            .iter()
            .filter_map(|n| self.page(*n))
            .map(|p| format!("[Página {} | {}]\n{}", p.number, p.document, p.text))
            .collect::<Vec<_>>()
            .join("\n\n")
    }

    pub fn all_numbers(&self) -> Vec<usize> {
        self.pages.iter().map(|p| p.number).collect()
    }

    /// The year that appears most in the package (4 digits, 2000 to 2099): for a date written without one.
    pub fn dominant_year(&self) -> Option<u32> {
        let re = Regex::new(r"\b(20\d\d)\b").expect("valid regex");
        let mut count: BTreeMap<u32, usize> = BTreeMap::new();
        for p in &self.pages {
            for m in re.find_iter(&p.text) {
                if let Ok(y) = m.as_str().parse::<u32>() {
                    *count.entry(y).or_default() += 1;
                }
            }
        }
        count.into_iter().max_by_key(|(y, c)| (*c, *y)).map(|(y, _)| y)
    }

    /// True if any page says that amounts are in pesos: what lets a bare «$» be read as MXN.
    pub fn mentions_pesos(&self) -> bool {
        let re = Regex::new(r"(?i)\bpesos\b|M\.\s?N\.|\bMXN\b").expect("valid regex");
        self.pages.iter().any(|p| re.is_match(&p.text))
    }

    /// Page map: for every page, what it opens with. A line that repeats on many pages of a document
    /// (a running header or footer) says nothing about the page and is skipped, whatever it says.
    pub fn page_map(&self) -> Vec<PageLabel> {
        let mut seen: HashMap<(String, String), usize> = HashMap::new();
        let mut per_doc: HashMap<String, usize> = HashMap::new();
        for p in &self.pages {
            *per_doc.entry(p.document.clone()).or_default() += 1;
            let mut lines: Vec<String> = p.text.lines().map(shape).filter(|l| !l.is_empty()).collect();
            lines.sort();
            lines.dedup();
            for l in lines {
                *seen.entry((p.document.clone(), l)).or_default() += 1;
            }
        }
        self.pages
            .iter()
            .map(|p| {
                let docs_pages = per_doc[&p.document];
                let label = p
                    .text
                    .lines()
                    .map(str::trim)
                    .filter(|l| l.chars().filter(|c| c.is_alphabetic()).count() >= 3)
                    .find(|l| {
                        let repeats = seen.get(&(p.document.clone(), shape(l))).copied().unwrap_or(0);
                        !(docs_pages >= 4 && repeats * 10 >= docs_pages * 3)
                    })
                    .or_else(|| p.text.lines().map(str::trim).find(|l| l.chars().filter(|c| c.is_alphabetic()).count() >= 3))
                    .unwrap_or("")
                    .chars()
                    .take(80)
                    .collect::<String>();
                PageLabel { id: format!("P{:03}", p.number), page: p.number, document: p.document.clone(), label, chars: p.text.chars().count() }
            })
            .collect()
    }

    /// The page map as text, to tell the model which pages exist even when it is not given all of them.
    pub fn render_map(&self) -> String {
        self.page_map().iter().map(|l| format!("{} | {} | {}", l.id, l.document, l.label)).collect::<Vec<_>>().join("\n")
    }
}

/// The pages of one file: a PDF page by page (with the rows of its tables), a Word or Excel file in pieces
/// of about the same size. The kind is told by the extension.
pub fn read_pieces(path: &Path) -> Result<Vec<String>, String> {
    match path.extension().map(|e| e.to_string_lossy().to_lowercase()).as_deref() {
        Some("pdf") => crate::documents::pdf::read_pdf_with_rows(path).map(|t| t.pages).map_err(|e| e.to_string()),
        Some("docx") => docx_pieces(path),
        Some("xlsx") | Some("xlsm") => xlsx_pieces(path),
        other => Err(format!("unsupported file type {other:?}")),
    }
}

/// A line without its digits: «legislacion.edomex.gob.mx 7» and «… 8» are the same line of a running header.
fn shape(line: &str) -> String {
    norm(line).chars().map(|c| if c.is_ascii_digit() { '#' } else { c }).collect()
}

fn pieces_of(text: &str) -> Vec<String> {
    let mut pieces: Vec<String> = Vec::new();
    let mut current = String::new();
    for line in text.lines().map(str::trim).filter(|l| !l.is_empty()) {
        if current.chars().count() + line.chars().count() > PIECE_CHARS && !current.is_empty() {
            pieces.push(std::mem::take(&mut current));
        }
        current.push_str(line);
        current.push('\n');
    }
    if !current.trim().is_empty() {
        pieces.push(current);
    }
    pieces
}

fn docx_pieces(path: &Path) -> Result<Vec<String>, String> {
    use std::io::Read;
    let file = std::fs::File::open(path).map_err(|e| e.to_string())?;
    let mut zip = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;
    let mut xml = String::new();
    zip.by_name("word/document.xml").map_err(|e| e.to_string())?.read_to_string(&mut xml).map_err(|e| e.to_string())?;
    let xml = Regex::new(r"</w:p>").unwrap().replace_all(&xml, "\n").to_string();
    let xml = Regex::new(r"<w:tab/>").unwrap().replace_all(&xml, " ").to_string();
    let text = Regex::new(r"<[^>]+>").unwrap().replace_all(&xml, "").to_string();
    let text = text.replace("&amp;", "&").replace("&lt;", "<").replace("&gt;", ">").replace("&quot;", "\"").replace("&apos;", "'");
    Ok(pieces_of(&text))
}

fn xlsx_pieces(path: &Path) -> Result<Vec<String>, String> {
    use calamine::{open_workbook_auto, Reader};
    let mut wb = open_workbook_auto(path).map_err(|e| e.to_string())?;
    let mut pieces = Vec::new();
    for name in wb.sheet_names().to_vec() {
        let range = wb.worksheet_range(&name).map_err(|e| e.to_string())?;
        let mut text = format!("Hoja: {name}\n");
        for row in range.rows() {
            let cells: Vec<String> = row.iter().map(|c| c.to_string().trim().to_string()).filter(|c| !c.is_empty()).collect();
            if !cells.is_empty() {
                text.push_str(&cells.join(" | "));
                text.push('\n');
            }
        }
        pieces.extend(pieces_of(&text));
    }
    Ok(pieces)
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    pub fn sample() -> Package {
        Package::from_documents(
            "ejemplo",
            vec![
                (
                    "bases.pdf".into(),
                    vec![
                        "CONVOCATORIA EJEMPLO 2027\nFundación Ficticia\n\nPresentación del programa.".into(),
                        "Podrán participar las organizaciones registradas.\n\nMonto máximo: $250,000 pesos por proyecto.".into(),
                        "Calendario\nCierre de postulación: 23 de mayo de 2027.".into(),
                    ],
                ),
                ("formato.docx".into(), vec!["Solicitud de apoyo\nNombre de la organización".into()]),
            ],
        )
    }

    #[test]
    fn pages_are_numbered_across_the_whole_package_and_render_with_their_document() {
        let p = sample();
        assert_eq!(p.pages.len(), 4);
        assert_eq!((p.pages[3].number, p.pages[3].local, p.pages[3].document.as_str()), (4, 1, "formato.docx"));
        let r = p.render(&[3, 1, 3]);
        assert!(r.starts_with("[Página 1 | bases.pdf]") && r.contains("[Página 3 | bases.pdf]") && !r.contains("[Página 2"), "{r}");
        assert_eq!(p.documents().iter().map(|d| (d.name.as_str(), d.pages)).collect::<Vec<_>>(), vec![("bases.pdf", 3), ("formato.docx", 1)]);
        assert_eq!(p.page(0), None);
        assert_eq!(p.page(5), None);
    }

    #[test]
    fn the_year_and_the_currency_come_from_the_text() {
        let p = sample();
        assert_eq!(p.dominant_year(), Some(2027));
        assert!(p.mentions_pesos());
        assert_eq!(Package::from_documents("x", vec![("a.pdf".into(), vec!["sin años".into()])]).dominant_year(), None);
    }

    #[test]
    fn the_page_map_skips_what_repeats_on_every_page() {
        let header = "FUNDACIÓN FICTICIA - CONVOCATORIA";
        let pages: Vec<String> = ["Objetivo del programa", "Requisitos de participación", "Calendario", "Anexos", "Firmas"]
            .iter()
            .enumerate()
            .map(|(i, t)| format!("{header}\n{t}\nTexto de la página {i}."))
            .collect();
        let p = Package::from_documents("m", vec![("bases.pdf".into(), pages)]);
        let map = p.page_map();
        assert_eq!(map.iter().map(|l| l.label.as_str()).collect::<Vec<_>>(), vec!["Objetivo del programa", "Requisitos de participación", "Calendario", "Anexos", "Firmas"]);
        assert_eq!(map[0].id, "P001");
        assert!(p.render_map().contains("P002 | bases.pdf | Requisitos de participación"));
        // a short document keeps its first line, even if it repeats
        let short = Package::from_documents("m", vec![("a.pdf".into(), vec!["Título\nuno".into(), "Título\ndos".into()])]);
        assert_eq!(short.page_map()[0].label, "Título");
    }

    #[test]
    fn a_running_header_with_the_page_number_is_still_a_running_header() {
        let pages: Vec<String> = ["Objetivo", "Requisitos", "Calendario", "Anexos"].iter().enumerate().map(|(i, t)| format!("sitio.gob.mx {}
{t}
Texto.", i + 2)).collect();
        let p = Package::from_documents("m", vec![("r.pdf".into(), pages)]);
        assert_eq!(p.page_map().iter().map(|l| l.label.as_str()).collect::<Vec<_>>(), vec!["Objetivo", "Requisitos", "Calendario", "Anexos"]);
    }

    #[test]
    fn long_text_without_pages_is_cut_between_lines() {
        let text = (0..400).map(|i| format!("Renglón número {i} con algo de texto para llenar")).collect::<Vec<_>>().join("\n");
        let pieces = pieces_of(&text);
        assert!(pieces.len() >= 5 && pieces.iter().all(|p| p.chars().count() <= PIECE_CHARS + 100));
        assert_eq!(pieces.join("\n").lines().filter(|l| !l.is_empty()).count(), 400);
    }
}
