//! Word: fill a .docx template by editing its ZIP/XML, keeping styles, headers,
//! footers, images and tables untouched.
//!
//! Markers look like `{{name}}`. Word often splits a marker across several runs
//! (`<w:r>`), so replacement works on the concatenated text of each paragraph and
//! writes the result into the first run involved, keeping that run's formatting.

use regex::Regex;
use std::collections::HashMap;
use std::io::{Cursor, Read, Write};
use std::sync::OnceLock;

#[derive(Debug, thiserror::Error)]
#[allow(dead_code)]
pub enum DocxError {
    #[error("zip error: {0}")]
    Zip(#[from] zip::result::ZipError),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("the template has no document body")]
    NoBody,
}

/// Table to insert in place of a `{{table:<key>}}` paragraph, or as a block of a document built from scratch.
#[derive(Debug, Clone, PartialEq)]
pub struct TableData {
    pub header: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

#[allow(dead_code)]
#[derive(Default)]
pub struct Fill {
    /// `{{key}}` -> text. Blank lines (`\n\n`) split the paragraph into several, cloning its format.
    pub text: HashMap<String, String>,
    /// `{{table:key}}` -> table.
    pub tables: HashMap<String, TableData>,
    /// New value for the document author (clears template metadata).
    pub author: Option<String>,
}

fn re(cell: &'static OnceLock<Regex>, pat: &str) -> &'static Regex {
    cell.get_or_init(|| Regex::new(pat).unwrap())
}

fn escape(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

fn unescape(s: &str) -> String {
    s.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&amp;", "&")
}

/// Fills the template bytes and returns the new .docx bytes.
#[allow(dead_code)]
pub fn fill_template(template: &[u8], fill: &Fill) -> Result<Vec<u8>, DocxError> {
    let mut zin = zip::ZipArchive::new(Cursor::new(template))?;
    let mut zout = zip::ZipWriter::new(Cursor::new(Vec::new()));
    let mut saw_body = false;
    for i in 0..zin.len() {
        let mut f = zin.by_index(i)?;
        let name = f.name().to_string();
        let opts = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);
        if f.is_dir() {
            zout.add_directory(name, opts)?;
            continue;
        }
        let mut data = Vec::new();
        f.read_to_end(&mut data)?;
        let is_part = name == "word/document.xml"
            || ((name.starts_with("word/header") || name.starts_with("word/footer"))
                && name.ends_with(".xml"));
        if is_part {
            saw_body |= name == "word/document.xml";
            let xml = String::from_utf8_lossy(&data).into_owned();
            data = fill_part(&xml, fill).into_bytes();
        } else if name == "docProps/core.xml" {
            if let Some(author) = &fill.author {
                data = set_author(&String::from_utf8_lossy(&data), author).into_bytes();
            }
        }
        zout.start_file(name, opts)?;
        zout.write_all(&data)?;
    }
    if !saw_body {
        return Err(DocxError::NoBody);
    }
    Ok(zout.finish()?.into_inner())
}

// ------------------------------------------------------------------ a document built from scratch

/// A block of a document the code writes whole (the guide of a project, ADR-018). The AI never writes the file:
/// it only supplies text that the person confirmed, and this puts it in place.
#[derive(Debug, Clone, PartialEq)]
pub enum Block {
    Title(String),
    /// A heading of level 1 or 2.
    Heading(u8, String),
    /// Blank lines split it into several paragraphs.
    Paragraph(String),
    Bullets(Vec<String>),
    /// A list of things to check off («☐»).
    Checklist(Vec<String>),
    Table(TableData),
    /// Small grey text for remarks and sources.
    Note(String),
}

impl Block {
    /// Everything the block says, for the scanner and for the tests.
    pub fn text(&self) -> String {
        match self {
            Block::Title(t) | Block::Heading(_, t) | Block::Paragraph(t) | Block::Note(t) => t.clone(),
            Block::Bullets(items) | Block::Checklist(items) => items.join("\n"),
            Block::Table(t) => t.header.iter().chain(t.rows.iter().flatten()).cloned().collect::<Vec<_>>().join("\n"),
        }
    }
}

const NS: &str = r#"xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships""#;
const XML_HEAD: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>"#;

fn run(text: &str, props: &str) -> String {
    let mut out = String::new();
    for (i, line) in text.split('\n').enumerate() {
        if i > 0 {
            out.push_str(&format!("<w:r>{props}<w:br/></w:r>"));
        }
        out.push_str(&format!(r#"<w:r>{props}<w:t xml:space="preserve">{}</w:t></w:r>"#, escape(line)));
    }
    out
}

fn paragraph(style: Option<&str>, ppr_extra: &str, text: &str, rpr: &str) -> String {
    let ppr = match (style, ppr_extra.is_empty()) {
        (None, true) => String::new(),
        (s, _) => format!("<w:pPr>{}{ppr_extra}</w:pPr>", s.map(|s| format!(r#"<w:pStyle w:val="{s}"/>"#)).unwrap_or_default()),
    };
    let props = if rpr.is_empty() { String::new() } else { format!("<w:rPr>{rpr}</w:rPr>") };
    format!("<w:p>{ppr}{}</w:p>", run(text, &props))
}

fn paragraphs(style: Option<&str>, text: &str, rpr: &str) -> String {
    text.split("\n\n").map(str::trim).filter(|t| !t.is_empty()).map(|t| paragraph(style, "", t, rpr)).collect()
}

fn cell(text: &str, width: u32, header: bool) -> String {
    let shade = if header { r#"<w:shd w:val="clear" w:color="auto" w:fill="E7E6E6"/>"# } else { "" };
    let rpr = if header { "<w:b/>" } else { "" };
    let body: String = if text.trim().is_empty() { paragraph(None, "", "", rpr) } else { text.split("\n\n").map(|t| paragraph(None, "", t.trim(), rpr)).collect() };
    format!(r#"<w:tc><w:tcPr><w:tcW w:w="{width}" w:type="dxa"/>{shade}</w:tcPr>{body}</w:tc>"#)
}

fn table(t: &TableData) -> String {
    let cols = t.header.len().max(t.rows.iter().map(Vec::len).max().unwrap_or(0)).max(1) as u32;
    let width = 9360 / cols; // the width of the page between margins, in twentieths of a point
    let grid: String = (0..cols).map(|_| format!(r#"<w:gridCol w:w="{width}"/>"#)).collect();
    let mut rows = String::new();
    if !t.header.is_empty() {
        let cells: String = (0..cols as usize).map(|i| cell(t.header.get(i).map_or("", String::as_str), width, true)).collect();
        rows.push_str(&format!("<w:tr><w:trPr><w:tblHeader/></w:trPr>{cells}</w:tr>"));
    }
    for r in &t.rows {
        let cells: String = (0..cols as usize).map(|i| cell(r.get(i).map_or("", String::as_str), width, false)).collect();
        rows.push_str(&format!("<w:tr>{cells}</w:tr>"));
    }
    // Word needs a paragraph after a table, and it separates it from what follows
    format!(r#"<w:tbl><w:tblPr><w:tblStyle w:val="TableGrid"/><w:tblW w:w="5000" w:type="pct"/></w:tblPr><w:tblGrid>{grid}</w:tblGrid>{rows}</w:tbl>{}"#, paragraph(None, "", "", ""))
}

fn block_xml(b: &Block) -> String {
    match b {
        Block::Title(t) => paragraph(Some("Title"), "", t, ""),
        Block::Heading(level, t) => paragraph(Some(if *level <= 1 { "Heading1" } else { "Heading2" }), "", t, ""),
        Block::Paragraph(t) => paragraphs(None, t, ""),
        Block::Note(t) => paragraphs(None, t, r#"<w:i/><w:color w:val="595959"/><w:sz w:val="18"/>"#),
        Block::Bullets(items) => items.iter().map(|i| paragraph(Some("ListParagraph"), r#"<w:ind w:left="567" w:hanging="283"/>"#, &format!("•  {i}"), "")).collect(),
        Block::Checklist(items) => items.iter().map(|i| paragraph(Some("ListParagraph"), r#"<w:ind w:left="567" w:hanging="283"/>"#, &format!("☐  {i}"), "")).collect(),
        Block::Table(t) => table(t),
    }
}

const STYLES: &str = r#"<w:styles xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
<w:docDefaults><w:rPrDefault><w:rPr><w:rFonts w:ascii="Calibri" w:hAnsi="Calibri" w:eastAsia="Calibri" w:cs="Calibri"/><w:sz w:val="22"/><w:szCs w:val="22"/><w:lang w:val="es-MX"/></w:rPr></w:rPrDefault><w:pPrDefault><w:pPr><w:spacing w:after="120" w:line="276" w:lineRule="auto"/></w:pPr></w:pPrDefault></w:docDefaults>
<w:style w:type="paragraph" w:default="1" w:styleId="Normal"><w:name w:val="Normal"/><w:qFormat/></w:style>
<w:style w:type="paragraph" w:styleId="Title"><w:name w:val="Title"/><w:basedOn w:val="Normal"/><w:next w:val="Normal"/><w:qFormat/><w:pPr><w:spacing w:after="240"/></w:pPr><w:rPr><w:b/><w:color w:val="1F3864"/><w:sz w:val="44"/><w:szCs w:val="44"/></w:rPr></w:style>
<w:style w:type="paragraph" w:styleId="Heading1"><w:name w:val="heading 1"/><w:basedOn w:val="Normal"/><w:next w:val="Normal"/><w:qFormat/><w:pPr><w:keepNext/><w:spacing w:before="360" w:after="120"/><w:outlineLvl w:val="0"/></w:pPr><w:rPr><w:b/><w:color w:val="1F3864"/><w:sz w:val="32"/><w:szCs w:val="32"/></w:rPr></w:style>
<w:style w:type="paragraph" w:styleId="Heading2"><w:name w:val="heading 2"/><w:basedOn w:val="Normal"/><w:next w:val="Normal"/><w:qFormat/><w:pPr><w:keepNext/><w:spacing w:before="240" w:after="80"/><w:outlineLvl w:val="1"/></w:pPr><w:rPr><w:b/><w:color w:val="2F5496"/><w:sz w:val="26"/><w:szCs w:val="26"/></w:rPr></w:style>
<w:style w:type="paragraph" w:styleId="ListParagraph"><w:name w:val="List Paragraph"/><w:basedOn w:val="Normal"/><w:qFormat/><w:pPr><w:spacing w:after="60"/></w:pPr></w:style>
<w:style w:type="table" w:default="1" w:styleId="TableNormal"><w:name w:val="Normal Table"/><w:uiPriority w:val="99"/><w:semiHidden/><w:tblPr><w:tblInd w:w="0" w:type="dxa"/><w:tblCellMar><w:top w:w="0" w:type="dxa"/><w:left w:w="108" w:type="dxa"/><w:bottom w:w="0" w:type="dxa"/><w:right w:w="108" w:type="dxa"/></w:tblCellMar></w:tblPr></w:style>
<w:style w:type="table" w:styleId="TableGrid"><w:name w:val="Table Grid"/><w:basedOn w:val="TableNormal"/><w:uiPriority w:val="39"/><w:tblPr><w:tblBorders><w:top w:val="single" w:sz="4" w:space="0" w:color="808080"/><w:left w:val="single" w:sz="4" w:space="0" w:color="808080"/><w:bottom w:val="single" w:sz="4" w:space="0" w:color="808080"/><w:right w:val="single" w:sz="4" w:space="0" w:color="808080"/><w:insideH w:val="single" w:sz="4" w:space="0" w:color="808080"/><w:insideV w:val="single" w:sz="4" w:space="0" w:color="808080"/></w:tblBorders><w:tblCellMar><w:top w:w="40" w:type="dxa"/><w:left w:w="108" w:type="dxa"/><w:bottom w:w="40" w:type="dxa"/><w:right w:w="108" w:type="dxa"/></w:tblCellMar></w:tblPr></w:style>
</w:styles>"#;

const CONTENT_TYPES: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/><Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/><Override PartName="/word/styles.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml"/><Override PartName="/docProps/core.xml" ContentType="application/vnd.openxmlformats-package.core-properties+xml"/><Override PartName="/docProps/app.xml" ContentType="application/vnd.openxmlformats-officedocument.extended-properties+xml"/></Types>"#;

const ROOT_RELS: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/><Relationship Id="rId2" Type="http://schemas.openxmlformats.org/package/2006/relationships/metadata/core-properties" Target="docProps/core.xml"/><Relationship Id="rId3" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/extended-properties" Target="docProps/app.xml"/></Relationships>"#;

const DOCUMENT_RELS: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles" Target="styles.xml"/></Relationships>"#;

/// Builds a .docx from blocks: letter size, Calibri, headings, lists, tables with a header row. The only metadata
/// it carries is the title, the author given and the creation date: nothing of any template.
pub fn build(blocks: &[Block], title: &str, author: &str, created_iso: &str) -> Result<Vec<u8>, DocxError> {
    let body: String = blocks.iter().map(block_xml).collect();
    let document = format!(
        r#"{XML_HEAD}<w:document {NS}><w:body>{body}<w:sectPr><w:pgSz w:w="12240" w:h="15840"/><w:pgMar w:top="1440" w:right="1440" w:bottom="1440" w:left="1440" w:header="708" w:footer="708" w:gutter="0"/></w:sectPr></w:body></w:document>"#
    );
    let core = format!(
        r#"{XML_HEAD}<cp:coreProperties xmlns:cp="http://schemas.openxmlformats.org/package/2006/metadata/core-properties" xmlns:dc="http://purl.org/dc/elements/1.1/" xmlns:dcterms="http://purl.org/dc/terms/" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"><dc:title>{}</dc:title><dc:creator>{a}</dc:creator><cp:lastModifiedBy>{a}</cp:lastModifiedBy><dcterms:created xsi:type="dcterms:W3CDTF">{c}</dcterms:created><dcterms:modified xsi:type="dcterms:W3CDTF">{c}</dcterms:modified></cp:coreProperties>"#,
        escape(title),
        a = escape(author),
        c = escape(created_iso)
    );
    let app = format!(r#"{XML_HEAD}<Properties xmlns="http://schemas.openxmlformats.org/officeDocument/2006/extended-properties"><Application>Cimiento</Application></Properties>"#);
    let styles = format!("{XML_HEAD}{STYLES}");

    let mut zout = zip::ZipWriter::new(Cursor::new(Vec::new()));
    let opts = zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
    for (name, data) in [
        ("[Content_Types].xml", CONTENT_TYPES.to_string()),
        ("_rels/.rels", ROOT_RELS.to_string()),
        ("word/document.xml", document),
        ("word/styles.xml", styles),
        ("word/_rels/document.xml.rels", DOCUMENT_RELS.to_string()),
        ("docProps/core.xml", core),
        ("docProps/app.xml", app),
    ] {
        zout.start_file(name, opts)?;
        zout.write_all(data.as_bytes())?;
    }
    Ok(zout.finish()?.into_inner())
}

fn set_author(xml: &str, author: &str) -> String {
    static CREATOR: OnceLock<Regex> = OnceLock::new();
    static MODIFIER: OnceLock<Regex> = OnceLock::new();
    let a = escape(author);
    let xml = re(&CREATOR, r"<dc:creator>[^<]*</dc:creator>")
        .replace(xml, format!("<dc:creator>{a}</dc:creator>").as_str());
    re(&MODIFIER, r"<cp:lastModifiedBy>[^<]*</cp:lastModifiedBy>")
        .replace(&xml, format!("<cp:lastModifiedBy>{a}</cp:lastModifiedBy>").as_str())
        .into_owned()
}

fn fill_part(xml: &str, fill: &Fill) -> String {
    static PARA: OnceLock<Regex> = OnceLock::new();
    re(&PARA, r"(?s)<w:p[ >].*?</w:p>")
        .replace_all(xml, |c: &regex::Captures| fill_paragraph(&c[0], fill))
        .into_owned()
}

/// Text of a paragraph and the byte spans of each `<w:t>` element's content.
fn paragraph_text(p: &str) -> (String, Vec<(usize, usize)>) {
    static T: OnceLock<Regex> = OnceLock::new();
    let mut text = String::new();
    let mut spans = Vec::new();
    for m in re(&T, r"(?s)<w:t(?: [^>]*)?>(.*?)</w:t>").captures_iter(p) {
        let g = m.get(1).unwrap();
        spans.push((g.start(), g.end()));
        text.push_str(&unescape(g.as_str()));
    }
    (text, spans)
}

fn fill_paragraph(p: &str, fill: &Fill) -> String {
    static MARK: OnceLock<Regex> = OnceLock::new();
    let (text, spans) = paragraph_text(p);
    if !text.contains("{{") {
        return p.to_string();
    }
    let mark = re(&MARK, r"\{\{([^{}]+)\}\}");

    // A paragraph that is exactly one table marker becomes a table.
    if let Some(m) = mark.captures(&text) {
        if let Some(key) = m[1].strip_prefix("table:") {
            if text.trim() == &m[0] {
                if let Some(t) = fill.tables.get(key) {
                    return build_table(t);
                }
            }
        }
    }

    let matches: Vec<(usize, usize, String)> = mark
        .captures_iter(&text)
        .filter_map(|c| {
            let m = c.get(0).unwrap();
            fill.text
                .get(&c[1])
                .map(|v| (m.start(), m.end(), v.clone()))
        })
        .collect();
    if matches.is_empty() {
        return p.to_string();
    }

    // Several paragraphs: only for a paragraph with a single marker whose text has blank lines.
    let chunks: Vec<String> = if matches.len() == 1 && matches[0].2.contains("

") {
        matches[0].2.split("

").map(str::to_string).collect()
    } else {
        vec![]
    };
    let render = |repl_for: &dyn Fn(usize) -> String| -> String {
        // Original text of each run, in order.
        let runs: Vec<&str> = spans.iter().map(|(s, e)| &p[*s..*e]).collect();
        let mut out = String::with_capacity(p.len());
        let mut last = 0;
        let mut offset = 0;
        for (i, (s, e)) in spans.iter().enumerate() {
            let run_text = unescape(runs[i]);
            let (rs, re_) = (offset, offset + run_text.len());
            offset = re_;
            let mut new_text = String::new();
            let mut cursor = rs;
            for (mi, (ms, me, _)) in matches.iter().enumerate() {
                if *me <= cursor || *ms >= re_ {
                    continue;
                }
                let seg_start = (*ms).max(rs);
                new_text.push_str(&text[cursor..seg_start]);
                if *ms >= rs {
                    new_text.push_str(&repl_for(mi));
                }
                cursor = (*me).min(re_);
            }
            new_text.push_str(&text[cursor..re_]);
            out.push_str(&p[last..*s]);
            out.push_str(&escape(&new_text));
            last = *e;
        }
        out.push_str(&p[last..]);
        // keep leading/trailing spaces
        out.replace("<w:t>", "<w:t xml:space=\"preserve\">")
    };
    if chunks.is_empty() {
        render(&|mi| matches[mi].2.clone())
    } else {
        chunks
            .iter()
            .map(|c| render(&|_| c.clone()))
            .collect::<Vec<_>>()
            .concat()
    }
}

fn build_table(t: &TableData) -> String {
    let cell = |s: &str, bold: bool| {
        let rpr = if bold { "<w:rPr><w:b/></w:rPr>" } else { "" };
        format!(
            "<w:tc><w:p><w:r>{rpr}<w:t xml:space=\"preserve\">{}</w:t></w:r></w:p></w:tc>",
            escape(s)
        )
    };
    let row = |cells: &[String], bold: bool| {
        format!(
            "<w:tr>{}</w:tr>",
            cells.iter().map(|c| cell(c, bold)).collect::<String>()
        )
    };
    let mut x = String::from(
        "<w:tbl><w:tblPr><w:tblStyle w:val=\"TableGrid\"/><w:tblW w:w=\"0\" w:type=\"auto\"/></w:tblPr>",
    );
    x.push_str(&row(&t.header, true));
    for r in &t.rows {
        x.push_str(&row(r, false));
    }
    x.push_str("</w:tbl><w:p/>");
    x
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn template() -> Vec<u8> {
        std::fs::read(
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../fixtures/office/plantilla-prueba.docx"),
        )
        .unwrap()
    }

    fn part(bytes: &[u8], name: &str) -> String {
        let mut z = zip::ZipArchive::new(Cursor::new(bytes)).unwrap();
        let mut s = String::new();
        z.by_name(name).unwrap().read_to_string(&mut s).unwrap();
        s
    }

    fn names(bytes: &[u8]) -> Vec<String> {
        let mut z = zip::ZipArchive::new(Cursor::new(bytes)).unwrap();
        (0..z.len())
            .map(|i| z.by_index(i).unwrap().name().to_string())
            .collect()
    }

    fn sample_fill() -> Fill {
        let mut f = Fill::default();
        for (k, v) in [
            ("titulo_proyecto", "Espacio seguro & accesible"),
            ("institucion_nombre", "Casa Hogar Ficticia"),
            (
                "section:justificacion",
                "Primer párrafo.\n\nSegundo párrafo.",
            ),
            ("beneficiarios_total", "18"),
            ("section:objetivo", "Reducir el riesgo de caídas."),
        ] {
            f.text.insert(k.into(), v.into());
        }
        f.tables.insert(
            "presupuesto".into(),
            TableData {
                header: vec!["Concepto".into(), "Cantidad".into(), "Importe".into()],
                rows: vec![
                    vec!["Regadera".into(), "4".into(), "$10,000.00".into()],
                    vec!["Barra de apoyo".into(), "6".into(), "$9,000.00".into()],
                ],
            },
        );
        f.author = Some("Casa Hogar Ficticia".into());
        f
    }

    #[test]
    fn replaces_markers_everywhere_and_keeps_structure() {
        let before = template();
        let after = fill_template(&before, &sample_fill()).unwrap();
        let out_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("target/office-out");
        std::fs::create_dir_all(&out_dir).unwrap();
        std::fs::write(out_dir.join("plantilla-prueba-llenada.docx"), &after).unwrap();

        // every package part still exists (header, footer, images, styles...)
        assert_eq!(names(&before), names(&after));

        let doc = part(&after, "word/document.xml");
        assert!(!doc.contains("{{"), "unreplaced marker left in body");
        assert!(doc.contains("Espacio seguro &amp; accesible"));
        assert!(doc.contains("Segundo párrafo."));
        // marker split across runs: formatting of the first run is kept
        assert!(doc.contains(">18<"));
        assert!(doc.contains("C00000"));
        // new table added next to the existing one
        assert_eq!(part(&before, "word/document.xml").matches("<w:tbl>").count(), 1);
        assert_eq!(doc.matches("<w:tbl>").count(), 2);
        assert!(doc.contains("Barra de apoyo"));
        // list style of the objective paragraph preserved
        assert!(doc.contains("ListBullet"));

        let footer = names(&after)
            .into_iter()
            .find(|n| n.starts_with("word/footer"))
            .unwrap();
        assert!(part(&after, &footer).contains("Casa Hogar Ficticia"));
        let header = names(&after)
            .into_iter()
            .find(|n| n.starts_with("word/header"))
            .unwrap();
        assert_eq!(
            part(&before, &header).contains("<w:drawing>"),
            part(&after, &header).contains("<w:drawing>")
        );

        let core = part(&after, "docProps/core.xml");
        assert!(core.contains("<dc:creator>Casa Hogar Ficticia</dc:creator>"));
        assert!(!core.contains("Autor de prueba"));
    }

    #[test]
    fn unknown_markers_are_left_untouched() {
        let after = fill_template(&template(), &Fill::default()).unwrap();
        assert!(part(&after, "word/document.xml").contains("{{section:justificacion}}"));
    }

    #[test]
    fn a_document_built_from_scratch_is_well_formed_and_reads_back_with_everything_in_it() {
        let blocks = vec![
            Block::Title("Guía del proyecto «Cocina & baños»".into()),
            Block::Heading(1, "1. Datos de la convocatoria".into()),
            Block::Table(TableData { header: vec!["Dato".into(), "Lo que dice".into()], rows: vec![vec!["Monto máximo".into(), "$250,000.00".into()], vec!["Cierre".into(), "2027-05-23 <límite>".into()]] }),
            Block::Heading(2, "Qué hay que entregar".into()),
            Block::Checklist(vec!["Acta constitutiva".into(), "Comprobante de domicilio".into()]),
            Block::Bullets(vec!["Primero".into(), "Segundo".into()]),
            Block::Paragraph("Primer párrafo.\n\nSegundo párrafo\ncon salto de línea.".into()),
            Block::Note("Documento de apoyo.".into()),
        ];
        let bytes = build(&blocks, "Guía", "Cimiento", "2026-10-03T12:00:00Z").unwrap();
        // every part is well-formed XML
        let mut zin = zip::ZipArchive::new(Cursor::new(&bytes)).unwrap();
        let names: Vec<String> = (0..zin.len()).map(|i| zin.by_index(i).unwrap().name().to_string()).collect();
        assert!(names.contains(&"word/document.xml".to_string()) && names.contains(&"[Content_Types].xml".to_string()), "{names:?}");
        for n in &names {
            let mut xml = String::new();
            zin.by_name(n).unwrap().read_to_string(&mut xml).unwrap();
            roxmltree::Document::parse(&xml).unwrap_or_else(|e| panic!("{n}: {e}"));
        }
        // it reads back through the same reader the app uses for Word files
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("guia.docx");
        std::fs::write(&path, &bytes).unwrap();
        let text = crate::documents::canonical::package::read_pieces(&path).unwrap().join("\n");
        for needle in ["Guía del proyecto «Cocina & baños»", "1. Datos de la convocatoria", "Monto máximo", "$250,000.00", "2027-05-23 <límite>", "☐  Acta constitutiva", "•  Primero", "Primer párrafo.", "Segundo párrafo", "con salto de línea.", "Documento de apoyo."] {
            assert!(text.contains(needle), "missing {needle:?} in {text}");
        }
    }

    #[test]
    fn the_only_metadata_is_the_title_the_author_given_and_the_date() {
        let bytes = build(&[Block::Paragraph("x".into())], "Mi guía", "Asilo Ficticio", "2026-10-03T12:00:00Z").unwrap();
        let mut zin = zip::ZipArchive::new(Cursor::new(&bytes)).unwrap();
        let mut core = String::new();
        zin.by_name("docProps/core.xml").unwrap().read_to_string(&mut core).unwrap();
        assert!(core.contains("<dc:title>Mi guía</dc:title>") && core.contains("<dc:creator>Asilo Ficticio</dc:creator>") && core.contains("2026-10-03T12:00:00Z"));
        assert!(!core.contains("Microsoft") && !core.contains("python"));
    }

    #[test]
    fn a_table_pads_short_rows_and_the_text_of_a_block_is_everything_it_says() {
        let t = TableData { header: vec!["A".into(), "B".into(), "C".into()], rows: vec![vec!["1".into()], vec!["2".into(), "3".into(), "4".into()]] };
        let xml = block_xml(&Block::Table(t.clone()));
        assert_eq!(xml.matches("<w:tc>").count(), 9, "three columns in every row");
        assert!(xml.contains("<w:tblHeader/>"));
        roxmltree::Document::parse(&format!("<w:root {NS}>{xml}</w:root>")).unwrap();
        assert_eq!(Block::Table(t).text(), "A\nB\nC\n1\n2\n3\n4");
        assert_eq!(Block::Bullets(vec!["x".into(), "y".into()]).text(), "x\ny");
        assert_eq!(Block::Heading(2, "Título".into()).text(), "Título");
        // an empty document is still a valid file
        assert!(build(&[], "t", "a", "2026-01-01T00:00:00Z").is_ok());
    }
}
