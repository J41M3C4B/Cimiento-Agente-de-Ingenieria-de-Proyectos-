//! Reading the text of a picture (OCR, ADR-035). Tesseract reads a page; Pdfium first draws a page of a PDF as a
//! picture. Both ship with the program in `resources/ocr` (`scripts/fetch-ocr.ps1`). Nothing leaves the computer
//! and no picture is written to disk: a page reaches Tesseract through its standard input.

use pdfium_render::prelude::*;
use std::borrow::Cow;
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
/// Letters shorter than this many pixels are read badly (a screenshot, a little photo): the picture is enlarged and
/// read again. Measured 2026-10-13: a screenshot with letters of 7 pixels lost a number and a title, and read «m» for
/// «rn»; four times bigger, it read them all.
const SMALL_LETTERS: f32 = 20.0;
/// How tall the letters of an enlarged picture are made...
const LETTERS_TO: f32 = 30.0;
/// ...but it is never enlarged more than this.
const MAX_ENLARGE: f32 = 4.0;
/// Fewer words than this do not tell how tall the letters are.
const WORDS_TO_MEASURE: usize = 5;
/// A picture shorter than this on its longest side where no word was measured is enlarged anyway, this many times:
/// its letters may be too small to be found at all (a little photo taken from afar). A page of a PDF or a photo of a
/// phone is bigger: a blank one is not read again.
const SMALL_PICTURE: u32 = 1200;
const UNREAD_ENLARGE: f32 = 3.0;
/// How sure Tesseract must be that a page is turned before it is put upright and read again (its «orientation
/// confidence»; a page turned on its side gives 12 to 15, measured on the fixtures 2026-10-13).
const TURN_SURE: f32 = 2.0;
/// A pixel this much darker than the paper is ink.
const INK_BELOW_PAPER: u8 = 50;
/// A page with less ink than this share is blank or has only a stamp: it is not read again.
const LITTLE_INK: f32 = 0.002;

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
    /// How tall its letters are, in pixels: the lower quarter of the heights of its words. Tesseract gives some
    /// words the height of their whole line, never less than their own, so the lower quarter is the true one.
    /// `None` with fewer than `WORDS_TO_MEASURE` words.
    pub letters: Option<f32>,
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
/// A picture with small letters is enlarged and read again, and the surer of the two readings is kept; one whose
/// letters are big enough is not read twice for this. A page still read with doubts that has ink on it is checked
/// for being turned (on its side, upside down) and, if so, put upright and read again; a blank one is left as it is.
///
/// A page read with doubts is read once more with the adaptive threshold of Leptonica, and the surer of the two
/// is kept: a faded copy is read where the plain threshold finds nothing, but on a good page the adaptive one
/// misreads signs like the «@» of an email, which the scanner then would not find.
pub fn read_picture(picture: &[u8], dpi: Option<u32>) -> Result<PageRead, OcrError> {
    let mut best = run(picture, dpi, false)?;
    let (mut picture, mut dpi) = (Cow::Borrowed(picture), dpi);
    if let Some((big, times)) = enlarged(&picture, best.letters) {
        let big_dpi = dpi.map(|d| (d as f32 * times).round() as u32);
        if let Ok(read) = run(&big, big_dpi, false) {
            if read.confidence >= best.confidence {
                (best, picture, dpi) = (read, Cow::Owned(big), big_dpi);
            }
        }
    }
    // a blank back of a sheet or one with only a stamp: reading it again cannot gain anything
    if !best.doubtful() || ink(&picture) < LITTLE_INK {
        return Ok(best);
    }
    // a sheet scanned on its side or upside down is put upright and read again
    if let Some(up) = orientation(&picture).and_then(|turn| turned(&picture, turn)) {
        if let Ok(read) = run(&up, dpi, false) {
            if read.confidence > best.confidence {
                (best, picture) = (read, Cow::Owned(up));
            }
        }
        if !best.doubtful() {
            return Ok(best);
        }
    }
    match run(&picture, dpi, true) {
        Ok(adaptive) if adaptive.confidence > best.confidence => Ok(adaptive),
        _ => Ok(best),
    }
}

/// How many times a picture is enlarged before it is read again, if at all: its letters were measured small, or
/// none could be measured in a small picture. No side grows past `MAX_SIDE`.
fn enlargement(letters: Option<f32>, longest: u32) -> Option<f32> {
    let times = match letters {
        Some(tall) if tall < SMALL_LETTERS => (LETTERS_TO / tall.max(1.0)).min(MAX_ENLARGE),
        None if longest < SMALL_PICTURE => UNREAD_ENLARGE,
        _ => return None,
    };
    let times = times.min(MAX_SIDE / longest.max(1) as f32);
    // a little bigger reads the same
    (times >= 1.25).then_some(times)
}

/// The picture enlarged, in gray as Tesseract reads it (PGM), and how many times, when its letters are small.
fn enlarged(picture: &[u8], letters: Option<f32>) -> Option<(Vec<u8>, f32)> {
    // first only its size, from the head of the file: a page with letters big enough is not opened
    let open = || image::ImageReader::new(std::io::Cursor::new(picture)).with_guessed_format().ok();
    let (w, h) = open()?.into_dimensions().ok().filter(|&(w, h)| w > 0 && h > 0)?;
    let times = enlargement(letters, w.max(h))?;
    let gray = gray_over_white(open()?.decode().ok()?);
    let (nw, nh) = ((w as f32 * times).round() as usize, (h as f32 * times).round() as usize);
    let mut out = format!("P5\n{nw} {nh}\n255\n").into_bytes();
    out.extend(enlarge_gray(gray.as_raw(), w as usize, h as usize, nw, nh));
    Some((out, times))
}

/// A picture in gray; what is transparent shows white, as Tesseract sees it.
fn gray_over_white(picture: image::DynamicImage) -> image::GrayImage {
    if !picture.color().has_alpha() {
        return picture.into_luma8();
    }
    let rgba = picture.into_rgba8();
    image::GrayImage::from_fn(rgba.width(), rgba.height(), |x, y| {
        let [r, g, b, a] = rgba.get_pixel(x, y).0.map(u32::from);
        let luma = (77 * r + 150 * g + 29 * b) >> 8;
        image::Luma([((luma * a + 255 * (255 - a)) / 255) as u8])
    })
}

/// Enlarges a gray picture of `w`×`h` to `nw`×`nh` with Lanczos (three lobes), first along its rows and then along its
/// columns. Done here and not with the `image` crate, which keeps four numbers per pixel in between: a drawn page
/// would take some 200 MB for each page read at the same time.
fn enlarge_gray(gray: &[u8], w: usize, h: usize, nw: usize, nh: usize) -> Vec<u8> {
    let pixel = |v: f32| v.round().clamp(0.0, 255.0) as u8;
    let across = weights(w, nw);
    let mut wide = Vec::with_capacity(nw * h);
    for row in gray.chunks_exact(w).take(h) {
        wide.extend(across.iter().map(|(first, k)| pixel(k.iter().zip(&row[*first..]).map(|(k, &p)| k * p as f32).sum())));
    }
    let mut out = Vec::with_capacity(nw * nh);
    let mut line = vec![0f32; nw];
    for (first, k) in weights(h, nh) {
        line.fill(0.0);
        for (j, k) in k.iter().enumerate() {
            for (sum, &p) in line.iter_mut().zip(&wide[(first + j) * nw..][..nw]) {
                *sum += k * p as f32;
            }
        }
        out.extend(line.iter().map(|&v| pixel(v)));
    }
    out
}

/// For each pixel of a side grown from `old` to `new` pixels (`new` ≥ `old`): the first old pixel it takes from and
/// how much each one counts, adding up to one.
fn weights(old: usize, new: usize) -> Vec<(usize, Vec<f32>)> {
    let lanczos = |x: f32| {
        if x.abs() < 1e-6 {
            1.0
        } else if x.abs() >= 3.0 {
            0.0
        } else {
            let px = std::f32::consts::PI * x;
            3.0 * px.sin() * (px / 3.0).sin() / (px * px)
        }
    };
    let scale = new as f32 / old as f32;
    (0..new)
        .map(|i| {
            let center = (i as f32 + 0.5) / scale - 0.5;
            let first = (center - 3.0).ceil().max(0.0) as usize;
            let last = ((center + 3.0).floor().max(0.0) as usize).min(old - 1);
            let mut k: Vec<f32> = (first..=last).map(|j| lanczos(j as f32 - center)).collect();
            let sum: f32 = k.iter().sum();
            k.iter_mut().for_each(|v| *v /= sum);
            (first, k)
        })
        .collect()
}

fn run(picture: &[u8], dpi: Option<u32>, adaptive: bool) -> Result<PageRead, OcrError> {
    // page segmentation 4 (one column of lines of any size): a table is read row by row; the automatic one (3) cut it
    // into a block per column. Two columns of prose came out the same with both (measured 2026-10-11)
    let mut args: Vec<String> = ["-l", LANGUAGES, "--oem", "1", "--psm", "4"].map(String::from).to_vec();
    if let Some(dpi) = dpi {
        args.extend(["--dpi".to_string(), dpi.to_string()]);
    }
    if adaptive {
        args.extend(["-c".to_string(), "thresholding_method=1".to_string()]);
    }
    args.push("tsv".into());
    Ok(from_tsv(&tesseract_on(picture, &args)?))
}

/// How many degrees a page has to turn clockwise to stand upright, when Tesseract is sure of it (its model of
/// orientation, `osd`): 90, 180 or 270; `None` for a page upright, or when it cannot tell.
fn orientation(picture: &[u8]) -> Option<u32> {
    let said = tesseract_on(picture, &["--psm".into(), "0".into()]).ok()?;
    let value = |label: &str| said.lines().find_map(|l| l.strip_prefix(label)).map(str::trim);
    let turn: u32 = value("Rotate:")?.parse().ok()?;
    let sure: f32 = value("Orientation confidence:")?.parse().ok()?;
    (matches!(turn, 90 | 180 | 270) && sure >= TURN_SURE).then_some(turn)
}

/// Runs Tesseract on a picture given through its standard input, with `args` after its data, and gives what it
/// writes. One thread each (the pages of a file are read side by side), below the normal priority: the computer is
/// given to whoever uses it, and Tesseract takes what is left (decision of the institution, 2026-10-13).
fn tesseract_on(picture: &[u8], args: &[String]) -> Result<String, OcrError> {
    let (exe, data) = tesseract().ok_or(OcrError::Missing)?;
    let mut cmd = Command::new(exe);
    cmd.args(["stdin", "stdout", "--tessdata-dir"]).arg(&data).args(args);
    cmd.env("OMP_THREAD_LIMIT", "1").stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        const BELOW_NORMAL_PRIORITY_CLASS: u32 = 0x0000_4000;
        cmd.creation_flags(CREATE_NO_WINDOW | BELOW_NORMAL_PRIORITY_CLASS);
    }
    let mut child = cmd.spawn().map_err(|e| OcrError::Failed(e.to_string()))?;
    let (Some(mut input), Some(mut output), Some(mut errors)) = (child.stdin.take(), child.stdout.take(), child.stderr.take()) else {
        return Err(OcrError::Failed("no pipes".into()));
    };
    let (status, out, said) = std::thread::scope(|s| {
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
    Ok(out)
}

/// A picture turned clockwise `turn` degrees (90, 180 or 270), in gray as Tesseract reads it (PGM).
fn turned(picture: &[u8], turn: u32) -> Option<Vec<u8>> {
    let gray = gray_over_white(image::load_from_memory(picture).ok()?);
    let up = match turn {
        90 => image::imageops::rotate90(&gray),
        180 => image::imageops::rotate180(&gray),
        270 => image::imageops::rotate270(&gray),
        _ => return None,
    };
    let mut out = format!("P5\n{} {}\n255\n", up.width(), up.height()).into_bytes();
    out.extend_from_slice(up.as_raw());
    Some(out)
}

/// How much of a picture is ink (0 to 1): the pixels clearly darker than its paper. The paper is its light gray (the
/// lightest tenth of its pixels), so a faded copy with gray letters still has ink. A blank back of a sheet has so
/// little that reading it again cannot gain anything.
fn ink(picture: &[u8]) -> f32 {
    let Some(gray) = image::load_from_memory(picture).ok().map(gray_over_white) else { return 1.0 };
    let px = gray.as_raw();
    if px.is_empty() {
        return 0.0;
    }
    let mut counts = [0usize; 256];
    for &p in px {
        counts[p as usize] += 1;
    }
    // the paper: the gray below which nine tenths of the pixels are
    let mut seen = 0;
    let paper = (0..=255u8).find(|&g| {
        seen += counts[g as usize];
        seen * 10 >= px.len() * 9
    });
    let dark = paper.unwrap_or(255).saturating_sub(INK_BELOW_PAPER);
    counts[..dark as usize].iter().sum::<usize>() as f32 / px.len() as f32
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
    let mut heights = Vec::new();
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
        // a sign or a lone letter does not say how tall the letters are
        if word.chars().filter(|c| c.is_alphanumeric()).count() >= 2 {
            heights.push(height as f32);
        }
    }
    heights.sort_by(f32::total_cmp);
    let tall = (heights.len() >= WORDS_TO_MEASURE).then(|| heights[heights.len() / 4]);
    PageRead { text, confidence: if letters > 0.0 { sum / letters } else { 0.0 }, letters: tall }
}

static PDFIUM: OnceLock<Option<Pdfium>> = OnceLock::new();

/// Pdfium, bound once: it draws the scanned pages here and takes out the text of every PDF in `pdf` (ADR-037).
pub(super) fn pdfium() -> Option<&'static Pdfium> {
    PDFIUM
        .get_or_init(|| {
            let dir = home()?.join("pdfium");
            Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path(&dir)).ok().map(Pdfium::new)
        })
        .as_ref()
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

    /// How tall the letters are is the lower quarter of the heights of the words: some words come as tall as their
    /// line. Signs and lone letters do not count, and a few words say nothing.
    #[test]
    fn the_letters_are_measured_by_the_lower_quarter_of_the_words() {
        let head = "level\tpage_num\tblock_num\tpar_num\tline_num\tword_num\tleft\ttop\twidth\theight\tconf\ttext\n";
        let words = |list: &[(&str, u32)]| {
            let rows: String = list.iter().enumerate().map(|(i, (w, h))| format!("5\t1\t1\t1\t1\t{i}\t{}\t0\t20\t{h}\t90\t{w}\n", i * 30)).collect();
            from_tsv(&format!("{head}{rows}"))
        };
        let read = words(&[("Folio", 9), ("de", 8), ("registro", 30), ("48213", 9), ("Casa", 31), ("Hogar", 30), ("a", 2), ("—", 1)]);
        assert_eq!(read.letters, Some(9.0));
        assert_eq!(words(&[("Casa", 9), ("Hogar", 9), ("del", 9), ("Valle", 9)]).letters, None);
    }

    /// Small letters are enlarged to about 30 pixels, four times at most; letters big enough, never. A small picture
    /// where nothing was measured is enlarged three times; a big one (a page, a photo of a phone) is not read again.
    #[test]
    fn a_picture_is_enlarged_only_when_its_letters_are_small() {
        assert_eq!(enlargement(Some(7.0), 520), Some(MAX_ENLARGE));
        assert_eq!(enlargement(Some(12.0), 1654), Some(2.5));
        assert_eq!(enlargement(Some(22.0), 520), None);
        assert_eq!(enlargement(Some(19.0), 4200), None, "no side past 5000 pixels leaves too little to gain");
        assert_eq!(enlargement(None, 440), Some(UNREAD_ENLARGE));
        assert_eq!(enlargement(None, 3300), None, "a blank page of a PDF is not read again");
        assert_eq!(enlargement(Some(10.0), 2000), Some(2.5), "no side past 5000 pixels");
    }

    /// A blank back of a sheet or one with only a stamp has too little ink to be read again; a faded page of text has
    /// enough (it is rescued by the second reading, ADR-035).
    #[test]
    fn a_blank_page_is_not_read_again_and_a_faded_one_is() {
        let pgm = |w: usize, h: usize, px: Vec<u8>| {
            let mut out = format!("P5\n{w} {h}\n255\n").into_bytes();
            out.extend(px);
            out
        };
        assert!(ink(&pgm(1000, 1400, vec![250; 1_400_000])) < LITTLE_INK);
        // a stamp of 60 by 40 pixels on a page of 1000 by 1400
        let mut stamp = vec![250u8; 1_400_000];
        for y in 0..40 {
            for x in 0..60 {
                stamp[y * 1000 + x] = 30;
            }
        }
        assert!(ink(&pgm(1000, 1400, stamp)) < LITTLE_INK);
        let pages = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../fixtures/office/acta-escaneada.pdf");
        if available() {
            let mut inks = vec![];
            draw_pages(&pages, &[1, 2], |_, picture, _| inks.push(ink(&picture))).unwrap();
            assert!(inks.iter().all(|i| *i >= LITTLE_INK), "{inks:?}");
        }
    }

    /// A picture turned a quarter clockwise is upright again, and its size swaps.
    #[test]
    fn a_turned_picture_is_put_upright() {
        let mut picture = b"P5\n3 2\n255\n".to_vec();
        picture.extend([1, 2, 3, 4, 5, 6]);
        let up = turned(&picture, 90).unwrap();
        assert_eq!(up, [b"P5\n2 3\n255\n".to_vec(), vec![4, 1, 5, 2, 6, 3]].concat());
        assert_eq!(turned(&picture, 45), None);
    }

    #[test]
    fn an_enlarged_picture_keeps_its_grays_and_its_edges() {
        let flat = enlarge_gray(&[200; 12], 4, 3, 13, 10);
        assert_eq!(flat, vec![200; 130]);
        // black then white: twice as wide, the edge stays in the middle and nothing goes past black or white
        let edge = enlarge_gray(&[0, 0, 0, 255, 255, 255], 6, 1, 12, 2);
        assert_eq!(edge.len(), 24);
        assert!(edge[..4].iter().all(|&v| v < 20) && edge[8..12].iter().all(|&v| v > 235), "{edge:?}");
        assert_eq!(edge[..12], edge[12..]);
        // a photo with a see-through background turns white there, not black
        let clear = image::DynamicImage::ImageRgba8(image::RgbaImage::from_pixel(2, 1, image::Rgba([0, 0, 0, 0])));
        assert_eq!(gray_over_white(clear).as_raw(), &vec![255, 255]);
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
