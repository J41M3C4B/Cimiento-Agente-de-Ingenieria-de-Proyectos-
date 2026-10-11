//! Reading the text of a picture (OCR, ADR-035). Tesseract reads a page; Pdfium first draws a page of a PDF as a
//! picture. Both ship with the program in `resources/ocr` (`scripts/fetch-ocr.ps1`). Nothing leaves the computer
//! and no picture is written to disk: a page reaches Tesseract through its standard input.

use pdfium_render::prelude::*;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::OnceLock;
use std::time::{Duration, Instant};

/// Below this mean confidence (0–100) a page is said to be read with doubts: it is kept and the person is told
/// (decision of the institution, 2026-10-10).
pub const DOUBTFUL_BELOW: f32 = 70.0;
/// The resolution a page of a PDF is drawn at for Tesseract.
const DPI: f32 = 300.0;
/// No side of a drawn page is longer than this: a plan or a poster would take too much memory.
const MAX_SIDE: f32 = 5000.0;
/// English first, then Spanish: the Spanish model has no «@», and an email read as «contacto(Dcasa…» would escape
/// the scanner. Together they read the accents and the emails of Spanish papers (measured on the fixtures).
const LANGUAGES: &str = "eng+spa";
/// A gap between two words of a line wider than this many times their height parts two cells of a table: a space
/// between words is about a third of their height, and a justified line stays under one.
const COLUMN_GAP: f32 = 1.5;
/// A page Tesseract has not read in this time is given up.
const PAGE_TIMEOUT: Duration = Duration::from_secs(120);

#[derive(Debug, thiserror::Error)]
pub enum OcrError {
    #[error("what reads pictures is not installed")]
    Missing,
    #[error("tesseract failed: {0}")]
    Failed(String),
    #[error("tesseract took too long")]
    TimedOut,
    #[error("the PDF could not be drawn: {0}")]
    Draw(String),
}

/// The text of a picture and how sure Tesseract is of it.
#[derive(Debug, Clone, PartialEq)]
pub struct PageRead {
    pub text: String,
    /// The mean confidence of its words (0–100), each weighed by its letters; 0 with no words.
    pub confidence: f32,
}

impl PageRead {
    pub fn doubtful(&self) -> bool {
        self.confidence < DOUBTFUL_BELOW
    }
}

static HOME: OnceLock<PathBuf> = OnceLock::new();

/// Where the installed program keeps what reads pictures: set once at start, from its resources.
pub fn set_home(dir: PathBuf) {
    let _ = HOME.set(plain_path(dir));
}

/// A path without the `\\?\` of Windows: Tauri gives the resources that way, and Tesseract cannot open its models
/// through it («Error opening data file \\?\D:\…»).
fn plain_path(dir: PathBuf) -> PathBuf {
    match dir.to_str().and_then(|s| s.strip_prefix(r"\\?\")) {
        Some(rest) if !rest.starts_with(r"UNC\") => PathBuf::from(rest),
        _ => dir,
    }
}

fn home() -> Option<PathBuf> {
    // the tests and a build in development read it from the source tree
    HOME.get().cloned().or_else(|| cfg!(debug_assertions).then(|| Path::new(env!("CARGO_MANIFEST_DIR")).join("resources").join("ocr")))
}

fn tesseract() -> Option<(PathBuf, PathBuf)> {
    let home = home()?;
    let exe = home.join("tesseract").join(if cfg!(windows) { "tesseract.exe" } else { "tesseract" });
    let data = home.join("tessdata");
    (exe.is_file() && data.join("spa.traineddata").is_file() && data.join("eng.traineddata").is_file()).then_some((exe, data))
}

/// Whether pictures can be read on this computer.
pub fn available() -> bool {
    tesseract().is_some()
}

/// Reads the text of a picture (JPEG, PNG or PGM). `dpi` when it is known (a drawn page); a photo
/// says its own or Tesseract estimates it.
///
/// A page read with doubts is read once more with the adaptive threshold of Leptonica, and the surer of the two
/// is kept: a faded copy is read where the plain threshold finds nothing, but on a good page the adaptive one
/// misreads signs like the «@» of an email, which the scanner then would not find.
pub fn read_picture(picture: &[u8], dpi: Option<u32>) -> Result<PageRead, OcrError> {
    let plain = run(picture, dpi, false)?;
    if !plain.doubtful() {
        return Ok(plain);
    }
    match run(picture, dpi, true) {
        Ok(adaptive) if adaptive.confidence > plain.confidence => Ok(adaptive),
        _ => Ok(plain),
    }
}

fn run(picture: &[u8], dpi: Option<u32>, adaptive: bool) -> Result<PageRead, OcrError> {
    let (exe, data) = tesseract().ok_or(OcrError::Missing)?;
    let mut cmd = Command::new(exe);
    // page segmentation 4 (one column of lines of any size): a table is read row by row; the automatic one (3) cut it
    // into a block per column. Two columns of prose came out the same with both (measured 2026-10-11)
    cmd.args(["stdin", "stdout", "--tessdata-dir"]).arg(&data).args(["-l", LANGUAGES, "--oem", "1", "--psm", "4"]);
    if let Some(dpi) = dpi {
        cmd.args(["--dpi", &dpi.to_string()]);
    }
    if adaptive {
        cmd.args(["-c", "thresholding_method=1"]);
    }
    // one thread each: the pages of a file are read side by side
    cmd.arg("tsv").env("OMP_THREAD_LIMIT", "1").stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    let mut child = cmd.spawn().map_err(|e| OcrError::Failed(e.to_string()))?;
    let (Some(mut input), Some(mut output), Some(mut errors)) = (child.stdin.take(), child.stdout.take(), child.stderr.take()) else {
        return Err(OcrError::Failed("no pipes".into()));
    };
    let (status, tsv, said) = std::thread::scope(|s| {
        // the picture is written while the answer is read: neither side waits for the other
        s.spawn(move || {
            let _ = input.write_all(picture);
        });
        let out = s.spawn(move || {
            let mut b = String::new();
            let _ = output.read_to_string(&mut b);
            b
        });
        let err = s.spawn(move || {
            let mut b = String::new();
            let _ = errors.read_to_string(&mut b);
            b
        });
        let start = Instant::now();
        let status = loop {
            match child.try_wait() {
                Ok(Some(status)) => break Ok(status),
                Ok(None) if start.elapsed() > PAGE_TIMEOUT => {
                    let _ = child.kill();
                    let _ = child.wait();
                    break Err(OcrError::TimedOut);
                }
                Ok(None) => std::thread::sleep(Duration::from_millis(30)),
                Err(e) => break Err(OcrError::Failed(e.to_string())),
            }
        };
        (status, out.join().unwrap_or_default(), err.join().unwrap_or_default())
    });
    if !status?.success() {
        return Err(OcrError::Failed(said.lines().last().unwrap_or_default().to_string()));
    }
    Ok(from_tsv(&tsv))
}

/// The text of Tesseract's table of words: a space between words, a line break between lines and an empty line
/// between paragraphs. Words of a line far apart are cells of a table: they are kept apart with « | », as the rows
/// of a PDF with text are (`pdf_rows`).
fn from_tsv(tsv: &str) -> PageRead {
    let mut text = String::new();
    let mut at: Option<(&str, &str, &str)> = None;
    // where the word before ends and how tall it is
    let (mut right, mut tall) = (0i64, 0i64);
    let (mut letters, mut sum) = (0f32, 0f32);
    for row in tsv.lines().skip(1) {
        let c: Vec<&str> = row.split('\t').collect();
        // level 5 is a word: level, page, block, paragraph, line, word, left, top, width, height, conf, text
        if c.len() < 12 || c[0] != "5" {
            continue;
        }
        let (word, conf) = (c[11].trim(), c[10].parse::<f32>().unwrap_or(-1.0));
        if word.is_empty() || conf < 0.0 {
            continue;
        }
        let place = (c[2], c[3], c[4]);
        let number = |i: usize| c[i].parse::<i64>().unwrap_or(0);
        let (left, width, height) = (number(6), number(8), number(9));
        match at {
            None => {}
            Some(p) if p == place && (left - right) as f32 > COLUMN_GAP * tall.max(height) as f32 => text.push_str(" | "),
            Some(p) if p == place => text.push(' '),
            Some(p) if (p.0, p.1) == (place.0, place.1) => text.push('\n'),
            Some(_) => text.push_str("\n\n"),
        }
        at = Some(place);
        (right, tall) = (left + width, height);
        text.push_str(word);
        let n = word.chars().count() as f32;
        letters += n;
        sum += conf * n;
    }
    PageRead { text, confidence: if letters > 0.0 { sum / letters } else { 0.0 } }
}

static PDFIUM: OnceLock<Option<Pdfium>> = OnceLock::new();

fn pdfium() -> Option<&'static Pdfium> {
    PDFIUM
        .get_or_init(|| {
            let dir = home()?.join("pdfium");
            Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path(&dir)).ok().map(Pdfium::new)
        })
        .as_ref()
}

/// How many pages a PDF has, by Pdfium: for a file whose text cannot be taken out.
pub fn page_count(path: &Path) -> Result<usize, OcrError> {
    let doc = pdfium().ok_or(OcrError::Missing)?.load_pdf_from_file(path, None).map_err(|e| OcrError::Draw(e.to_string()))?;
    Ok(doc.pages().len() as usize)
}

/// Draws pages of a PDF (numbers from 1), one by one in gray, as Tesseract reads them (PGM), with the resolution
/// each was drawn at.
pub fn draw_pages(path: &Path, numbers: &[usize], mut each: impl FnMut(usize, Vec<u8>, u32)) -> Result<(), OcrError> {
    let doc = pdfium().ok_or(OcrError::Missing)?.load_pdf_from_file(path, None).map_err(|e| OcrError::Draw(e.to_string()))?;
    for &n in numbers {
        let page = doc.pages().get(n as PdfPageIndex - 1).map_err(|e| OcrError::Draw(e.to_string()))?;
        let longest = page.width().value.max(page.height().value).max(1.0);
        let scale = (DPI / 72.0).min(MAX_SIDE / longest);
        let config = PdfRenderConfig::new().scale_page_by_factor(scale).set_format(PdfBitmapFormat::BGRA);
        let bitmap = page.render_with_config(&config).map_err(|e| OcrError::Draw(e.to_string()))?;
        let (w, h) = (bitmap.width() as usize, bitmap.height() as usize);
        each(n, gray_pgm(&bitmap.as_raw_bytes(), w, h), (72.0 * scale).round() as u32);
    }
    Ok(())
}

/// A BGRA picture as a gray PGM: what Tesseract reads, a quarter of the size.
fn gray_pgm(bgra: &[u8], w: usize, h: usize) -> Vec<u8> {
    let stride = if h > 0 { bgra.len() / h } else { 0 };
    let mut out = format!("P5\n{w} {h}\n255\n").into_bytes();
    out.reserve(w * h);
    for y in 0..h {
        for px in bgra[y * stride..].chunks_exact(4).take(w) {
            let (b, g, r) = (px[0] as u32, px[1] as u32, px[2] as u32);
            out.push(((29 * b + 150 * g + 77 * r) >> 8) as u8);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_table_of_words_becomes_lines_and_paragraphs_and_its_confidence_counts_letters() {
        let tsv = "level\tpage_num\tblock_num\tpar_num\tline_num\tword_num\tleft\ttop\twidth\theight\tconf\ttext\n\
                   1\t1\t0\t0\t0\t0\t0\t0\t100\t100\t-1\t\n\
                   5\t1\t1\t1\t1\t1\t0\t0\t1\t1\t90\tActa\n\
                   5\t1\t1\t1\t1\t2\t0\t0\t1\t1\t90\tconstitutiva\n\
                   5\t1\t1\t1\t2\t1\t0\t0\t1\t1\t60\tdel\n\
                   5\t1\t1\t2\t1\t1\t0\t0\t1\t1\t30\tx\n\
                   5\t1\t2\t1\t1\t1\t0\t0\t1\t1\t-1\t \n";
        let read = from_tsv(tsv);
        assert_eq!(read.text, "Acta constitutiva\ndel\n\nx");

        // a row of a table: words far apart are cells, words close together are one cell
        let row = "level\tpage_num\tblock_num\tpar_num\tline_num\tword_num\tleft\ttop\twidth\theight\tconf\ttext\n\
                   5\t1\t1\t1\t1\t1\t100\t0\t120\t30\t95\tProducto\n\
                   5\t1\t1\t1\t1\t2\t230\t0\t15\t30\t95\t1\n\
                   5\t1\t1\t1\t1\t3\t600\t0\t15\t30\t95\t2\n\
                   5\t1\t1\t1\t1\t4\t900\t0\t60\t30\t95\t€100\n";
        assert_eq!(from_tsv(row).text, "Producto 1 | 2 | €100");
        // (4·90 + 12·90 + 3·60 + 1·30) / 20
        assert!((read.confidence - 82.5).abs() < 0.01, "{}", read.confidence);
        assert_eq!(from_tsv("level\n").confidence, 0.0);
    }

    #[test]
    fn the_folder_of_the_resources_loses_the_long_path_prefix_of_windows() {
        assert_eq!(plain_path(PathBuf::from(r"\\?\D:\Programas\Cimiento\ocr")), PathBuf::from(r"D:\Programas\Cimiento\ocr"));
        assert_eq!(plain_path(PathBuf::from(r"\\?\UNC\servidor\ocr")), PathBuf::from(r"\\?\UNC\servidor\ocr"));
        assert_eq!(plain_path(PathBuf::from(r"D:\ocr")), PathBuf::from(r"D:\ocr"));
    }

    #[test]
    fn a_drawn_page_turns_gray_row_by_row() {
        // 2×1 pixels in BGRA, rows padded to 12 bytes
        let bgra = [0, 0, 0, 255, 255, 255, 255, 255, 9, 9, 9, 9];
        assert_eq!(gray_pgm(&bgra, 2, 1), b"P5\n2 1\n255\n\x00\xff".to_vec());
    }
}
