//! What every drawer does when a file comes in (ADR-036 §2): its pages through the scanner, which covers what it
//! finds. The file itself is read by `documents::reading` and kept by the owner of its drawer (today the Expediente,
//! `core::records`); here only the clean text is decided.

use crate::core::error::ServiceError;
use crate::core::screen::scanner_for;
use crate::scanner::guard::Decision;
use crate::scanner::SensitiveScanner;
use rusqlite::Connection;

/// The pages of a file with the data the scanner found covered, and how many were covered.
pub struct CoveredPages {
    pub pages: Vec<String>,
    pub covered: i64,
}

/// Passes the pages of a file through the scanner (each page a field, `page_1`…), for the table `entity`, and covers
/// what it finds. A paper of the institution names its people (an act, its partners; a tax certificate, its
/// representative), and how many is never known: it is kept with them covered, never refused nor held for the person
/// to decide (decision of the institution, 2026-10-13). Each covering goes to the log.
/// `pictures`: some page was read from a picture, where an email may come with its «@» misread (OCR, ADR-035).
pub fn cover_pages(conn: &Connection, entity: &str, pages: &[String], pictures: bool) -> Result<CoveredPages, ServiceError> {
    let scanner = if pictures { scanner_for(conn)?.reading_pictures() } else { scanner_for(conn)? };
    let fields: Vec<(String, String)> = pages.iter().enumerate().map(|(i, p)| (format!("page_{}", i + 1), p.clone())).collect();
    let covered = fields.iter().map(|(_, p)| scanner.scan(p).findings.len() as i64).sum();
    match crate::scanner::guard::screen_texts(conn, &scanner, entity, &fields, Some(Decision::Redact))? {
        Ok(pages) => Ok(CoveredPages { pages, covered }),
        // covering never holds a text
        Err(_) => Err(ServiceError::Internal("the scanner held a text it was told to cover".into())),
    }
}
