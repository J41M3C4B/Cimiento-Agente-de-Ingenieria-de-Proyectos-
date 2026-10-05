//! Excel: edit a copy of a workbook writing only answer cells, keeping format.
//!
//! Phase 0 findings (see ADR-004): `umya-spreadsheet` keeps merged cells, list
//! validation, formulas, images, styles and hidden sheets, but always writes
//! `<calcPr>` without `fullCalcOnLoad`, so we patch it afterwards.

use std::io::{Cursor, Read, Write};
use std::path::Path;

#[derive(Debug, thiserror::Error)]
#[allow(dead_code)]
pub enum XlsxError {
    #[error("spreadsheet error: {0}")]
    Sheet(String),
    #[error("zip error: {0}")]
    Zip(#[from] zip::result::ZipError),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

#[allow(dead_code)]
pub enum CellValue {
    Text(String),
    Number(f64),
}

/// Writes `answers` (sheet, cell, value) into a copy of `src` saved at `dst`.
#[allow(dead_code)]
pub fn fill_copy(
    src: &Path,
    dst: &Path,
    answers: &[(&str, &str, CellValue)],
) -> Result<(), XlsxError> {
    let mut book =
        umya_spreadsheet::reader::xlsx::read(src).map_err(|e| XlsxError::Sheet(e.to_string()))?;
    for (sheet, cell, value) in answers {
        let ws = book
            .get_sheet_by_name_mut(sheet)
            .map_err(|_| XlsxError::Sheet(format!("missing sheet {sheet}")))?;
        let c = ws.get_cell_mut(*cell);
        match value {
            CellValue::Text(t) => c.set_value(t.as_str()),
            CellValue::Number(n) => c.set_value_number(*n),
        };
    }
    let mut buf = Cursor::new(Vec::new());
    umya_spreadsheet::writer::xlsx::write_writer(&book, &mut buf)
        .map_err(|e| XlsxError::Sheet(e.to_string()))?;
    let patched = mark_full_calc_on_load(buf.into_inner())?;
    std::fs::write(dst, patched)?;
    Ok(())
}

/// Adds `fullCalcOnLoad="1"` to `<calcPr>` so Excel recalculates formulas on open.
fn mark_full_calc_on_load(bytes: Vec<u8>) -> Result<Vec<u8>, XlsxError> {
    let mut zin = zip::ZipArchive::new(Cursor::new(bytes))?;
    let mut zout = zip::ZipWriter::new(Cursor::new(Vec::new()));
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
        if name == "xl/workbook.xml" {
            let xml = String::from_utf8_lossy(&data).into_owned();
            let xml = if xml.contains("fullCalcOnLoad") {
                xml
            } else if xml.contains("<calcPr") {
                xml.replacen("<calcPr", "<calcPr fullCalcOnLoad=\"1\"", 1)
            } else {
                xml.replacen("</workbook>", "<calcPr fullCalcOnLoad=\"1\"/></workbook>", 1)
            };
            data = xml.into_bytes();
        }
        zout.start_file(name, opts)?;
        zout.write_all(&data)?;
    }
    Ok(zout.finish()?.into_inner())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    fn fixture() -> std::path::PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../fixtures/office/cuestionario-prueba.xlsx")
    }

    fn parts(path: &Path) -> BTreeMap<String, String> {
        let mut z = zip::ZipArchive::new(std::fs::File::open(path).unwrap()).unwrap();
        (0..z.len())
            .map(|i| {
                let mut f = z.by_index(i).unwrap();
                let mut b = Vec::new();
                f.read_to_end(&mut b).unwrap();
                (f.name().to_string(), String::from_utf8_lossy(&b).into_owned())
            })
            .collect()
    }

    fn n(hay: &str, needle: &str) -> usize {
        hay.matches(needle).count()
    }

    #[test]
    fn ten_cells_written_and_format_preserved() {
        use CellValue::*;
        let dir = tempfile::tempdir().unwrap();
        let out = dir.path().join("out.xlsx");
        let answers = [
            ("Datos generales", "B5", Text("Casa Hogar Ficticia".into())),
            ("Datos generales", "B6", Number(18.0)),
            ("Datos generales", "B7", Number(46023.0)),
            ("Datos generales", "B8", Text("Asilo".into())),
            ("Datos generales", "B9", Number(150000.0)),
            ("Datos generales", "B10", Number(50000.0)),
            ("Datos generales", "A15", Text("Mejorará la seguridad.".into())),
            ("Presupuesto", "A2", Text("Regadera con barra".into())),
            ("Presupuesto", "B2", Number(4.0)),
            ("Presupuesto", "C2", Number(2500.0)),
        ];
        fill_copy(&fixture(), &out, &answers).unwrap();
        let out_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("target/office-out");
        std::fs::create_dir_all(&out_dir).unwrap();
        std::fs::copy(&out, out_dir.join("cuestionario-prueba-llenado.xlsx")).unwrap();

        let (a, b) = (parts(&fixture()), parts(&out));
        for sheet in 1..=3 {
            let k = format!("xl/worksheets/sheet{sheet}.xml");
            let (sa, sb) = (&a[&k], &b[&k]);
            for tag in ["<mergeCell ", "<dataValidation ", "<f>", "<drawing"] {
                assert_eq!(n(sa, tag), n(sb, tag), "{k} {tag}");
            }
        }
        let media = |m: &BTreeMap<String, String>| m.keys().filter(|k| k.starts_with("xl/media/")).count();
        assert_eq!(media(&a), media(&b));
        assert!(b["xl/workbook.xml"].contains("state=\"hidden\""));
        assert!(b["xl/workbook.xml"].contains("fullCalcOnLoad=\"1\""));
        assert_eq!(n(&a["xl/styles.xml"], "<fill>"), n(&b["xl/styles.xml"], "<fill>"));
        assert_eq!(n(&a["xl/styles.xml"], "<numFmt "), n(&b["xl/styles.xml"], "<numFmt "));

        // Written cells keep the answer-cell style (yellow fill) and the original is untouched.
        let s1 = &b["xl/worksheets/sheet1.xml"];
        let cell = |xml: &str, r: &str| {
            let i = xml.find(&format!("r=\"{r}\"")).unwrap();
            xml[i..xml[i..].find("</c>").map(|e| i + e).unwrap_or(xml.len())].to_string()
        };
        let style = |c: &str| c.split("s=\"").nth(1).unwrap().split('"').next().unwrap().to_string();
        assert_eq!(style(&cell(&a["xl/worksheets/sheet1.xml"], "B6")), style(&cell(s1, "B6")));

        use calamine::{open_workbook, Reader, Xlsx};
        let mut wb: Xlsx<_> = open_workbook(&out).unwrap();
        let r = wb.worksheet_range("Datos generales").unwrap();
        assert_eq!(r.get_value((5, 1)).unwrap().to_string(), "18");
        assert_eq!(r.get_value((4, 1)).unwrap().to_string(), "Casa Hogar Ficticia");
        let mut orig: Xlsx<_> = open_workbook(fixture()).unwrap();
        assert!(orig.worksheet_range("Datos generales").unwrap().get_value((5, 1)).map_or(true, |v| v.to_string().is_empty()));
    }
}
