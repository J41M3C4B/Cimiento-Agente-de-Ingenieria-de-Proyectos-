//! Tables drawn with ruling lines: the cells are where the lines say they are.
//!
//! `pdf_rows` finds a table from how its text is laid out, which fails when the cells hold whole
//! paragraphs or when the columns are not aligned the way it expects. A table that has its borders drawn
//! does not need guessing: the lines of the page are read (thin rectangles filled with ink, stroked
//! segments, boxes), the ones that touch each other make a grid, and each letter belongs to the cell that
//! contains it. Long cells, many lines in a cell, merged cells and columns of any width all come out right
//! because the geometry, and not the text, decides what a cell is.
//!
//! Like `pdf_rows`, this only adds: the rows are appended to the page as a block of their own and the
//! page text is never changed. It knows nothing about any funder or layout, only about lines and letters.

use super::pdf_rows::{cell_text, Dir, Glyph};
use lopdf::content::Operation;
use lopdf::{Dictionary, Document, Object, ObjectId};

/// A rectangle thinner than this (in points) is a ruling line drawn as ink, not a box.
const THIN: f64 = 2.5;
/// Two lines closer than this are the same line; a line that ends this close to another touches it.
const TOUCH: f64 = 2.0;
/// Gap between two pieces of one drawn line that is still one line.
const JOIN_GAP: f64 = 3.0;
/// Rules per page above which the page is vector art (a map, a chart), not a table.
const MAX_RULES: usize = 6_000;
/// A cell repeated in every row it spans only if its text is this short.
const MAX_REPEATED_CHARS: usize = 120;
/// A grid is a table of data only if at least this many of its cells hold something...
const MIN_FILLED_CELLS: usize = 4;
/// ...at least a quarter of its cells are written in (a page footer or a frame around a page is made of
/// ruling lines and boxes too, and is almost all empty)...
const MIN_FILLED_SHARE: f64 = 0.25;
/// ...and the writing in it has at least this many letters (numbers alone are a chart's axis, a page number).
const MIN_LETTERS: usize = 8;
/// A label turned a quarter is often longer than the cell it belongs to and runs past the table's last line: its
/// letters count for the nearest cell if they are no farther than this from the table.
const TURNED_OVERFLOW: f64 = 120.0;
/// Nested forms are followed this deep.
const MAX_FORM_DEPTH: usize = 3;

/// An axis-aligned line: `at` is its `y` when horizontal and its `x` when vertical.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rule {
    pub horizontal: bool,
    pub at: f64,
    pub from: f64,
    pub to: f64,
}

/// How much of a grid is written in, to tell a table of data from the boxes of a page's design.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Fill {
    pub cells: usize,
    pub filled: usize,
    pub letters: usize,
}

impl Fill {
    pub fn is_data(&self) -> bool {
        self.filled >= MIN_FILLED_CELLS && self.filled as f64 >= MIN_FILLED_SHARE * self.cells as f64 && self.letters >= MIN_LETTERS
    }
}

/// The cells of one table, row by row.
#[derive(Debug, Clone, PartialEq)]
pub struct GridTable {
    /// `(left, top, right, bottom)` in page coordinates, top first (y grows downward).
    pub bounds: (f64, f64, f64, f64),
    pub rows: Vec<String>,
}

impl GridTable {
    pub fn contains(&self, x: f64, y: f64) -> bool {
        x >= self.bounds.0 - TOUCH && x <= self.bounds.2 + TOUCH && y >= self.bounds.1 - TOUCH && y <= self.bounds.3 + TOUCH
    }
}

// ------------------------------------------------------------------ reading the lines of a page

type Matrix = [f64; 6];
const IDENTITY: Matrix = [1.0, 0.0, 0.0, 1.0, 0.0, 0.0];

/// `a` first, then `b` (the PDF convention: `cm` puts the new matrix in front of the current one).
fn then(a: &Matrix, b: &Matrix) -> Matrix {
    [
        a[0] * b[0] + a[1] * b[2],
        a[0] * b[1] + a[1] * b[3],
        a[2] * b[0] + a[3] * b[2],
        a[2] * b[1] + a[3] * b[3],
        a[4] * b[0] + a[5] * b[2] + b[4],
        a[4] * b[1] + a[5] * b[3] + b[5],
    ]
}

fn apply(m: &Matrix, x: f64, y: f64) -> (f64, f64) {
    (m[0] * x + m[2] * y + m[4], m[1] * x + m[3] * y + m[5])
}

fn num(o: &Object) -> Option<f64> {
    o.as_float().ok().map(f64::from).or_else(|| o.as_i64().ok().map(|n| n as f64))
}

fn nums(ops: &[Object]) -> Option<Vec<f64>> {
    ops.iter().map(num).collect()
}

#[derive(Debug)]
enum Item {
    Rect(f64, f64, f64, f64),
    /// Straight segments of one subpath (a curve breaks it: it is not a ruling line).
    Poly(Vec<(f64, f64)>),
}

/// Where a painted path leaves ink: the thin rectangles and segments that can be the lines of a table, and
/// the boxes that can be a cell. A box that fills the whole page is a background and is left out.
fn paint(items: &[Item], ctm: &Matrix, stroke: bool, page: (f64, f64), out: &mut Vec<Rule>) {
    let rotated = ctm[1].abs() > 1e-6 || ctm[2].abs() > 1e-6;
    if rotated {
        return;
    }
    let boxed = |x0: f64, y0: f64, x1: f64, y1: f64, out: &mut Vec<Rule>| {
        let (x0, x1, y0, y1) = (x0.min(x1), x0.max(x1), y0.min(y1), y0.max(y1));
        let (w, h) = (x1 - x0, y1 - y0);
        if w <= THIN && h <= THIN {
            return;
        }
        if h <= THIN {
            out.push(Rule { horizontal: true, at: (y0 + y1) / 2.0, from: x0, to: x1 });
        } else if w <= THIN {
            out.push(Rule { horizontal: false, at: (x0 + x1) / 2.0, from: y0, to: y1 });
        } else if !(w >= page.0 * 0.9 && h >= page.1 * 0.9) {
            out.push(Rule { horizontal: true, at: y0, from: x0, to: x1 });
            out.push(Rule { horizontal: true, at: y1, from: x0, to: x1 });
            out.push(Rule { horizontal: false, at: x0, from: y0, to: y1 });
            out.push(Rule { horizontal: false, at: x1, from: y0, to: y1 });
        }
    };
    for item in items {
        match item {
            Item::Rect(x, y, w, h) => {
                let (a, b) = (apply(ctm, *x, *y), apply(ctm, x + w, y + h));
                boxed(a.0, a.1, b.0, b.1, out);
            }
            Item::Poly(points) => {
                let p: Vec<(f64, f64)> = points.iter().map(|(x, y)| apply(ctm, *x, *y)).collect();
                // four corners (closed or not) that make an axis-aligned rectangle: a box drawn with lines
                let closed = p.len() == 5 && (p[0].0 - p[4].0).abs() < 0.01 && (p[0].1 - p[4].1).abs() < 0.01;
                if p.len() == 4 || closed {
                    let xs: Vec<f64> = p.iter().take(4).map(|q| q.0).collect();
                    let ys: Vec<f64> = p.iter().take(4).map(|q| q.1).collect();
                    let spans = |v: &[f64]| v.iter().cloned().fold(f64::MAX, f64::min)..=v.iter().cloned().fold(f64::MIN, f64::max);
                    let aligned = p.iter().take(4).all(|q| xs.iter().any(|x| (x - q.0).abs() < 0.01) && ys.iter().any(|y| (y - q.1).abs() < 0.01));
                    if aligned {
                        let (xr, yr) = (spans(&xs), spans(&ys));
                        boxed(*xr.start(), *yr.start(), *xr.end(), *yr.end(), out);
                        continue;
                    }
                }
                if stroke {
                    for pair in p.windows(2) {
                        let ((x0, y0), (x1, y1)) = (pair[0], pair[1]);
                        if (y1 - y0).abs() <= 0.5 && (x1 - x0).abs() > 0.5 {
                            out.push(Rule { horizontal: true, at: (y0 + y1) / 2.0, from: x0.min(x1), to: x0.max(x1) });
                        } else if (x1 - x0).abs() <= 0.5 && (y1 - y0).abs() > 0.5 {
                            out.push(Rule { horizontal: false, at: (x0 + x1) / 2.0, from: y0.min(y1), to: y0.max(y1) });
                        }
                    }
                }
            }
        }
    }
}

/// Runs the drawing operators of a content stream and collects the lines it leaves. `form` gives the
/// operators and matrix of a nested form by name (the caller owns the document).
fn interpret(ops: &[Operation], start: Matrix, page: (f64, f64), form: &dyn Fn(&str) -> Option<(Vec<Operation>, Matrix)>, depth: usize, out: &mut Vec<Rule>) {
    let mut ctm = start;
    let mut stack: Vec<Matrix> = Vec::new();
    let mut path: Vec<Item> = Vec::new();
    let mut subpath: Vec<(f64, f64)> = Vec::new();
    let mut curved = false;
    let flush = |path: &mut Vec<Item>, subpath: &mut Vec<(f64, f64)>, curved: &mut bool| {
        if subpath.len() >= 2 && !*curved {
            path.push(Item::Poly(std::mem::take(subpath)));
        }
        subpath.clear();
        *curved = false;
    };
    for op in ops {
        if out.len() > MAX_RULES {
            return;
        }
        let o = op.operands.as_slice();
        match op.operator.as_str() {
            "q" => stack.push(ctm),
            "Q" => {
                if let Some(m) = stack.pop() {
                    ctm = m;
                }
            }
            "cm" => {
                if let Some([a, b, c, d, e, f]) = nums(o).as_deref() {
                    ctm = then(&[*a, *b, *c, *d, *e, *f], &ctm);
                }
            }
            "m" => {
                flush(&mut path, &mut subpath, &mut curved);
                if let Some([x, y]) = nums(o).as_deref() {
                    subpath.push((*x, *y));
                }
            }
            "l" => {
                if let Some([x, y]) = nums(o).as_deref() {
                    subpath.push((*x, *y));
                }
            }
            "c" | "v" | "y" => curved = true,
            "h" => {
                if let Some(first) = subpath.first().copied() {
                    subpath.push(first);
                }
            }
            "re" => {
                if let Some([x, y, w, h]) = nums(o).as_deref() {
                    path.push(Item::Rect(*x, *y, *w, *h));
                }
            }
            "S" | "s" | "f" | "F" | "f*" | "B" | "B*" | "b" | "b*" => {
                if matches!(op.operator.as_str(), "s" | "b" | "b*") {
                    if let Some(first) = subpath.first().copied() {
                        subpath.push(first);
                    }
                }
                flush(&mut path, &mut subpath, &mut curved);
                let stroke = matches!(op.operator.as_str(), "S" | "s" | "B" | "B*" | "b" | "b*");
                paint(&path, &ctm, stroke, page, out);
                path.clear();
            }
            "n" => {
                path.clear();
                subpath.clear();
                curved = false;
            }
            "Do" if depth < MAX_FORM_DEPTH => {
                if let Some(name) = o.first().and_then(|n| n.as_name().ok()).and_then(|n| std::str::from_utf8(n).ok()) {
                    if let Some((inner, matrix)) = form(name) {
                        interpret(&inner, then(&matrix, &ctm), page, form, depth + 1, out);
                    }
                }
            }
            _ => {}
        }
    }
}

fn matrix_of(d: &Dictionary) -> Matrix {
    d.get(b"Matrix").ok().and_then(|m| m.as_array().ok()).and_then(|a| nums(a)).and_then(|v| <[f64; 6]>::try_from(v).ok()).unwrap_or(IDENTITY)
}

/// The lines of a page in the PDF's own coordinates (y grows upward).
pub fn page_rules(doc: &Document, page_id: ObjectId, page: (f64, f64)) -> Vec<Rule> {
    let Ok(bytes) = doc.get_page_content(page_id) else { return Vec::new() };
    let Ok(content) = lopdf::content::Content::decode(&bytes) else { return Vec::new() };
    let resources = doc.get_page_resources(page_id).ok().and_then(|(r, _)| r);
    let xobject = |name: &str| -> Option<(Vec<Operation>, Matrix)> {
        let xo = doc.dereference(resources?.get(b"XObject").ok()?).ok()?.1.as_dict().ok()?;
        let stream = doc.dereference(xo.get(name.as_bytes()).ok()?).ok()?.1.as_stream().ok()?;
        if stream.dict.get(b"Subtype").ok().and_then(|s| s.as_name().ok()) != Some(&b"Form"[..]) {
            return None;
        }
        let ops = lopdf::content::Content::decode(&stream.decompressed_content().ok()?).ok()?.operations;
        Some((ops, matrix_of(&stream.dict)))
    };
    let mut out = Vec::new();
    interpret(&content.operations, IDENTITY, page, &xobject, 0, &mut out);
    if out.len() > MAX_RULES {
        return Vec::new();
    }
    out
}

// ------------------------------------------------------------------ from lines to cells

/// Puts the lines in page coordinates (y grows downward), snaps lines that are the same line and joins the
/// pieces of one line.
fn tidy(rules: &[Rule], height: f64) -> Vec<Rule> {
    let all: Vec<Rule> = rules
        .iter()
        .map(|r| if r.horizontal { Rule { horizontal: true, at: height - r.at, from: r.from, to: r.to } } else { Rule { horizontal: false, at: r.at, from: height - r.to, to: height - r.from } })
        .filter(|r| r.at.is_finite() && r.from.is_finite() && r.to.is_finite())
        .collect();
    let mut out: Vec<Rule> = Vec::new();
    for horizontal in [true, false] {
        let mut same: Vec<Rule> = all.iter().copied().filter(|r| r.horizontal == horizontal).collect();
        same.sort_by(|a, b| a.at.total_cmp(&b.at).then(a.from.total_cmp(&b.from)));
        // lines at (nearly) the same place share one position
        let mut anchor = f64::MIN;
        for r in &mut same {
            if (r.at - anchor).abs() > TOUCH {
                anchor = r.at;
            }
            r.at = anchor;
        }
        same.sort_by(|a, b| a.at.total_cmp(&b.at).then(a.from.total_cmp(&b.from)));
        for r in same {
            match out.last_mut() {
                Some(l) if l.horizontal == horizontal && l.at == r.at && r.from <= l.to + JOIN_GAP => l.to = l.to.max(r.to),
                _ => out.push(r),
            }
        }
    }
    out
}

struct Dsu(Vec<usize>);

impl Dsu {
    fn new(n: usize) -> Dsu {
        Dsu((0..n).collect())
    }
    fn find(&mut self, mut x: usize) -> usize {
        while self.0[x] != x {
            self.0[x] = self.0[self.0[x]];
            x = self.0[x];
        }
        x
    }
    fn union(&mut self, a: usize, b: usize) {
        let (a, b) = (self.find(a), self.find(b));
        if a != b {
            self.0[b] = a;
        }
    }
}

fn touches(h: &Rule, v: &Rule) -> bool {
    v.at >= h.from - TOUCH && v.at <= h.to + TOUCH && h.at >= v.from - TOUCH && h.at <= v.to + TOUCH
}

fn distinct(mut v: Vec<f64>) -> Vec<f64> {
    v.sort_by(f64::total_cmp);
    v.dedup_by(|a, b| (*a - *b).abs() <= TOUCH);
    v
}

/// The table made by one set of lines that touch each other, or `None` if it is not one (fewer than two
/// rows or two columns, or nothing written in it).
fn table_of(lines: &[Rule], glyphs: &[Glyph]) -> Option<(GridTable, Fill)> {
    let xs = distinct(lines.iter().filter(|r| !r.horizontal).map(|r| r.at).collect());
    let ys = distinct(lines.iter().filter(|r| r.horizontal).map(|r| r.at).collect());
    if xs.len() < 3 || ys.len() < 3 {
        return None;
    }
    let (cols, rows) = (xs.len() - 1, ys.len() - 1);
    let covers_v = |x: f64, y: f64| lines.iter().any(|r| !r.horizontal && (r.at - x).abs() <= TOUCH && y >= r.from - TOUCH && y <= r.to + TOUCH);
    let covers_h = |y: f64, x: f64| lines.iter().any(|r| r.horizontal && (r.at - y).abs() <= TOUCH && x >= r.from - TOUCH && x <= r.to + TOUCH);
    // two neighbouring grid cells with no line between them are one cell (a merged cell)
    let mut dsu = Dsu::new(rows * cols);
    for i in 0..rows {
        for j in 0..cols {
            if j + 1 < cols && !covers_v(xs[j + 1], (ys[i] + ys[i + 1]) / 2.0) {
                dsu.union(i * cols + j, i * cols + j + 1);
            }
            if i + 1 < rows && !covers_h(ys[i + 1], (xs[j] + xs[j + 1]) / 2.0) {
                dsu.union(i * cols + j, (i + 1) * cols + j);
            }
        }
    }
    // each letter goes to the grid cell that contains its centre; letters outside the lines are not the table's
    let mut texts: Vec<Vec<Glyph>> = vec![Vec::new(); rows * cols];
    let (left, top, right, bottom) = (xs[0], ys[0], xs[cols], ys[rows]);
    for g in glyphs {
        let (cx, mut cy) = (g.cx, g.cy);
        let turned = matches!(g.dir, Dir::Up | Dir::Down);
        if turned && cx >= left - 1.0 && cx <= right + 1.0 && cy >= top - TURNED_OVERFLOW && cy <= bottom + TURNED_OVERFLOW {
            cy = cy.clamp(top, bottom);
        }
        if cx < left - 1.0 || cx > right + 1.0 || cy < top - 1.0 || cy > bottom + 1.0 {
            continue;
        }
        let j = xs.partition_point(|x| *x <= cx).saturating_sub(1).min(cols - 1);
        let i = ys.partition_point(|y| *y <= cy).saturating_sub(1).min(rows - 1);
        texts[i * cols + j].push(g.clone());
    }
    let mut cell_glyphs: std::collections::HashMap<usize, Vec<Glyph>> = std::collections::HashMap::new();
    for (k, g) in texts.into_iter().enumerate() {
        cell_glyphs.entry(dsu.find(k)).or_default().extend(g);
    }
    let cell: std::collections::HashMap<usize, String> = cell_glyphs.iter().map(|(k, g)| (*k, cell_text(g))).collect();
    let mut out_rows: Vec<String> = Vec::new();
    let mut widest = 0;
    for i in 0..rows {
        let mut seen: Vec<usize> = Vec::new();
        let mut cells: Vec<String> = Vec::new();
        for j in 0..cols {
            let root = dsu.find(i * cols + j);
            if seen.contains(&root) {
                continue;
            }
            seen.push(root);
            // a cell that spans rows is written in its first row, and again in the next ones if it is short
            let first_row = (0..rows).find(|r| (0..cols).any(|c| dsu.find(r * cols + c) == root)).unwrap_or(i);
            let text = cell.get(&root).cloned().unwrap_or_default();
            cells.push(if first_row == i || text.chars().count() <= MAX_REPEATED_CHARS { text } else { String::new() });
        }
        while cells.last().is_some_and(String::is_empty) {
            cells.pop();
        }
        if cells.iter().any(|c| !c.is_empty()) {
            widest = widest.max(cells.len());
            out_rows.push(cells.join(" | "));
        }
    }
    // is it a table of data, or the ruled boxes of a page's design?
    let roots: std::collections::BTreeSet<usize> = (0..rows * cols).map(|k| dsu.find(k)).collect();
    let filled: Vec<&String> = roots.iter().filter_map(|r| cell.get(r)).filter(|t| !t.is_empty()).collect();
    let letters: usize = filled.iter().map(|t| t.chars().filter(|c| c.is_alphabetic()).count()).sum();
    let fill = Fill { cells: roots.len(), filled: filled.len(), letters };
    (out_rows.len() >= 2 && widest >= 2).then_some((GridTable { bounds: (left, top, right, bottom), rows: out_rows }, fill))
}

/// The tables of a page that have their borders drawn. `rules` are in the PDF's coordinates, `glyphs` in
/// page coordinates (y grows downward) and `height` is what turns one into the other.
pub fn grid_tables(rules: &[Rule], glyphs: &[Glyph], height: f64) -> Vec<GridTable> {
    grid_candidates(rules, glyphs, height).into_iter().filter(|(_, f)| f.is_data()).map(|(t, _)| t).collect()
}

/// Every grid the lines make, with how full it is: the data tables and what is only the page's design.
pub fn grid_candidates(rules: &[Rule], glyphs: &[Glyph], height: f64) -> Vec<(GridTable, Fill)> {
    let lines = tidy(rules, height);
    if lines.len() < 6 {
        return Vec::new();
    }
    let mut dsu = Dsu::new(lines.len());
    for (a, h) in lines.iter().enumerate().filter(|(_, r)| r.horizontal) {
        for (b, v) in lines.iter().enumerate().filter(|(_, r)| !r.horizontal) {
            if touches(h, v) {
                dsu.union(a, b);
            }
        }
    }
    let mut groups: std::collections::BTreeMap<usize, Vec<Rule>> = std::collections::BTreeMap::new();
    for (k, r) in lines.iter().enumerate() {
        let root = dsu.find(k);
        groups.entry(root).or_default().push(*r);
    }
    let mut tables: Vec<(GridTable, Fill)> = groups.values().filter_map(|g| table_of(g, glyphs)).collect();
    tables.sort_by(|a, b| a.0.bounds.1.total_cmp(&b.0.bounds.1).then(a.0.bounds.0.total_cmp(&b.0.bounds.0)));
    tables
}

#[cfg(test)]
mod tests {
    use super::*;

    fn letters(x: f64, y: f64, text: &str) -> Vec<Glyph> {
        text.chars().enumerate().map(|(i, c)| Glyph::across(x + i as f64 * 5.0, x + (i as f64 + 1.0) * 5.0, y, 10.0, &c.to_string())).collect()
    }

    /// A word written turned a quarter, its baseline at `x`, starting at `y` and running up (`Up`) or down (`Down`).
    fn turned(x: f64, y: f64, text: &str, dir: Dir) -> Vec<Glyph> {
        let sign = if dir == Dir::Up { -1.0 } else { 1.0 };
        text.chars()
            .enumerate()
            .map(|(i, c)| {
                let along = y + sign * i as f64 * 5.0;
                Glyph { x, end: x + 5.0, y: along, size: 10.0, text: c.to_string(), dir, cx: x + if dir == Dir::Up { -3.0 } else { 3.0 }, cy: along + sign * 2.5 }
            })
            .collect()
    }

    /// Page height 800: a rule at `y` from the top is drawn at `800 - y` in the PDF's own coordinates.
    const H: f64 = 800.0;

    fn hline(y_top: f64, x0: f64, x1: f64) -> Rule {
        Rule { horizontal: true, at: H - y_top, from: x0, to: x1 }
    }

    fn vline(x: f64, y_top0: f64, y_top1: f64) -> Rule {
        Rule { horizontal: false, at: x, from: H - y_top1, to: H - y_top0 }
    }

    /// A full grid: columns at `xs`, rows at `ys` (from the top).
    fn grid(xs: &[f64], ys: &[f64]) -> Vec<Rule> {
        let mut r = Vec::new();
        for y in ys {
            r.push(hline(*y, xs[0], xs[xs.len() - 1]));
        }
        for x in xs {
            r.push(vline(*x, ys[0], ys[ys.len() - 1]));
        }
        r
    }

    #[test]
    fn cells_with_whole_paragraphs_in_several_lines_are_read_cell_by_cell() {
        // what the text layout cannot do: two columns of long sentences, each cell on three lines
        let rules = grid(&[50.0, 250.0, 550.0], &[100.0, 160.0, 240.0]);
        let mut g = Vec::new();
        g.extend(letters(55.0, 120.0, "Requisito"));
        g.extend(letters(255.0, 120.0, "Descripcion"));
        for (k, line) in ["La organizacion debe", "tener dos anos de", "operacion comprobada"].iter().enumerate() {
            g.extend(letters(55.0, 180.0 + k as f64 * 14.0, line));
        }
        for (k, line) in ["Se acredita con el acta", "constitutiva y con los estados", "financieros del ultimo ano"].iter().enumerate() {
            g.extend(letters(255.0, 180.0 + k as f64 * 14.0, line));
        }
        let tables = grid_tables(&rules, &g, H);
        assert_eq!(tables.len(), 1);
        assert_eq!(
            tables[0].rows,
            vec![
                "Requisito | Descripcion",
                "La organizacion debe tener dos anos de operacion comprobada | Se acredita con el acta constitutiva y con los estados financieros del ultimo ano",
            ]
        );
    }

    #[test]
    fn a_merged_cell_spanning_columns_and_one_spanning_rows_keep_their_meaning() {
        // 3 columns, 4 rows. Row 1 is one title across all columns. In column 0, rows 2 and 3 are one cell.
        // the line at 190 does not cross column 0: its rows 2 and 3 are one cell
        let mut rules = vec![hline(100.0, 50.0, 450.0), hline(130.0, 50.0, 450.0), hline(160.0, 50.0, 450.0), hline(190.0, 150.0, 450.0), hline(220.0, 50.0, 450.0)];
        rules.extend([vline(50.0, 100.0, 220.0), vline(450.0, 100.0, 220.0), vline(150.0, 130.0, 220.0), vline(300.0, 130.0, 220.0)]);
        let mut g = Vec::new();
        g.extend(letters(60.0, 120.0, "Calendario"));
        g.extend(letters(55.0, 150.0, "Etapa"));
        g.extend(letters(155.0, 150.0, "Inicio"));
        g.extend(letters(305.0, 150.0, "Fin"));
        g.extend(letters(55.0, 185.0, "Registro"));
        g.extend(letters(155.0, 175.0, "agosto"));
        g.extend(letters(305.0, 175.0, "septiembre"));
        g.extend(letters(155.0, 205.0, "octubre"));
        g.extend(letters(305.0, 205.0, "noviembre"));
        let t = &grid_tables(&rules, &g, H)[0];
        assert_eq!(t.rows, vec!["Calendario", "Etapa | Inicio | Fin", "Registro | agosto | septiembre", "Registro | octubre | noviembre"]);
    }

    #[test]
    fn a_box_with_text_is_not_a_table_and_neither_is_a_single_column() {
        // one box
        let rules = grid(&[50.0, 400.0], &[100.0, 200.0]);
        assert!(grid_tables(&rules, &letters(60.0, 150.0, "Un aviso dentro de un cuadro"), H).is_empty());
        // one column of boxed lines
        let rules = grid(&[50.0, 400.0], &[100.0, 140.0, 180.0, 220.0]);
        let mut g = letters(60.0, 125.0, "Primer renglon");
        g.extend(letters(60.0, 165.0, "Segundo renglon"));
        assert!(grid_tables(&rules, &g, H).is_empty());
    }

    #[test]
    fn the_ruled_boxes_of_a_pages_design_are_not_tables_of_data() {
        // a footer band: three rows, almost all empty, with the page number in one cell
        let rules = grid(&[0.0, 200.0, 400.0, 600.0], &[740.0, 770.0, 800.0]);
        assert!(grid_tables(&rules, &letters(580.0, 760.0, "26"), H).is_empty());
        // a frame around a whole page of running text: one cell with everything in it
        let rules = grid(&[0.0, 300.0, 600.0], &[0.0, 400.0, 800.0]);
        assert!(grid_tables(&rules, &letters(20.0, 50.0, "Un parrafo muy largo de la pagina entera"), H).is_empty());
        // a strip of numbers (the axis of a chart) has no words in it
        let rules = grid(&[60.0, 90.0, 120.0], &[100.0, 130.0, 160.0, 190.0]);
        let mut g = Vec::new();
        for (k, y) in [115.0, 145.0, 175.0].iter().enumerate() {
            g.extend(letters(65.0, *y, &format!("{}", k + 1)));
            g.extend(letters(95.0, *y, &format!("{}", k + 10)));
        }
        assert!(grid_tables(&rules, &g, H).is_empty());
    }

    #[test]
    fn a_column_label_turned_up_the_page_is_read_as_a_word_and_stays_in_its_cell() {
        // «Estado» written bottom to top in the first column, and the rows beside it
        let rules = grid(&[50.0, 90.0, 300.0, 500.0], &[100.0, 140.0, 180.0, 220.0]);
        let mut g = turned(70.0, 215.0, "Estado", Dir::Up);
        g.extend(letters(100.0, 125.0, "Atencion"));
        g.extend(letters(310.0, 125.0, "Si"));
        g.extend(letters(100.0, 165.0, "Formacion"));
        g.extend(letters(310.0, 165.0, "No"));
        g.extend(letters(100.0, 205.0, "Redes"));
        g.extend(letters(310.0, 205.0, "Si"));
        let t = &grid_tables(&rules, &g, H)[0];
        assert!(t.rows.iter().any(|r| r.starts_with("Estado")), "{:?}", t.rows);
        // read on its own: bottom to top for a label turned left, top down for one turned right
        // a label longer than its cell, running up past the top line of the table, is not cut at the line
        let rules = grid(&[50.0, 90.0, 300.0, 500.0], &[100.0, 140.0, 180.0]);
        let mut g = turned(70.0, 135.0, "Alimentacion", Dir::Up);
        g.extend(letters(100.0, 125.0, "Atencion"));
        g.extend(letters(310.0, 125.0, "Si"));
        g.extend(letters(100.0, 165.0, "Formacion"));
        g.extend(letters(310.0, 165.0, "No"));
        let t = &grid_tables(&rules, &g, H)[0];
        assert!(t.rows.iter().any(|r| r.contains("Alimentacion")), "{:?}", t.rows);
        assert_eq!(cell_text(&turned(70.0, 215.0, "Estado", Dir::Up)), "Estado");
        assert_eq!(cell_text(&turned(70.0, 100.0, "Monto", Dir::Down)), "Monto");
    }

    #[test]
    fn two_tables_on_one_page_are_two_tables_and_text_outside_the_lines_is_not_taken() {
        let mut rules = grid(&[50.0, 200.0, 350.0], &[100.0, 130.0, 160.0]);
        rules.extend(grid(&[50.0, 200.0, 350.0], &[400.0, 430.0, 460.0]));
        let mut g = Vec::new();
        for (k, (a, b)) in [("Rubro", "Monto"), ("Obra", "$300,000")].iter().enumerate() {
            g.extend(letters(55.0, 120.0 + k as f64 * 30.0, a));
            g.extend(letters(205.0, 120.0 + k as f64 * 30.0, b));
        }
        for (k, (a, b)) in [("Etapa", "Fecha"), ("Cierre", "23 de mayo")].iter().enumerate() {
            g.extend(letters(55.0, 420.0 + k as f64 * 30.0, a));
            g.extend(letters(205.0, 420.0 + k as f64 * 30.0, b));
        }
        g.extend(letters(55.0, 300.0, "Texto entre las dos tablas"));
        let t = grid_tables(&rules, &g, H);
        assert_eq!(t.len(), 2);
        assert_eq!((t[0].rows.clone(), t[1].rows.clone()), (vec!["Rubro | Monto".to_string(), "Obra | $300,000".to_string()], vec!["Etapa | Fecha".to_string(), "Cierre | 23 de mayo".to_string()]));
        assert!(t[0].contains(60.0, 125.0) && !t[0].contains(60.0, 300.0));
    }

    #[test]
    fn lines_drawn_in_pieces_and_slightly_off_are_still_the_same_lines() {
        let mut rules = vec![hline(100.0, 50.0, 200.0), hline(100.4, 201.0, 350.0), hline(130.0, 50.0, 350.0), hline(160.2, 50.0, 350.0)];
        rules.extend([vline(50.0, 100.0, 160.0), vline(200.5, 100.0, 160.0), vline(350.0, 100.0, 160.0)]);
        let mut g = letters(55.0, 120.0, "Etapa");
        g.extend(letters(205.0, 120.0, "Fecha"));
        g.extend(letters(55.0, 150.0, "Cierre"));
        g.extend(letters(205.0, 150.0, "mayo"));
        assert_eq!(grid_tables(&rules, &g, H)[0].rows, vec!["Etapa | Fecha", "Cierre | mayo"]);
    }

    // ---- the operators of a content stream

    fn op(name: &str, args: &[f64]) -> Operation {
        Operation::new(name, args.iter().map(|a| Object::Real(*a as f32)).collect())
    }

    fn no_forms(_: &str) -> Option<(Vec<Operation>, Matrix)> {
        None
    }

    fn rules_of(ops: &[Operation]) -> Vec<Rule> {
        let mut out = Vec::new();
        interpret(ops, IDENTITY, (600.0, 800.0), &no_forms, 0, &mut out);
        out
    }

    #[test]
    fn thin_filled_rectangles_stroked_segments_and_boxes_are_all_lines() {
        // a Word-style table: borders are thin rectangles filled with ink (`re f*`), under a `cm`
        let ops = vec![op("q", &[]), op("cm", &[1.0, 0.0, 0.0, 1.0, 10.0, 20.0]), op("re", &[40.0, 700.0, 300.0, 0.5]), op("f*", &[]), op("re", &[40.0, 600.0, 0.5, 100.0]), op("f", &[]), op("Q", &[])];
        assert_eq!(rules_of(&ops), vec![Rule { horizontal: true, at: 720.25, from: 50.0, to: 350.0 }, Rule { horizontal: false, at: 50.25, from: 620.0, to: 720.0 }]);
        // stroked lines
        let ops = vec![op("m", &[10.0, 100.0]), op("l", &[210.0, 100.0]), op("S", &[]), op("m", &[10.0, 100.0]), op("l", &[10.0, 200.0]), op("S", &[])];
        let r = rules_of(&ops);
        assert_eq!((r.len(), r[0].horizontal, r[1].horizontal), (2, true, false));
        // a stroked box gives its four sides; a box that covers the page is a background and gives none
        assert_eq!(rules_of(&[op("re", &[50.0, 50.0, 200.0, 100.0]), op("S", &[])]).len(), 4);
        assert!(rules_of(&[op("re", &[0.0, 0.0, 600.0, 800.0]), op("f", &[])]).is_empty());
        // a path that is not painted (a clip) leaves nothing, and a diagonal or a curve is no line
        assert!(rules_of(&[op("re", &[50.0, 50.0, 200.0, 0.5]), op("n", &[])]).is_empty());
        assert!(rules_of(&[op("m", &[0.0, 0.0]), op("l", &[100.0, 100.0]), op("S", &[])]).is_empty());
        assert!(rules_of(&[op("m", &[0.0, 0.0]), op("c", &[1.0, 2.0, 3.0, 4.0, 5.0, 6.0]), op("l", &[100.0, 0.0]), op("S", &[])]).is_empty());
    }

    #[test]
    fn a_nested_form_is_followed_with_its_own_matrix() {
        let form = |name: &str| (name == "Fm1").then(|| (vec![op("re", &[0.0, 0.0, 100.0, 0.5]), op("f", &[])], [1.0, 0.0, 0.0, 1.0, 30.0, 400.0]));
        let mut out = Vec::new();
        let ops = vec![Operation::new("Do", vec![Object::Name(b"Fm1".to_vec())])];
        interpret(&ops, IDENTITY, (600.0, 800.0), &form, 0, &mut out);
        assert_eq!(out, vec![Rule { horizontal: true, at: 400.25, from: 30.0, to: 130.0 }]);
    }
}
