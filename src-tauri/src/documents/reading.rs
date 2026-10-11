//! A file read into pages of text, saying which pages came from a picture (OCR, ADR-035):
//!
//! - a PDF page by page; a page with no text layer (a scan) is drawn and read as a picture;
//! - a Word or Excel file in pieces, as before (a picture inside it is not read);
//! - a photo (JPEG or PNG) as one page.
//!
//! The text comes out raw: whoever keeps it cleans it and passes it through the scanner. Reading pictures takes a
//! few seconds per page: the pages of a file are read side by side and the screen hears how many are done.

use super::ocr::{self, PageRead};
use sha2::{Digest, Sha256};
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{mpsc, Mutex, OnceLock};

/// More pages to read as pictures than this in one file is too long to wait for.
pub const MAX_PICTURE_PAGES: usize = 100;

/// The kinds of file read here, by their extension.
pub const EXTENSIONS: &[&str] = &["pdf", "docx", "xlsx", "xlsm", "jpg", "jpeg", "png"];

#[derive(Debug, Clone, PartialEq)]
pub struct Reading {
    pub pages: Vec<String>,
    /// The pages (numbers from 1) whose text was read from a picture.
    pub from_picture: Vec<usize>,
    /// Of those, the ones read with doubts: the person is asked to check them.
    pub doubtful: Vec<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReadError {
    Unsupported,
    Damaged,
    /// More pages to read as pictures than `MAX_PICTURE_PAGES`.
    TooManyPictures,
}

/// Who hears how a reading of pictures goes (the screen, through an event): pages done and pages in all.
type Listener = Box<dyn Fn(usize, usize) + Send + Sync>;
static LISTENER: OnceLock<Listener> = OnceLock::new();

/// Set once at start: who is told how many pages are read.
pub fn on_progress(listener: impl Fn(usize, usize) + Send + Sync + 'static) {
    let _ = LISTENER.set(Box::new(listener));
}

fn tell(done: usize, total: usize) {
    if let Some(l) = LISTENER.get() {
        l(done, total);
    }
}

/// The last file read from pictures, by the fingerprint of its bytes: a file held for the scanner comes back with
/// the decision of the person and is not read again. In memory only, never on disk.
static LAST: Mutex<Option<(String, Reading)>> = Mutex::new(None);

fn extension(path: &Path) -> String {
    path.extension().map(|e| e.to_string_lossy().to_lowercase()).unwrap_or_default()
}

/// Reads a file from where it is.
pub fn read_file(path: &Path) -> Result<Reading, ReadError> {
    let ext = extension(path);
    if !EXTENSIONS.contains(&ext.as_str()) {
        return Err(ReadError::Unsupported);
    }
    if !matches!(ext.as_str(), "pdf" | "jpg" | "jpeg" | "png") {
        let pages = super::canonical::package::read_pieces(path).map_err(|_| ReadError::Damaged)?;
        return Ok(Reading { pages, from_picture: vec![], doubtful: vec![] });
    }
    let bytes = std::fs::read(path).map_err(|_| ReadError::Damaged)?;
    let key = hex::encode(Sha256::digest(&bytes));
    if let Some((k, r)) = LAST.lock().ok().and_then(|l| l.clone()) {
        if k == key {
            return Ok(r);
        }
    }
    let reading = if ext == "pdf" { read_pdf(path)? } else { read_photo(&bytes) };
    if !reading.from_picture.is_empty() {
        if let Ok(mut l) = LAST.lock() {
            *l = Some((key, reading.clone()));
        }
    }
    Ok(reading)
}

/// Reads a file the screen sent as bytes: it is put in a folder of its own only while it is read, and the folder is
/// gone when this returns, whatever happens (principle 6).
pub fn read_upload(name: &str, bytes: &[u8]) -> Result<Reading, ReadError> {
    let ext = name.rsplit('.').next().map(str::to_lowercase).unwrap_or_default();
    if !EXTENSIONS.contains(&ext.as_str()) || !name.contains('.') {
        return Err(ReadError::Unsupported);
    }
    let dir = Scratch::new().map_err(|_| ReadError::Damaged)?;
    let path = dir.0.join(format!("upload.{ext}"));
    std::fs::write(&path, bytes).map_err(|_| ReadError::Damaged)?;
    read_file(&path)
}

/// A folder for one upload that is gone when it is dropped, whatever happens in between.
struct Scratch(std::path::PathBuf);

impl Scratch {
    fn new() -> std::io::Result<Scratch> {
        let dir = std::env::temp_dir().join(format!("cimiento-{}", ulid::Ulid::generate()));
        std::fs::create_dir_all(&dir)?;
        Ok(Scratch(dir))
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn read_photo(bytes: &[u8]) -> Reading {
    tell(0, 1);
    let read = ocr::read_picture(bytes, None).ok();
    tell(1, 1);
    let mut r = Reading { pages: vec![String::new()], from_picture: vec![], doubtful: vec![] };
    if let Some(read) = read.filter(|p| !p.text.trim().is_empty()) {
        r.from_picture.push(1);
        if read.doubtful() {
            r.doubtful.push(1);
        }
        r.pages[0] = read.text;
    }
    r
}

fn read_pdf(path: &Path) -> Result<Reading, ReadError> {
    let (mut pages, scanned) = match super::pdf::read_pdf_with_rows(path) {
        Ok(t) => (t.pages, t.pages_without_text),
        Err(_) => {
            // the text could not be taken out; the pages may still be drawn (a scan the extractor does not follow)
            let n = ocr::page_count(path).map_err(|_| ReadError::Damaged)?;
            (vec![String::new(); n], (1..=n).collect())
        }
    };
    let mut reading = Reading { pages: vec![], from_picture: vec![], doubtful: vec![] };
    if !scanned.is_empty() && ocr::available() {
        if scanned.len() > MAX_PICTURE_PAGES {
            return Err(ReadError::TooManyPictures);
        }
        for (n, read) in read_pictures(path, &scanned) {
            if read.text.trim().is_empty() {
                continue;
            }
            reading.from_picture.push(n);
            if read.doubtful() {
                reading.doubtful.push(n);
            }
            pages[n - 1] = read.text;
        }
    }
    reading.pages = pages;
    Ok(reading)
}

/// Reads pages of a PDF as pictures: one thread draws them in order and a few read them side by side, one page each.
/// A page that cannot be drawn or read is left out.
fn read_pictures(path: &Path, numbers: &[usize]) -> Vec<(usize, PageRead)> {
    let total = numbers.len();
    let readers = std::thread::available_parallelism().map_or(2, |n| n.get() / 2).clamp(1, 4);
    let (send, take) = mpsc::sync_channel::<(usize, Vec<u8>, u32)>(readers);
    let (take, done, out) = (Mutex::new(take), AtomicUsize::new(0), Mutex::new(Vec::new()));
    tell(0, total);
    std::thread::scope(|s| {
        for _ in 0..readers {
            s.spawn(|| loop {
                let Ok((n, picture, dpi)) = take.lock().map_err(|_| ()).and_then(|t| t.recv().map_err(|_| ())) else { break };
                if let Ok(read) = ocr::read_picture(&picture, Some(dpi)) {
                    if let Ok(mut o) = out.lock() {
                        o.push((n, read));
                    }
                }
                tell(done.fetch_add(1, Ordering::SeqCst) + 1, total);
            });
        }
        let _ = ocr::draw_pages(path, numbers, |n, picture, dpi| {
            let _ = send.send((n, picture, dpi));
        });
        drop(send);
    });
    let mut out = out.into_inner().unwrap_or_default();
    out.sort_by_key(|(n, _)| *n);
    out
}

#[cfg(test)]
mod tests;
