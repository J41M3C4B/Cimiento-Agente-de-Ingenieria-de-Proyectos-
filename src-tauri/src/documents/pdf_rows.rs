//! Table rows from where the letters sit on the page.
//!
//! The text extractor writes a page in the order its content stream draws it. A table is often drawn
//! column by column: all the labels, then all the values. The text then reads «Evaluación… Resultados…
//! Entrega…» and, further down, «octubre y noviembre… 1 semana de diciembre…», and no quote can join a
//! label with its value. The letters do carry their position: the ones that share a baseline are one
//! visual row, and a wide gap inside a row is a column break.
//!
//! This module only adds. It never replaces the page text: the rows it recovers are appended to the page
//! as a separate block, so a quote taken from either place is a literal piece of the page the model reads.
//! It knows nothing about any funder: only that letters on one baseline with wide gaps are cells of a row.

use super::pdf_grid;
use pdf_extract::{ColorSpace, MediaBox, OutputDev, OutputError, Path, Transform};

/// A cell longer than this is prose in a column, not the cell of a table.
const MAX_CELL_CHARS: usize = 70;
/// A gap wider than this many font sizes between two letters on one baseline starts a new cell.
const GAP_IN_FONT_SIZES: f64 = 1.5;
/// Two letters belong to the same baseline when their heights differ by less than this many font sizes.
const SAME_LINE_IN_FONT_SIZES: f64 = 0.5;
/// A table needs at least this many lines with columns; one line with a wide gap is just a line.
const MIN_ROWS: usize = 2;
/// Cells of running prose are whole lines of text, long on average; the cells of a table are short.
const MAX_MEAN_CELL_CHARS: f64 = 32.0;
/// Lines closer than this many font sizes belong to the same row of a table.
const BAND_GAP_IN_FONT_SIZES: f64 = 1.3;
/// A row of a table has at most this many lines (a value written over several lines).
const MAX_LINES_IN_A_ROW: usize = 5;
/// A table ends where the next line is farther than this many font sizes from the last.
const MAX_ROW_GAP_IN_FONT_SIZES: f64 = 3.0;

/// Lines of a column that make running prose are at least this long (the middle one of them; a table's cells are shorter,
/// and a few short lines, like a heading, do not make a column a table).
const PROSE_MIN_LINE_CHARS: usize = 22;
/// A column is running prose when at least this share of its lines go on with the sentence of the line above.
const PROSE_CONTINUING_SHARE: f64 = 0.5;

/// A row seen on this many pages is a running header or footer.
const RUNNING_MIN_PAGES: usize = 3;

/// Header of the block appended to a page.
pub const BLOCK_TITLE: &str = "[Filas de tabla reconstruidas por posición]";

/// Which way a letter is written on the page.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dir {
    /// Left to right, the way a line is read.
    Across,
    /// Turned a quarter to the left: read from the bottom of the page to the top (the label of a table's column).
    Up,
    /// Turned a quarter to the right: read from the top down.
    Down,
    /// Upside down or slanted: never part of a line.
    Other,
}

#[derive(Debug, Clone)]
pub struct Glyph {
    pub x: f64,
    pub end: f64,
    pub y: f64,
    pub size: f64,
    pub text: String,
    pub dir: Dir,
    /// The centre of the letter on the page (y grows downward): what says which cell or table it is in.
    pub cx: f64,
    pub cy: f64,
}

impl Glyph {
    /// A letter written across the page, `x` to `end` on the baseline `y`.
    #[cfg_attr(not(test), allow(dead_code))] // used by the measurement tests, not by the application
    pub fn across(x: f64, end: f64, y: f64, size: f64, text: &str) -> Glyph {
        Glyph { x, end, y, size, text: text.to_string(), dir: Dir::Across, cx: (x + end) / 2.0, cy: y - 0.3 * size }
    }
}

/// Collects the position of every letter of a page; draws nothing.
#[derive(Default)]
pub struct GlyphCollector {
    flip: Option<Transform>,
    pub pages: Vec<Vec<Glyph>>,
    /// `(width, height)` of each page, in the same order as `pages`.
    pub sizes: Vec<(f64, f64)>,
}

impl OutputDev for GlyphCollector {
    fn begin_page(&mut self, _page_num: u32, media_box: &MediaBox, _art_box: Option<(f64, f64, f64, f64)>) -> Result<(), OutputError> {
        self.flip = Some(Transform::row_major(1., 0., 0., -1., 0., media_box.ury - media_box.lly));
        self.pages.push(Vec::new());
        self.sizes.push((media_box.urx - media_box.llx, media_box.ury - media_box.lly));
        Ok(())
    }
    fn end_page(&mut self) -> Result<(), OutputError> {
        Ok(())
    }
    fn output_character(&mut self, trm: &Transform, width: f64, _spacing: f64, font_size: f64, char: &str) -> Result<(), OutputError> {
        let Some(flip) = self.flip else { return Ok(()) };
        let p = trm.post_transform(&flip);
        // the size of one side of the square with the same area as the font box after the transform
        let (vx, vy) = (font_size * (trm.m11 + trm.m21), font_size * (trm.m12 + trm.m22));
        let size = (vx * vy).abs().sqrt();
        // the direction the baseline runs, in the page's own axes (y grows upward there)
        let dir = if trm.m11 > 0.0 && trm.m12.abs() <= 0.2 * trm.m11 {
            Dir::Across
        } else if trm.m12 > 0.0 && trm.m11.abs() <= 0.2 * trm.m12 {
            Dir::Up
        } else if trm.m12 < 0.0 && trm.m11.abs() <= 0.2 * trm.m12.abs() {
            Dir::Down
        } else {
            Dir::Other
        };
        let advance = width * size;
        let (cx, cy) = match dir {
            // the letter rises to the left of the baseline when turned left, to the right when turned right
            Dir::Up => (p.m31 - 0.3 * size, p.m32 - advance / 2.0),
            Dir::Down => (p.m31 + 0.3 * size, p.m32 + advance / 2.0),
            _ => (p.m31 + advance / 2.0, p.m32 - 0.3 * size),
        };
        if let Some(page) = self.pages.last_mut() {
            page.push(Glyph { x: p.m31, end: p.m31 + advance, y: p.m32, size, text: char.to_string(), dir, cx, cy });
        }
        Ok(())
    }
    fn begin_word(&mut self) -> Result<(), OutputError> {
        Ok(())
    }
    fn end_word(&mut self) -> Result<(), OutputError> {
        Ok(())
    }
    fn end_line(&mut self) -> Result<(), OutputError> {
        Ok(())
    }
    fn stroke(&mut self, _ctm: &Transform, _colorspace: &ColorSpace, _color: &[f64], _path: &Path) -> Result<(), OutputError> {
        Ok(())
    }
    fn fill(&mut self, _ctm: &Transform, _colorspace: &ColorSpace, _color: &[f64], _path: &Path) -> Result<(), OutputError> {
        Ok(())
    }
}

/// A run of letters on one baseline with no wide gap inside.
#[derive(Debug, Clone)]
struct Piece {
    x: f64,
    end: f64,
    text: String,
}

/// One visual line: its pieces from left to right.
#[derive(Debug, Clone)]
struct Line {
    y: f64,
    size: f64,
    pieces: Vec<Piece>,
}

/// The text of a cell: its lines from top to bottom, the pieces of each line from left to right, all in one line.
pub(super) fn cell_text(glyphs: &[Glyph]) -> String {
    let across = squash(&visual_lines(glyphs).iter().map(|l| l.pieces.iter().map(|p| p.text.as_str()).collect::<Vec<_>>().join(" ")).collect::<Vec<_>>().join(" "));
    let turned: Vec<String> = [Dir::Up, Dir::Down].iter().filter_map(|d| turned_text(glyphs, *d)).collect();
    if turned.is_empty() {
        across
    } else {
        squash(&format!("{} {across}", turned.join(" ")))
    }
}

/// The text written turned a quarter (a column label running up the page): the letters that share a baseline
/// are one line, read along the page from the bottom up (turned left) or from the top down (turned right), and
/// the lines follow one another to the right (turned left) or to the left (turned right).
fn turned_text(glyphs: &[Glyph], dir: Dir) -> Option<String> {
    let mut letters: Vec<&Glyph> = glyphs.iter().filter(|g| g.dir == dir && g.size > 0.0 && g.x.is_finite() && g.y.is_finite()).collect();
    if letters.is_empty() {
        return None;
    }
    letters.sort_by(|a, b| a.x.total_cmp(&b.x));
    let mut lines: Vec<Vec<&Glyph>> = Vec::new();
    for g in letters {
        match lines.last_mut() {
            Some(l) if (g.x - l[0].x).abs() <= g.size * SAME_LINE_IN_FONT_SIZES => l.push(g),
            _ => lines.push(vec![g]),
        }
    }
    if dir == Dir::Down {
        lines.reverse();
    }
    let text: Vec<String> = lines
        .into_iter()
        .map(|mut l| {
            if dir == Dir::Up {
                l.sort_by(|a, b| b.y.total_cmp(&a.y));
            } else {
                l.sort_by(|a, b| a.y.total_cmp(&b.y));
            }
            l.iter().map(|g| g.text.as_str()).collect::<String>()
        })
        .collect();
    Some(squash(&text.join(" "))).filter(|t| !t.is_empty())
}

fn squash(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// The visual lines of a page, each cut into pieces at the wide gaps.
fn visual_lines(glyphs: &[Glyph]) -> Vec<Line> {
    let mut sorted: Vec<&Glyph> = glyphs.iter().filter(|g| g.dir == Dir::Across && g.size > 0.0 && g.y.is_finite() && g.x.is_finite()).collect();
    sorted.sort_by(|a, b| a.y.total_cmp(&b.y).then(a.x.total_cmp(&b.x)));
    let mut groups: Vec<Vec<&Glyph>> = Vec::new();
    for g in sorted {
        match groups.last_mut() {
            Some(line) if (g.y - line[0].y).abs() <= g.size * SAME_LINE_IN_FONT_SIZES => line.push(g),
            _ => groups.push(vec![g]),
        }
    }
    groups
        .into_iter()
        .map(|mut line| {
            line.sort_by(|a, b| a.x.total_cmp(&b.x));
            let y = line.iter().map(|g| g.y).sum::<f64>() / line.len() as f64;
            let size = line[0].size;
            let mut pieces: Vec<Piece> = Vec::new();
            let mut last_end = f64::MIN;
            for g in line {
                if pieces.is_empty() || g.x - last_end > g.size * GAP_IN_FONT_SIZES {
                    pieces.push(Piece { x: g.x, end: g.end, text: String::new() });
                }
                if let Some(c) = pieces.last_mut() {
                    c.text.push_str(&g.text);
                    c.end = c.end.max(g.end);
                }
                last_end = last_end.max(g.end);
            }
            for c in &mut pieces {
                c.text = squash(&c.text);
            }
            pieces.retain(|c| !c.text.is_empty());
            Line { y, size, pieces }
        })
        .filter(|l| !l.pieces.is_empty())
        .collect()
}

/// A line with two or more short pieces: where a table shows its columns.
fn is_anchor(line: &Line) -> bool {
    line.pieces.len() >= 2 && line.pieces.iter().all(|p| p.text.chars().count() <= MAX_CELL_CHARS)
}

fn median(mut v: Vec<f64>) -> f64 {
    v.sort_by(f64::total_cmp);
    v[v.len() / 2]
}

/// Where the columns of a table split: for the commonest number of columns among its anchor lines, the
/// middle of the gap between each pair of neighbours, taken as the median over those lines.
fn column_splits(anchors: &[&Line]) -> Vec<f64> {
    let mut counts: Vec<(usize, usize)> = Vec::new();
    for a in anchors {
        match counts.iter_mut().find(|(n, _)| *n == a.pieces.len()) {
            Some((_, c)) => *c += 1,
            None => counts.push((a.pieces.len(), 1)),
        }
    }
    let Some(&(columns, _)) = counts.iter().max_by_key(|(n, c)| (*c, *n)) else { return Vec::new() };
    (0..columns - 1)
        .map(|g| median(anchors.iter().filter(|a| a.pieces.len() == columns).map(|a| (a.pieces[g].end + a.pieces[g + 1].x) / 2.0).collect()))
        .collect()
}

fn column_of(piece: &Piece, splits: &[f64]) -> usize {
    let centre = (piece.x + piece.end) / 2.0;
    splits.iter().filter(|s| centre >= **s).count()
}

fn crosses(piece: &Piece, splits: &[f64]) -> bool {
    splits.iter().any(|s| piece.x < *s && piece.end > *s)
}

/// «a)», «b) c) d)», «1.»: the margin of a list, not a cell.
fn is_list_marker(cell: &str) -> bool {
    cell.split_whitespace().all(|t| t.chars().count() <= 3 && (t.ends_with(')') || t.ends_with('.')))
}

/// The line goes on with the sentence of the one above: the one above does not end a sentence and this one starts
/// in lower case, or the one above ends in a hyphen.
fn continues(prev: &str, next: &str) -> bool {
    let end = prev.trim_end().chars().last();
    let sentence_end = matches!(end, Some('.' | ':' | ';' | '?' | '!'));
    let starts_lower = next.trim_start().chars().next().is_some_and(char::is_lowercase);
    (!sentence_end && starts_lower) || end == Some('-')
}

/// A page laid out in columns of running text, whose lines only look like cells because of the gap between the
/// columns: in every column with enough lines, the lines are long and go on one with the next.
fn is_prose_columns(run: &[&Line], splits: &[f64]) -> bool {
    let mut cols: Vec<Vec<&str>> = vec![Vec::new(); splits.len() + 1];
    for l in run {
        for p in &l.pieces {
            cols[column_of(p, splits)].push(p.text.as_str());
        }
    }
    let mut judged = 0;
    for c in cols.iter().filter(|c| c.len() >= 4) {
        let pairs = c.windows(2).count();
        let continuing = c.windows(2).filter(|w| continues(w[0], w[1])).count();
        let mut lengths: Vec<usize> = c.iter().map(|t| t.chars().count()).collect();
        lengths.sort_unstable();
        if lengths[lengths.len() / 2] < PROSE_MIN_LINE_CHARS || (continuing as f64) < PROSE_CONTINUING_SHARE * pairs as f64 {
            return false;
        }
        judged += 1;
    }
    judged >= 2
}

/// The rows of one table run: lines are grouped into bands by the vertical gaps between them (a cell
/// whose text is centred in a tall row does not share a baseline with its neighbours, but it does share
/// the band), and each band is cut into columns at the splits.
fn rows_of_run(run: &[&Line], splits: &[f64], one_band: bool) -> Vec<String> {
    let mut bands: Vec<Vec<&Line>> = Vec::new();
    if one_band {
        // columns of running text: each column is one cell, read from its first line to its last
        bands.push(run.to_vec());
    } else {
        for l in run {
            match bands.last_mut() {
                Some(b) if l.y - b.last().map_or(l.y, |p| p.y) <= l.size * BAND_GAP_IN_FONT_SIZES => b.push(l),
                _ => bands.push(vec![l]),
            }
        }
        // a «row» of many lines is a block of running text that happens to sit in columns, not a table
        if bands.iter().any(|b| b.len() > MAX_LINES_IN_A_ROW) {
            return Vec::new();
        }
    }
    bands
        .iter()
        .filter_map(|band| {
            let mut cols: Vec<String> = vec![String::new(); splits.len() + 1];
            for l in band {
                for p in &l.pieces {
                    let c = &mut cols[column_of(p, splits)];
                    if !c.is_empty() {
                        c.push(' ');
                    }
                    c.push_str(&p.text);
                }
            }
            let cells: Vec<&str> = cols.iter().map(String::as_str).filter(|c| !c.is_empty()).collect();
            (cells.len() >= 2 && !is_list_marker(cells[0])).then(|| cells.join(" | "))
        })
        .collect()
}

/// The rows of the tables of a page, one string per row, cells joined by « | ». A table starts at a line
/// with two or more short pieces and goes on through the lines below it while they are either the same
/// kind of line or a single piece that sits inside one column; a line that runs across a column break is
/// prose and ends it. Columns of running prose are not tables: their pieces are whole lines of text.
pub fn table_rows(glyphs: &[Glyph]) -> Vec<String> {
    let lines = visual_lines(glyphs);
    let mut out = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        if !is_anchor(&lines[i]) {
            i += 1;
            continue;
        }
        let mut run: Vec<&Line> = vec![&lines[i]];
        let mut anchors: Vec<&Line> = vec![&lines[i]];
        let mut j = i + 1;
        while j < lines.len() {
            let (prev, line) = (run[run.len() - 1], &lines[j]);
            if line.y - prev.y > line.size * MAX_ROW_GAP_IN_FONT_SIZES {
                break;
            }
            if is_anchor(line) {
                anchors.push(line);
            } else if !(line.pieces.len() == 1 && line.pieces[0].text.chars().count() <= MAX_CELL_CHARS && !crosses(&line.pieces[0], &column_splits(&anchors))) {
                break;
            }
            run.push(line);
            j += 1;
        }
        let cells: Vec<usize> = anchors.iter().flat_map(|a| a.pieces.iter().map(|p| p.text.chars().count())).collect();
        let mean = cells.iter().sum::<usize>() as f64 / cells.len().max(1) as f64;
        if anchors.len() >= MIN_ROWS {
            let splits = column_splits(&anchors);
            if is_prose_columns(&run, &splits) {
                out.extend(rows_of_run(&run, &splits, true));
            } else if mean <= MAX_MEAN_CELL_CHARS {
                out.extend(rows_of_run(&run, &splits, false));
            }
        }
        i = j;
    }
    out
}

/// The rows of the tables of a page: first the ones that have their borders drawn (the lines say where the
/// cells are, whatever they hold), then, among the letters that are left, the ones the layout of the text
/// shows as a table.
pub fn page_rows(glyphs: &[Glyph], rules: &[pdf_grid::Rule], height: f64) -> Vec<String> {
    let tables = pdf_grid::grid_tables(rules, glyphs, height);
    let rest: Vec<Glyph> = if tables.is_empty() {
        glyphs.to_vec()
    } else {
        glyphs.iter().filter(|g| !tables.iter().any(|t| t.contains(g.cx, g.cy))).cloned().collect()
    };
    let mut out: Vec<String> = tables.into_iter().flat_map(|t| t.rows).collect();
    out.extend(table_rows(&rest));
    out
}

/// A row that comes back on many pages (a header or a footer with its page number) is not a table of the
/// call: it is dropped from every page. Digits are ignored so that «página 3» and «página 4» are the same.
fn without_running_rows(mut pages: Vec<Vec<String>>) -> Vec<Vec<String>> {
    if pages.len() < RUNNING_MIN_PAGES {
        return pages;
    }
    let key = |row: &str| row.chars().filter(|c| !c.is_ascii_digit()).collect::<String>();
    let mut seen: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for page in &pages {
        let mut once: Vec<String> = page.iter().map(|r| key(r)).collect();
        once.sort();
        once.dedup();
        for k in once {
            *seen.entry(k).or_default() += 1;
        }
    }
    for page in &mut pages {
        page.retain(|r| seen.get(&key(r)).copied().unwrap_or(0) < RUNNING_MIN_PAGES);
    }
    pages
}

/// The rows of every page of a PDF, page by page. Best effort: a file the second pass cannot read gives
/// no rows, never an error, because the plain text is already there.
pub fn rows_by_page(path: &std::path::Path) -> Vec<Vec<String>> {
    let path = path.to_path_buf();
    std::panic::catch_unwind(move || {
        let mut doc = pdf_extract::Document::load(&path).ok()?;
        if doc.is_encrypted() && doc.decrypt("").is_err() {
            return None;
        }
        let mut collector = GlyphCollector::default();
        let mut ids = Vec::new();
        for (page_num, page_id) in doc.get_pages() {
            // one page that fails does not take the others with it: it gets an empty entry
            let before = collector.pages.len();
            if pdf_extract::output_doc_page(&doc, &mut collector, page_num).is_err() && collector.pages.len() == before {
                collector.pages.push(Vec::new());
                collector.sizes.push((0.0, 0.0));
            }
            ids.push(page_id);
        }
        let rows = collector
            .pages
            .iter()
            .zip(&collector.sizes)
            .zip(&ids)
            .map(|((glyphs, size), id)| {
                // a page whose lines cannot be read still has its rows from the layout of the text
                let rules = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| pdf_grid::page_rules(&doc, *id, *size))).unwrap_or_default();
                page_rows(glyphs, &rules, size.1)
            })
            .collect();
        Some(without_running_rows(rows))
    })
    .ok()
    .flatten()
    .unwrap_or_default()
}

/// The page text with its table rows appended as a block of their own, if it has any.
pub fn with_rows(text: &str, rows: &[String]) -> String {
    if rows.is_empty() {
        text.to_string()
    } else {
        format!("{text}\n\n{BLOCK_TITLE}\n{}", rows.join("\n"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// One word as letters of a fixed width, starting at `x` on the baseline `y`.
    fn word(x: f64, y: f64, text: &str) -> Vec<Glyph> {
        text.chars().enumerate().map(|(i, c)| Glyph::across(x + i as f64 * 5.0, x + (i as f64 + 1.0) * 5.0, y, 10.0, &c.to_string())).collect()
    }

    fn phrase(x: f64, y: f64, text: &str) -> Vec<Glyph> {
        word(x, y, text)
    }

    #[test]
    fn a_table_drawn_column_by_column_is_read_row_by_row() {
        // the labels first, then the values: the order in which the content stream draws them
        let mut g = Vec::new();
        for (i, label) in ["Evaluación de proyectos", "Resultados", "Entrega de recursos"].iter().enumerate() {
            g.extend(phrase(50.0, 100.0 + i as f64 * 20.0, label));
        }
        for (i, value) in ["octubre y noviembre", "1 semana de diciembre de 2026", "enero 2027"].iter().enumerate() {
            g.extend(phrase(300.0, 100.0 + i as f64 * 20.0, value));
        }
        assert_eq!(
            table_rows(&g),
            vec!["Evaluación de proyectos | octubre y noviembre", "Resultados | 1 semana de diciembre de 2026", "Entrega de recursos | enero 2027"]
        );
    }

    #[test]
    fn a_value_written_on_two_lines_continues_its_cell() {
        let mut g = Vec::new();
        g.extend(phrase(50.0, 100.0, "Registro"));
        g.extend(phrase(300.0, 100.0, "agosto y septiembre"));
        g.extend(phrase(50.0, 120.0, "Entrega de recursos"));
        g.extend(phrase(300.0, 120.0, "enero 2027 alimentación"));
        g.extend(phrase(300.0, 132.0, "febrero 2027 educación"));
        let rows = table_rows(&g);
        assert_eq!(rows, vec!["Registro | agosto y septiembre", "Entrega de recursos | enero 2027 alimentación febrero 2027 educación"]);
    }

    /// One word as letters of a given size, starting at `x` on the baseline `y`.
    fn sized(x: f64, y: f64, size: f64, text: &str) -> Vec<Glyph> {
        text.chars().enumerate().map(|(i, c)| Glyph::across(x + i as f64 * 5.0, x + (i as f64 + 1.0) * 5.0, y, size, &c.to_string())).collect()
    }

    #[test]
    fn cells_centred_in_tall_rows_are_read_as_one_row() {
        // the geometry of a real call's calendar: the label of a row sits between the lines of its value,
        // and each cell is centred in its column, so no two cells of a row share a baseline
        let mut g = Vec::new();
        let rows = [
            (100.0, 100.0, 50.0, 300.0, "Etapa", "Fechas"),
            (122.0, 122.0, 40.0, 290.0, "Lanzamiento de convocatoria", "julio 2026"),
            (160.0, 160.0, 60.0, 280.0, "Registro y preselección", "agosto y septiembre 2026"),
            (195.0, 197.0, 62.0, 290.0, "Evaluación de proyectos", "octubre y noviembre"),
        ];
        for (ly, vy, lx, vx, label, value) in rows {
            g.extend(sized(lx, ly, 12.0, label));
            g.extend(sized(vx, vy, 11.0, value));
        }
        // «Resultados»: the value is a little above its label
        g.extend(sized(100.0, 232.0, 12.0, "Resultados"));
        g.extend(sized(290.0, 226.0, 11.0, "1 semana de diciembre de 2026"));
        // «Entrega»: the label sits between the two lines of its value
        g.extend(sized(50.0, 260.0, 12.0, "Entrega de recursos e Inicio"));
        g.extend(sized(280.0, 250.0, 11.0, "enero 2027 alimentación"));
        g.extend(sized(280.0, 263.0, 11.0, "febrero 2027 educación"));
        // prose right below: it runs across the column break and does not belong to the table
        g.extend(sized(50.0, 330.0, 12.0, "Los proyectos serán evaluados por un comité independiente de especialistas."));
        let rows = table_rows(&g);
        assert_eq!(
            rows,
            vec![
                "Etapa | Fechas",
                "Lanzamiento de convocatoria | julio 2026",
                "Registro y preselección | agosto y septiembre 2026",
                "Evaluación de proyectos | octubre y noviembre",
                "Resultados | 1 semana de diciembre de 2026",
                "Entrega de recursos e Inicio | enero 2027 alimentación febrero 2027 educación",
            ]
        );
    }

    #[test]
    fn lists_with_a_letter_marker_and_running_headers_are_not_tables() {
        let mut g = Vec::new();
        for (i, t) in ["Atiendan a personas indígenas", "Demuestren mayor cobertura", "Atiendan a víctimas"].iter().enumerate() {
            g.extend(phrase(50.0, 100.0 + i as f64 * 14.0, &["a)", "b)", "c)"][i]));
            g.extend(phrase(90.0, 100.0 + i as f64 * 14.0, t));
        }
        assert!(table_rows(&g).is_empty(), "{:?}", table_rows(&g));
        // the same two-line header on every page is dropped; a table that only one page has stays
        let header = vec!["sitio.gob.mx | GACETA".to_string(), "otro.gob.mx | 7 | DEL GOBIERNO".to_string()];
        let mut pages: Vec<Vec<String>> = (0..4).map(|i| vec![header[0].clone(), header[1].replace('7', &i.to_string())]).collect();
        pages[2].push("Etapa | Fechas".to_string());
        let kept = without_running_rows(pages);
        assert_eq!(kept.iter().map(Vec::len).collect::<Vec<_>>(), vec![0, 0, 1, 0]);
        assert_eq!(kept[2], vec!["Etapa | Fechas"]);
    }

    #[test]
    fn prose_and_a_lone_wide_gap_are_not_tables() {
        let mut g = Vec::new();
        // a justified paragraph: words close together, no cell breaks
        g.extend(phrase(50.0, 100.0, "El apoyo se entrega por una sola vez a cada organización"));
        g.extend(phrase(50.0, 112.0, "seleccionada y debe comprobarse en ciento veinte días"));
        // one line with a wide gap (a title and a page number): a single row is not a table
        g.extend(phrase(50.0, 300.0, "Reglas de operación"));
        g.extend(phrase(500.0, 300.0, "12"));
        assert!(table_rows(&g).is_empty(), "{:?}", table_rows(&g));
    }

    #[test]
    fn long_cells_that_do_not_go_on_with_one_another_are_not_a_table() {
        let long = "una columna de texto corrido que ocupa casi todo el ancho de la columna del documento impreso";
        let mut g = Vec::new();
        for i in 0..3 {
            g.extend(phrase(50.0, 100.0 + i as f64 * 12.0, long));
            g.extend(phrase(700.0, 100.0 + i as f64 * 12.0, long));
        }
        assert!(table_rows(&g).is_empty());
    }

    #[test]
    fn columns_of_running_text_are_read_one_column_at_a_time_and_not_line_by_line_across() {
        // a page in two columns: the lines of each column go on with the sentence above them
        let left = ["El programa apoya proyectos de atencion a las personas", "mayores que viven en asilos y casas hogar, con", "prioridad en los que atienden a quienes no tienen", "red familiar ni ingresos propios para su sustento", "diario y que requieren cuidados permanentes."];
        let right = ["Las organizaciones deberan presentar su solicitud", "antes de la fecha de cierre y adjuntar los documentos", "que se piden en esta convocatoria, firmados por su", "representante legal y con la informacion completa", "sobre el proyecto que desean realizar este anio."];
        let mut g = Vec::new();
        for i in 0..5 {
            g.extend(phrase(50.0, 100.0 + i as f64 * 12.0, left[i]));
            g.extend(phrase(450.0, 100.0 + i as f64 * 12.0, right[i]));
        }
        let rows = table_rows(&g);
        assert_eq!(rows.len(), 1, "{rows:?}");
        assert_eq!(rows[0], format!("{} | {}", left.join(" "), right.join(" ")));
    }

    #[test]
    fn short_lowercase_cells_are_a_table_and_not_prose() {
        let mut g = Vec::new();
        for (i, (a, b)) in [("agua", "luz"), ("gas", "internet"), ("renta", "seguros"), ("papeleria", "viaticos")].iter().enumerate() {
            g.extend(phrase(50.0, 100.0 + i as f64 * 20.0, a));
            g.extend(phrase(300.0, 100.0 + i as f64 * 20.0, b));
        }
        assert_eq!(table_rows(&g), vec!["agua | luz", "gas | internet", "renta | seguros", "papeleria | viaticos"]);
    }

    #[test]
    fn the_rows_go_after_the_page_text_in_a_block_of_their_own() {
        let t = with_rows("Texto de la página.", &["a | b".to_string(), "c | d".to_string()]);
        assert_eq!(t, format!("Texto de la página.\n\n{BLOCK_TITLE}\na | b\nc | d"));
        assert_eq!(with_rows("Texto.", &[]), "Texto.");
    }

    #[test]
    fn a_file_that_is_not_a_pdf_gives_no_rows_and_no_error() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("x.pdf");
        std::fs::write(&p, b"this is not a pdf").unwrap();
        assert!(rows_by_page(&p).is_empty());
    }
}

#[cfg(test)]
mod probe {
    use super::*;

    /// Prints the cells of every visual line of one page of a real PDF: `CIMIENTO_ROWS_PDF`, `CIMIENTO_ROWS_PAGE`.
    #[test]
    #[ignore]
    fn print_lines_of_a_page() {
        let path = std::env::var("CIMIENTO_ROWS_PDF").expect("CIMIENTO_ROWS_PDF");
        let page: usize = std::env::var("CIMIENTO_ROWS_PAGE").expect("CIMIENTO_ROWS_PAGE").parse().unwrap();
        let doc = pdf_extract::Document::load(&path).unwrap();
        let mut c = GlyphCollector::default();
        for (n, _) in doc.get_pages() {
            pdf_extract::output_doc_page(&doc, &mut c, n).unwrap();
        }
        let g = &c.pages[page - 1];
        for line in visual_lines(g) {
            println!("y{:.1} {}", line.y, line.pieces.iter().map(|c| format!("[{:.0}] {}", c.x, c.text)).collect::<Vec<_>>().join("  ||  "));
        }
    }

    /// For every page of a real PDF: how many tables the lines give and how many rows the layout of the text gives
    /// in what is left. `CIMIENTO_ROWS_PDF` names the file; `CIMIENTO_ROWS_SHOW=1` prints the rows of the lines.
    #[test]
    #[ignore]
    fn count_tables_by_kind() {
        let path = std::env::var("CIMIENTO_ROWS_PDF").expect("CIMIENTO_ROWS_PDF");
        let show = std::env::var("CIMIENTO_ROWS_SHOW").is_ok();
        let doc = pdf_extract::Document::load(&path).unwrap();
        let mut c = GlyphCollector::default();
        let mut ids = Vec::new();
        for (n, id) in doc.get_pages() {
            pdf_extract::output_doc_page(&doc, &mut c, n).unwrap();
            ids.push(id);
        }
        let (mut grid_total, mut text_total) = (0, 0);
        for (i, id) in ids.iter().enumerate() {
            let rules = pdf_grid::page_rules(&doc, *id, c.sizes[i]);
            let candidates = pdf_grid::grid_candidates(&rules, &c.pages[i], c.sizes[i].1);
            let tables = pdf_grid::grid_tables(&rules, &c.pages[i], c.sizes[i].1);
            let rows = page_rows(&c.pages[i], &rules, c.sizes[i].1);
            for (t, f) in candidates.iter().filter(|(_, f)| !f.is_data()) {
                println!("  página {}: RECHAZADA como diseño {:?}: {} celdas, {} con texto, {} letras · {:?}", i + 1, t.bounds, f.cells, f.filled, f.letters, t.rows.iter().take(2).map(|r| r.chars().take(50).collect::<String>()).collect::<Vec<_>>());
            }
            let by_grid: usize = tables.iter().map(|t| t.rows.len()).sum();
            let by_text = rows.len() - by_grid;
            grid_total += tables.len();
            text_total += usize::from(by_text > 0);
            if !tables.is_empty() || by_text > 0 {
                println!("página {:>3}: {:>5} líneas · {} tablas con bordes ({} filas) · {} filas por el texto", i + 1, rules.len(), tables.len(), by_grid, by_text);
            }
            if show {
                for t in &tables {
                    println!("  [tabla {:?}]", t.bounds);
                    for r in &t.rows {
                        println!("    {}", r.chars().take(200).collect::<String>());
                    }
                }
            }
        }
        println!("TOTAL: {} páginas, {} tablas con bordes, {} páginas con filas por el texto", ids.len(), grid_total, text_total);
    }
}
