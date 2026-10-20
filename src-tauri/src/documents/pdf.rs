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

/// A page mostly covered by pictures with little text is a scan with a printed line or two (a header, a stamp) or a
/// scanner's own text layer: it is read as a picture, and what OCR reads replaces its text (ADR-037).
const MIXED_PICTURE_SHARE: f64 = 0.5;
const MIXED_MAX_LETTERS: usize = 200;

/// The text of each page.
pub fn read_pdf(path: &Path) -> Result<PdfText, PdfError> {
    read(path, false)
}

/// Like `read_pdf`, and each page also carries the rows of its tables rebuilt from where its letters sit and the
/// lines it draws (`pdf_rows`, `pdf_grid`), appended as a block of their own. The plain text is never changed.
pub fn read_pdf_with_rows(path: &Path) -> Result<PdfText, PdfError> {
    read(path, true)
}

/// One pass over the pages with Pdfium: the text, and, when asked, the letters and lines that make its tables. A
/// page whose tables cannot be read keeps just its text.
fn read(path: &Path, with_rows: bool) -> Result<PdfText, PdfError> {
    let pdfium = super::ocr::pdfium().ok_or_else(|| PdfError::Unreadable("what reads PDF is not installed".into()))?;
    let doc = pdfium.load_pdf_from_file(path, None).map_err(|e| match e {
        PdfiumError::PdfiumLibraryInternalError(PdfiumInternalError::PasswordError) => PdfError::Protected,
        e => PdfError::Unreadable(format!("{e:?}")),
    })?;
    let (mut pages, mut rows, mut mixed) = (Vec::new(), Vec::new(), Vec::new());
    for (i, page) in doc.pages().iter().enumerate() {
        let text = page.text().map_err(|e| PdfError::Unreadable(format!("{e:?}")))?;
        let plain = page_text(&text);
        if picture_share(&page) >= MIXED_PICTURE_SHARE && plain.chars().filter(|c| c.is_alphabetic()).count() < MIXED_MAX_LETTERS {
            mixed.push(i + 1);
        }
        if with_rows {
            let height = f64::from(page.height().value);
            let found = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let glyphs = super::pdf_rows::glyphs_of(&text, height);
                super::pdf_rows::page_rows(&glyphs, &super::pdf_grid::rules_of_page(&page), height)
            }));
            rows.push(found.unwrap_or_default());
        }
        pages.push(plain);
    }
    let mut out = with_empty_pages(pages);
    if with_rows {
        for (page, rows) in out.pages.iter_mut().zip(super::pdf_rows::without_running_rows(rows)) {
            *page = super::pdf_rows::with_rows(page, &rows);
        }
    }
    out.pages_without_text.extend(mixed);
    out.pages_without_text.sort_unstable();
    out.pages_without_text.dedup();
    Ok(out)
}

/// How much of a page its pictures cover (0 to 1), by their boxes; a picture inside a form counts by the form's box.
fn picture_share(page: &PdfPage) -> f64 {
    fn has_picture(object: &PdfPageObject, depth: usize) -> bool {
        match object {
            PdfPageObject::Image(_) => true,
            PdfPageObject::XObjectForm(form) if depth < 3 => form.iter().any(|c| has_picture(&c, depth + 1)),
            _ => false,
        }
    }
    let (w, h) = (f64::from(page.width().value), f64::from(page.height().value));
    if w <= 0.0 || h <= 0.0 {
        return 0.0;
    }
    let covered: f64 = page
        .objects()
        .iter()
        .filter(|o| has_picture(o, 0))
        .filter_map(|o| o.bounds().ok())
        .map(|q| {
            let r = q.to_rect();
            let (l, r_, b, t) = (f64::from(r.left().value).max(0.0), f64::from(r.right().value).min(w), f64::from(r.bottom().value).max(0.0), f64::from(r.top().value).min(h));
            (r_ - l).max(0.0) * (t - b).max(0.0)
        })
        .sum();
    (covered / (w * h)).min(1.0)
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


/// Like `read_pdf`, and each page also carries the rows of its tables rebuilt from where the letters sit

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

    /// What Pdfium takes out of each PDF of a folder (ADR-037): pages, time, letters, words, accents, odd signs,
    /// glued words, words of one letter (a text cut letter by letter), pages left to read as pictures, paragraphs and
    /// rows of tables. Prints no text unless a page is asked for; only for public samples.
    ///   $env:SOCIAI_PDF_DIR="D:\...\pdf_pruebas"; cargo test pdf_report -- --ignored --nocapture > salida.txt
    ///   (optional: $env:SOCIAI_PDF_SHOW="archivo.pdf:3", the text of that page)
    #[test]
    #[ignore]
    fn pdf_report() {
        use std::time::Instant;
        let dir = std::path::PathBuf::from(std::env::var("SOCIAI_PDF_DIR").expect("SOCIAI_PDF_DIR"));
        let show = std::env::var("SOCIAI_PDF_SHOW").ok();
        let mut files: Vec<_> = std::fs::read_dir(&dir).unwrap().filter_map(|e| e.ok()).map(|e| e.path()).filter(|p| p.extension().is_some_and(|e| e.eq_ignore_ascii_case("pdf"))).collect();
        files.sort();
        let block = super::super::pdf_rows::BLOCK_TITLE;
        println!("{:<36} {:>5} {:>6} {:>7} {:>6} {:>5} {:>5} {:>5} {:>5} {:>4} {:>5} {:>5}", "archivo", "pág", "ms", "letras", "palab", "acent", "raros", "pegad", "1letr", "vac", "párr", "filas");
        for path in &files {
            let name: String = path.file_name().unwrap().to_string_lossy().chars().take(36).collect();
            let t = Instant::now();
            let read = match read_pdf_with_rows(path) {
                Ok(r) => r,
                Err(e) => {
                    println!("{name:<36} {e}");
                    continue;
                }
            };
            let ms = t.elapsed().as_millis();
            let plain: Vec<&str> = read.pages.iter().map(|p| p.split(block).next().unwrap_or_default()).collect();
            let all = plain.join("\n");
            let words: Vec<&str> = all.split_whitespace().collect();
            let letters = all.chars().filter(|c| !c.is_whitespace()).count();
            let accents = all.chars().filter(|c| "áéíóúñüÁÉÍÓÚÑÜ".contains(*c)).count();
            let odd = all.chars().filter(|&c| c == '\u{FFFD}' || (c.is_control() && c != '\n' && c != '\t') || ('\u{E000}'..='\u{F8FF}').contains(&c)).count();
            let glued = words.iter().filter(|w| w.chars().filter(|c| c.is_alphabetic()).count() > 25).count();
            let singles = words.iter().filter(|w| w.chars().count() == 1 && w.chars().all(char::is_alphabetic)).count();
            let paragraphs: usize = plain.iter().map(|p| p.split("\n\n").filter(|b| !b.trim().is_empty()).count()).sum();
            let rows: usize = read.pages.iter().map(|p| p.split_once(block).map_or(0, |(_, r)| r.lines().filter(|l| !l.trim().is_empty()).count())).sum();
            println!("{name:<36} {:>5} {ms:>6} {letters:>7} {:>6} {accents:>5} {odd:>5} {glued:>5} {singles:>5} {:>4} {paragraphs:>5} {rows:>5}", read.pages.len(), words.len(), read.pages_without_text.len());
            if let Some((file, page)) = show.as_deref().and_then(|s| s.rsplit_once(':')) {
                if path.file_name().is_some_and(|n| n.to_string_lossy() == file) {
                    let n: usize = page.parse().unwrap_or(1);
                    println!("----- página {n} -----\n{}\n", read.pages.get(n - 1).map(|p| p.chars().take(2000).collect::<String>()).unwrap_or_default());
                }
            }
        }
        println!("\n«1letr»: palabras de una sola letra (texto partido letra por letra); «pegad»: palabras de más de 25 letras; «vac»: páginas que se leen como imagen; «párr»: párrafos; «filas»: filas de tabla.");
    }

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
