//! Reading PDF/Excel/Word and pictures (OCR), templates and export.

pub mod canonical;
pub mod docx;
pub mod chunking;
pub mod ocr;
pub mod pdf;
pub mod pdf_grid;
pub mod pdf_rows;
pub mod reading;
pub mod text;
pub mod xlsx;

use std::path::Path;

#[allow(dead_code)]
pub struct ExtractedDocument {
    pub text: String,
    pub pages: Vec<String>,
}

#[allow(dead_code)]
pub trait DocumentReader {
    fn read(&self, path: &Path) -> Result<ExtractedDocument, String>;
}
