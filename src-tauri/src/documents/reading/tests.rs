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

#[test]
fn the_folder_of_an_upload_is_gone_when_it_is_dropped() {
    let scratch = Scratch::new().unwrap();
    let path = scratch.0.clone();
    std::fs::write(path.join("upload.pdf"), b"x").unwrap();
    assert!(path.exists());
    drop(scratch);
    assert!(!path.exists());
}
