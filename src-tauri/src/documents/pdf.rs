//! PDF: the text of each page, by Pdfium (ADR-037), the engine that also draws the scanned pages to read them as
//! pictures (ADR-035). A page with no text is reported. It replaced `pdf-extract`, which put spaces inside words with
//! some fonts («Thi s report outli nes»).

use pdfium_render::prelude::*;
use std::path::Path;

/// A page with fewer visible characters than this is treated as having no text layer.
const MIN_CHARS_PER_PAGE: usize = 20;
/// Two lines farther apart than this many times their height (the shorter one's) are two paragraphs: inside a
/// paragraph the space between two lines is about a fifth of their height. Pdfium does not mark paragraphs; the search cuts its pieces
/// at them (`chunking`).
const PARAGRAPH_GAP: f32 = 0.5;

#[derive(Debug, thiserror::Error)]
#[allow(dead_code)]
pub enum PdfError {
    #[error("the PDF could not be read: {0}")]
    Unreadable(String),
    /// It asks for a password to be opened: the person is told, never left with a silent failure.
    #[error("the PDF has a password")]
    Protected,
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct PdfText {
    pub pages: Vec<String>,
    /// 1-based numbers of pages with no usable text (probably scanned images).
    pub pages_without_text: Vec<usize>,
}

#[allow(dead_code)]
impl PdfText {
    /// True when no page has text: the whole file is a scan.
    pub fn is_fully_scanned(&self) -> bool {
        !self.pages.is_empty() && self.pages_without_text.len() == self.pages.len()
    }

    pub fn text(&self) -> String {
        self.pages.join("\n\n")
    }
}

pub fn read_pdf(path: &Path) -> Result<PdfText, PdfError> {
    let pdfium = super::ocr::pdfium().ok_or_else(|| PdfError::Unreadable("what reads PDF is not installed".into()))?;
    let doc = pdfium.load_pdf_from_file(path, None).map_err(|e| match e {
        PdfiumError::PdfiumLibraryInternalError(PdfiumInternalError::PasswordError) => PdfError::Protected,
        e => PdfError::Unreadable(format!("{e:?}")),
    })?;
    let pages = doc
        .pages()
        .iter()
        .map(|page| page.text().map(|t| page_text(&t)).map_err(|e| PdfError::Unreadable(format!("{e:?}"))))
        .collect::<Result<Vec<String>, PdfError>>()?;
    Ok(with_empty_pages(pages))
}

/// The pages and which of them have no text to speak of.
fn with_empty_pages(pages: Vec<String>) -> PdfText {
    let pages_without_text = pages
        .iter()
        .enumerate()
        .filter(|(_, t)| t.chars().filter(|c| !c.is_whitespace()).count() < MIN_CHARS_PER_PAGE)
        .map(|(i, _)| i + 1)
        .collect();
    PdfText { pages, pages_without_text }
}

/// A line of a page as Pdfium gives it: its text and where its letters sit (points, `y` growing upward). A line made
/// only of what Pdfium added (spaces, breaks) has no place.
struct Line {
    text: String,
    top: f32,
    bottom: f32,
}

impl Line {
    fn new() -> Line {
        Line { text: String::new(), top: f32::MIN, bottom: f32::MAX }
    }

    fn placed(&self) -> bool {
        self.top >= self.bottom
    }
}

/// The text of a page, line by line, with an empty line between paragraphs.
fn page_text(text: &PdfPageText) -> String {
    let mut lines = vec![];
    let mut line = Line::new();
    for c in text.chars().iter() {
        let Some(ch) = c.unicode_char() else { continue };
        match ch {
            '\r' => {}
            '\n' => lines.push(std::mem::replace(&mut line, Line::new())),
            // Pdfium marks so the hyphen that cuts a word at the end of a line
            '\u{2}' => line.text.push('-'),
            _ => {
                line.text.push(ch);
                if !ch.is_whitespace() && !c.is_generated().unwrap_or(false) {
                    if let Ok(b) = c.loose_bounds() {
                        line.top = line.top.max(b.top().value);
                        line.bottom = line.bottom.min(b.bottom().value);
                    }
                }
            }
        }
    }
    lines.push(line);
    join_lines(&lines)
}

/// Lines one under the other; an empty line where they stand far apart (a new paragraph) or where the next one goes
/// up the page (another column).
fn join_lines(lines: &[Line]) -> String {
    let mut out = String::new();
    let mut before: Option<&Line> = None;
    for line in lines.iter().filter(|l| !l.text.trim().is_empty()) {
        if let Some(b) = before {
            out.push_str(if apart(b, line) { "\n\n" } else { "\n" });
        }
        out.push_str(line.text.trim());
        before = Some(line);
    }
    out
}

fn apart(above: &Line, next: &Line) -> bool {
    if !above.placed() || !next.placed() {
        return false;
    }
    // the shorter of the two: under a big title the gap is small next to the title, but not next to the text
    let tall = (above.top - above.bottom).min(next.top - next.bottom);
    above.bottom - next.top > PARAGRAPH_GAP * tall || next.bottom > above.top
}

/// The text by `pdf-extract`, the engine before Pdfium: only to measure the two side by side (ADR-037) until the
/// tables move to Pdfium too.
#[cfg(test)]
fn read_with_pdf_extract(path: &Path) -> Result<PdfText, PdfError> {
    let path = path.to_path_buf();
    let pages = std::panic::catch_unwind(move || pdf_extract::extract_text_by_pages(&path))
        .map_err(|_| PdfError::Unreadable("unsupported or damaged file".into()))?
        .map_err(|e| PdfError::Unreadable(e.to_string()))?;
    Ok(with_empty_pages(pages))
}

/// Like `read_pdf`, and each page also carries the rows of its tables rebuilt from where the letters sit
/// (`pdf_rows`), appended as a block of their own. The plain text is never changed; a page the second
/// pass cannot read keeps just its text.
pub fn read_pdf_with_rows(path: &Path) -> Result<PdfText, PdfError> {
    let mut text = read_pdf(path)?;
    let rows = super::pdf_rows::rows_by_page(path);
    for (page, rows) in text.pages.iter_mut().zip(&rows) {
        *page = super::pdf_rows::with_rows(page, rows);
    }
    Ok(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture(name: &str) -> std::path::PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../fixtures/office").join(name)
    }

    #[test]
    fn extracts_text_and_table_cells() {
        let r = read_pdf(&fixture("convocatoria-con-tablas.pdf")).unwrap();
        println!("{:#?}", r.pages);
        assert_eq!(r.pages.len(), 2);
        assert!(!r.is_fully_scanned());
        assert!(r.pages_without_text.is_empty());
        let t = r.text();
        assert!(t.contains("Convocatoria ficticia 2026"));
        assert!(t.contains("REQ-01"));
        // table cells come out as text with their values
        assert!(t.contains("Obra y equipamiento"));
        assert!(t.contains("$300,000"));
        assert!(t.contains("Capacitación"));
        assert!(t.contains("acta constitutiva"));
    }

    #[test]
    fn rows_are_added_to_the_text_and_never_take_anything_away() {
        let plain = read_pdf(&fixture("convocatoria-con-tablas.pdf")).unwrap();
        let rich = read_pdf_with_rows(&fixture("convocatoria-con-tablas.pdf")).unwrap();
        assert_eq!(plain.pages.len(), rich.pages.len());
        assert_eq!(plain.pages_without_text, rich.pages_without_text);
        for (p, r) in plain.pages.iter().zip(&rich.pages) {
            assert!(r.starts_with(p.as_str()), "the plain text of the page is kept as it was");
        }
        println!("{:#?}", rich.pages);
    }

    #[test]
    fn scanned_pdf_is_detected() {
        let r = read_pdf(&fixture("convocatoria-escaneada.pdf")).unwrap();
        assert!(r.is_fully_scanned());
        assert_eq!(r.pages_without_text, vec![1]);
    }

    /// What the two engines take out of each PDF of a folder, side by side: `pdf-extract` (today) and Pdfium (the one that
    /// already draws the scanned pages). Only measures, never prints a text unless asked; for public samples only.
    ///   $env:CIMIENTO_PDF_DIR="D:\...\pdf_pruebas"; cargo test compare_pdf_engines -- --ignored --nocapture > salida.txt
    ///   (optional: $env:CIMIENTO_PDF_SHOW="archivo.pdf:3", the text of that page by both engines)
    #[test]
    #[ignore]
    fn compare_pdf_engines() {
        use std::collections::HashMap;
        use std::time::Instant;

        let dir = std::path::PathBuf::from(std::env::var("CIMIENTO_PDF_DIR").expect("CIMIENTO_PDF_DIR"));
        let show = std::env::var("CIMIENTO_PDF_SHOW").ok();
        assert!(crate::documents::ocr::pdfium().is_some(), "pdfium: run scripts/fetch-ocr.ps1");
        let mut files: Vec<_> = std::fs::read_dir(&dir).unwrap().filter_map(|e| e.ok()).map(|e| e.path()).filter(|p| p.extension().is_some_and(|e| e.eq_ignore_ascii_case("pdf"))).collect();
        files.sort();

        let words = |pages: &[String]| -> Vec<String> { pages.iter().flat_map(|p| p.split_whitespace().map(str::to_string)).collect::<Vec<_>>() };
        // what is measured of a reading: letters, words, accents, odd signs, glued and split words, pages with no text
        let measure = |pages: &[String]| {
            let all: String = pages.join("\n");
            let ws = words(pages);
            let letters = all.chars().filter(|c| !c.is_whitespace()).count();
            let accents = all.chars().filter(|c| "áéíóúñüÁÉÍÓÚÑÜ".contains(*c)).count();
            let odd = all.chars().filter(|&c| c == '\u{FFFD}' || (c.is_control() && c != '\n' && c != '\t') || ('\u{E000}'..='\u{F8FF}').contains(&c)).count();
            let glued = ws.iter().filter(|w| w.chars().filter(|c| c.is_alphabetic()).count() > 25).count();
            let singles = ws.iter().filter(|w| w.chars().count() == 1 && w.chars().all(char::is_alphabetic)).count();
            let empty = pages.iter().filter(|p| p.chars().filter(|c| !c.is_whitespace()).count() < MIN_CHARS_PER_PAGE).count();
            let paragraphs: usize = pages.iter().map(|p| p.split("\n\n").filter(|b| !b.trim().is_empty()).count()).sum();
            (letters, ws.len(), accents, odd, glued, singles, empty, paragraphs)
        };
        // the share of the words of `a` found in `b`, each word counted as many times as it appears
        let shared = |a: &[String], b: &[String]| -> f64 {
            let mut bag: HashMap<&str, usize> = HashMap::new();
            for w in b {
                *bag.entry(w.as_str()).or_default() += 1;
            }
            let found = a.iter().filter(|w| bag.get_mut(w.as_str()).filter(|n| **n > 0).map(|n| *n -= 1).is_some()).count();
            if a.is_empty() { 1.0 } else { found as f64 / a.len() as f64 }
        };

        println!("{:<36} {:>5} | {:>7} {:>7} {:>6} {:>5} {:>5} {:>5} {:>5} {:>4} {:>5} | {:>7} {:>7} {:>6} {:>5} {:>5} {:>5} {:>5} {:>4} {:>5} | {:>5} {:>5}", "archivo", "pág", "ms", "letras", "palab", "acent", "raros", "pegad", "1letr", "vac", "párr", "ms", "letras", "palab", "acent", "raros", "pegad", "1letr", "vac", "párr", "E→P", "P→E");
        println!("{:<36} {:>5} | {:^64} | {:^64} |", "", "", "pdf-extract", "Pdfium");
        for path in &files {
            let name: String = path.file_name().unwrap().to_string_lossy().chars().take(36).collect();
            let t = Instant::now();
            let extract = read_with_pdf_extract(path).map(|r| r.pages).map_err(|e| e.to_string());
            let te = t.elapsed().as_millis();
            let t = Instant::now();
            let pdfium_pages = read_pdf(path).map(|r| r.pages).map_err(|e| e.to_string());
            let tp = t.elapsed().as_millis();
            let cols = |r: &Result<Vec<String>, String>, ms: u128| match r {
                Ok(p) => {
                    let (l, w, a, o, g, s, e, b) = measure(p);
                    format!("{ms:>7} {l:>7} {w:>6} {a:>5} {o:>5} {g:>5} {s:>5} {e:>4} {b:>5}")
                }
                Err(e) => format!("{:<57}", format!("ERROR {}", e.chars().take(50).collect::<String>())),
            };
            let pages = match (&extract, &pdfium_pages) {
                (Ok(a), Ok(b)) if a.len() == b.len() => a.len().to_string(),
                (Ok(a), Ok(b)) => format!("{}≠{}", a.len(), b.len()),
                (Ok(a), _) => a.len().to_string(),
                (_, Ok(b)) => b.len().to_string(),
                _ => "-".into(),
            };
            let agree = match (&extract, &pdfium_pages) {
                (Ok(a), Ok(b)) => {
                    let (wa, wb) = (words(a), words(b));
                    format!("{:>4.0}% {:>4.0}%", 100.0 * shared(&wa, &wb), 100.0 * shared(&wb, &wa))
                }
                _ => String::new(),
            };
            println!("{name:<36} {pages:>5} | {} | {} | {agree}", cols(&extract, te), cols(&pdfium_pages, tp));
            // which odd signs, by their code: never the text around them
            for (engine, r) in [("pdf-extract", &extract), ("Pdfium", &pdfium_pages)] {
                let mut odd: Vec<String> = r.iter().flatten().flat_map(|p| p.chars()).filter(|&c| c == '\u{FFFD}' || (c.is_control() && c != '\n' && c != '\t') || ('\u{E000}'..='\u{F8FF}').contains(&c)).map(|c| format!("U+{:04X}", c as u32)).collect();
                odd.dedup();
                if !odd.is_empty() {
                    println!("{:<42} signos raros de {engine}: {:?}", "", odd);
                }
            }
            if let Some((file, page)) = show.as_deref().and_then(|s| s.rsplit_once(':')) {
                if path.file_name().is_some_and(|n| n.to_string_lossy() == file) {
                    let n: usize = page.parse().unwrap_or(1);
                    for (engine, r) in [("pdf-extract", &extract), ("Pdfium", &pdfium_pages)] {
                        let text = r.as_ref().ok().and_then(|p| p.get(n - 1)).cloned().unwrap_or_default();
                        println!("----- {engine}, página {n} -----\n{}\n", text.chars().take(1500).collect::<String>());
                    }
                }
            }
        }
        println!("\nE→P: de las palabras de pdf-extract, cuántas saca igual Pdfium; P→E, al revés. «1letr»: palabras de una sola letra (texto partido letra por letra); «pegad»: palabras de más de 25 letras (palabras pegadas); «vac»: páginas que quedarían para leer como imagen.");
    }

    /// Pdfium marks no paragraph: lines far apart start one, lines close together are the same one, and a line that
    /// goes up the page (another column) starts one too. A line with only spaces is left out.
    #[test]
    fn paragraphs_come_from_how_far_apart_the_lines_are() {
        let line = |text: &str, top: f32| Line { text: text.into(), top, bottom: top - 10.0 };
        let lines = [
            line("Objetivos ", 700.0),
            line("1. Dar albergue a personas", 680.0),
            line("adultas mayores.", 668.0),
            Line { text: "  ".into(), ..Line::new() },
            line("2. Dar alimento.", 650.0),
            line("Segunda columna", 700.0),
        ];
        assert_eq!(join_lines(&lines), "Objetivos\n\n1. Dar albergue a personas\nadultas mayores.\n\n2. Dar alimento.\n\nSegunda columna");
    }

    /// A PDF with a password is said to have one, not to be damaged.
    #[test]
    fn a_pdf_with_a_password_says_so() {
        assert!(matches!(read_pdf(&fixture("protegido.pdf")), Err(PdfError::Protected)));
    }

    /// The text of a PDF is Pdfium's, with an empty line between paragraphs and every word whole.
    #[test]
    fn the_text_of_a_page_keeps_its_paragraphs() {
        let r = read_pdf(&fixture("convocatoria-con-tablas.pdf")).unwrap();
        assert!(r.pages[0].starts_with("Convocatoria ficticia 2026\n\nRequisito REQ-01"), "{}", r.pages[0]);
        assert!(r.pages[0].contains("Monto máximo (MXN)"), "{}", r.pages[0]);
    }

    #[test]
    fn garbage_file_is_an_error_not_a_crash() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("x.pdf");
        std::fs::write(&p, b"this is not a pdf").unwrap();
        assert!(read_pdf(&p).is_err());
    }
}
