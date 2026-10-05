//! PDF: text-only extraction (no OCR). Scanned pages are detected and reported.

use std::path::Path;

/// A page with fewer visible characters than this is treated as having no text layer.
const MIN_CHARS_PER_PAGE: usize = 20;

#[derive(Debug, thiserror::Error)]
#[allow(dead_code)]
pub enum PdfError {
    #[error("the PDF could not be read: {0}")]
    Unreadable(String),
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

#[allow(dead_code)]
pub fn read_pdf(path: &Path) -> Result<PdfText, PdfError> {
    // The extractor can panic on malformed files; turn that into an error.
    let path = path.to_path_buf();
    let pages = std::panic::catch_unwind(move || pdf_extract::extract_text_by_pages(&path))
        .map_err(|_| PdfError::Unreadable("unsupported or damaged file".into()))?
        .map_err(|e| PdfError::Unreadable(e.to_string()))?;
    let pages_without_text = pages
        .iter()
        .enumerate()
        .filter(|(_, t)| t.chars().filter(|c| !c.is_whitespace()).count() < MIN_CHARS_PER_PAGE)
        .map(|(i, _)| i + 1)
        .collect();
    Ok(PdfText {
        pages,
        pages_without_text,
    })
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

    #[test]
    fn garbage_file_is_an_error_not_a_crash() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("x.pdf");
        std::fs::write(&p, b"this is not a pdf").unwrap();
        assert!(read_pdf(&p).is_err());
    }
}
