use super::*;

fn fixture(name: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../fixtures/office").join(name)
}

fn needs_ocr() {
    assert!(ocr::available(), "what reads pictures is missing: run scripts/fetch-ocr.ps1");
}

/// A scanned PDF is drawn and read page by page; a worn page is kept and said to be read with doubts.
#[test]
fn a_scanned_pdf_is_read_as_pictures_and_a_worn_page_is_doubtful() {
    needs_ocr();
    let r = read_file(&fixture("acta-escaneada.pdf")).unwrap();
    assert_eq!((r.pages.len(), r.from_picture.clone()), (2, vec![1, 2]));
    assert!(r.pages[0].contains("ACTA CONSTITUTIVA") && r.pages[0].contains("Casa Hogar Ficticia del Valle"), "{}", r.pages[0]);
    assert!(r.pages[0].contains("contacto@casaficticia.example.org"), "the scanner will find it: {}", r.pages[0]);
    assert_eq!(r.doubtful, vec![2], "{:?}", r.pages);
    // read again (a file held for the scanner comes back): the same, without reading it again
    assert_eq!(read_file(&fixture("acta-escaneada.pdf")).unwrap(), r);
}

#[test]
fn a_photo_is_one_page_read_as_a_picture() {
    needs_ocr();
    let r = read_upload("constancia.JPG", &std::fs::read(fixture("constancia-foto.jpg")).unwrap()).unwrap();
    assert_eq!((r.from_picture.clone(), r.doubtful.clone()), (vec![1], vec![]));
    assert!(r.pages[0].contains("CONSTANCIA DE REGISTRO") && r.pages[0].contains("padrón"), "{}", r.pages[0]);
}

/// A PDF with text is not read as pictures; what is not a document is not read.
#[test]
fn a_pdf_with_text_keeps_its_text_and_other_files_are_not_read() {
    let r = read_file(&fixture("convocatoria-con-tablas.pdf")).unwrap();
    assert!(r.from_picture.is_empty() && r.pages[0].contains("Convocatoria ficticia 2026"));
    assert_eq!(read_upload("notas.txt", b"hola"), Err(ReadError::Unsupported));
    assert_eq!(read_upload("sin-extension", b"hola"), Err(ReadError::Unsupported));
    assert_eq!(read_upload("roto.pdf", b"esto no es un pdf"), Err(ReadError::Damaged));
}

/// ADR-038: a page scanned and pasted into a Word file is read after the text of the file, as a page read from a
/// picture; a small logo is not a page and is not read.
#[test]
fn a_page_pasted_into_a_word_file_is_read_and_a_logo_is_not() {
    needs_ocr();
    let r = read_file(&fixture("word-con-imagen.docx")).unwrap();
    assert!(r.pages[0].contains("Informe del patronato"), "{:?}", r.pages);
    let last = r.pages.last().unwrap().split_whitespace().collect::<Vec<_>>().join(" ");
    assert!(last.contains("Casa Hogar Ficticia del Valle") && last.contains("presupuesto"), "{last}");
    assert_eq!(r.from_picture, vec![r.pages.len()], "one page from the pasted scan, none from the logo");
    // a Word file with only a logo reads as before
    assert!(read_file(&fixture("plantilla-prueba.docx")).unwrap().from_picture.is_empty());
}

/// A sheet put in the scanner on its side: Tesseract's model of orientation says it is turned, and it is read
/// upright (ADR-035).
#[test]
fn a_sheet_scanned_on_its_side_is_read_upright() {
    needs_ocr();
    let r = read_file(&fixture("acta-girada.pdf")).unwrap();
    let text = r.pages[0].split_whitespace().collect::<Vec<_>>().join(" ");
    assert!(text.contains("ACTA DE ASAMBLEA ORDINARIA") && text.contains("Casa Hogar Ficticia del Valle"), "{text}");
    assert!(r.doubtful.is_empty(), "{:?}", r.doubtful);
}

/// ADR-037: a page with a printed line on top and its body scanned has text, but most of it is a picture: it is read
/// as a picture, and what OCR reads, the body too, replaces its text.
#[test]
fn a_page_with_a_printed_header_and_a_scanned_body_is_read_as_a_picture() {
    needs_ocr();
    let r = read_file(&fixture("pagina-mixta.pdf")).unwrap();
    assert_eq!(r.from_picture, vec![1]);
    let text = r.pages[0].split_whitespace().collect::<Vec<_>>().join(" ");
    assert!(text.contains("Casa Hogar Ficticia del Valle") && text.contains("informe anual"), "{text}");
    // a page with text all over it is not read as a picture, though it has a picture on it
    assert!(read_file(&fixture("convocatoria-con-tablas.pdf")).unwrap().from_picture.is_empty());
}

/// A PDF, Word or Excel file with a password is said to have one: never «damaged», never silent.
#[test]
fn a_file_with_a_password_says_so() {
    assert_eq!(read_file(&fixture("protegido.pdf")), Err(ReadError::Protected));
    // what Office writes for a file saved with a password: its old container and a part named «EncryptionInfo»
    let mut word = vec![0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1];
    word.extend(std::iter::repeat_n(0u8, 64));
    word.extend("EncryptionInfo".encode_utf16().flat_map(u16::to_le_bytes));
    assert_eq!(read_upload("estatutos.docx", &word), Err(ReadError::Protected));
    assert_eq!(read_upload("cuotas.xlsx", &word), Err(ReadError::Protected));
    // the same container without that part is an old file, not one with a password
    assert_eq!(read_upload("viejo.docx", &word[..72]), Err(ReadError::Damaged));
}

#[test]
fn the_folder_of_an_upload_is_gone_when_it_is_dropped() {
    let scratch = Scratch::new().unwrap();
    let path = scratch.0.clone();
    std::fs::write(path.join("upload.pdf"), b"x").unwrap();
    assert!(path.exists());
    drop(scratch);
    assert!(!path.exists());
}

/// A screenshot with letters of about 7 pixels is enlarged and read again: the title, the whole number and the «rn»
/// that came out as «m» («Intemado») are read.
#[test]
fn a_screenshot_with_small_letters_is_enlarged_and_read_whole() {
    needs_ocr();
    let r = read_upload("captura.png", &std::fs::read(fixture("captura-chica.png")).unwrap()).unwrap();
    let text = &r.pages[0];
    for words in ["Registro de Instituciones", "48213", "Casa Hogar Ficticia del Valle", "Internado", "personas adultas mayores", "atención", "alimentación"] {
        assert!(text.contains(words), "{words}: {text}");
    }
    assert!(text.contains("registro@casaficticia.example.org"), "the scanner will find it: {text}");
    assert!(r.doubtful.is_empty());
}

/// A little photo taken from afar, where Tesseract finds no word as it comes, is enlarged three times and read.
#[test]
fn a_little_photo_where_no_word_was_found_is_enlarged_and_read() {
    needs_ocr();
    let r = read_file(&fixture("foto-chica.jpg")).unwrap();
    let text = &r.pages[0];
    for words in ["AVISO DE FUNCIONAMIENTO", "Casa Hogar Ficticia del Valle", "martes a domingo", "Ernesto"] {
        assert!(text.contains(words), "{words}: {text}");
    }
}

/// Pages with letters big enough are read once: neither the scanned pages nor the photos of the fixtures are
/// enlarged.
#[test]
fn a_page_with_letters_big_enough_is_not_enlarged() {
    needs_ocr();
    let mut letters = vec![];
    ocr::draw_pages(&fixture("acta-escaneada.pdf"), &[1], |_, picture, dpi| letters.push(ocr::read_picture(&picture, Some(dpi)).unwrap().letters)).unwrap();
    for photo in ["constancia-foto.jpg", "factura-foto.jpg"] {
        letters.push(ocr::read_picture(&std::fs::read(fixture(photo)).unwrap(), None).unwrap().letters);
    }
    assert!(letters.iter().all(|l| l.is_some_and(|tall| tall >= 20.0)), "{letters:?}");
}

/// A table read from a picture keeps its rows and parts its cells with « | »; the email of who sells is read with its
/// «@», for the scanner to find it.
#[test]
fn an_invoice_keeps_the_cells_of_its_table_apart() {
    needs_ocr();
    let r = read_file(&fixture("factura-foto.jpg")).unwrap();
    let text = &r.pages[0];
    for row in ["Concepto | Cantidad | Precio | Importe", "Arroz | 10 | $28.00 | $280.00", "Aceite | 4 | $45.00 | $180.00", "Total | $635.00"] {
        assert!(text.contains(row), "{row}: {text}");
    }
    assert!(text.contains("ventas@abarrotesficticios.mx"), "{text}");
    assert!(r.doubtful.is_empty());
}
