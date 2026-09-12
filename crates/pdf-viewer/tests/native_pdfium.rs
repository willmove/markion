use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;
use markion_pdf_viewer::{
    DEV_RUNTIME_ENV, DocumentId, NativePdfBackend, PdfErrorKind, PdfRendererBackend,
    runtime_library_name,
};
use std::path::{Path, PathBuf};

fn fixture_bytes(name: &str) -> Vec<u8> {
    let encoded = match name {
        "one-page-embedded.pdf" => include_str!("fixtures/one-page-embedded.pdf.b64"),
        "mixed-pages.pdf" => include_str!("fixtures/mixed-pages.pdf.b64"),
        "encrypted.pdf" => include_str!("fixtures/encrypted.pdf.b64"),
        "corrupt.pdf" => include_str!("fixtures/corrupt.pdf.b64"),
        _ => panic!("unknown PDF fixture"),
    };
    let compact: String = encoded
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect();
    STANDARD.decode(compact).expect("valid base64 PDF fixture")
}

fn write_fixture(root: &Path, name: &str) -> PathBuf {
    let path = root.join(name);
    std::fs::write(&path, fixture_bytes(name)).expect("write PDF fixture");
    path
}

#[test]
#[ignore = "requires an explicitly staged PDFium 7881 runtime"]
fn native_pdfium_fixture_matrix() {
    let runtime = std::env::var_os(DEV_RUNTIME_ENV)
        .map(PathBuf::from)
        .expect("MARKION_PDFIUM_RUNTIME must name the staged runtime");
    let missing = runtime.with_file_name(format!("missing-{}", runtime_library_name()));
    assert!(matches!(
        NativePdfBackend::load(&missing),
        Err(PdfErrorKind::RuntimeMissing)
    ));

    let temporary = tempfile::tempdir().unwrap();
    let one_page = write_fixture(temporary.path(), "one-page-embedded.pdf");
    let mixed = write_fixture(temporary.path(), "mixed-pages.pdf");
    let encrypted = write_fixture(temporary.path(), "encrypted.pdf");
    let corrupt = write_fixture(temporary.path(), "corrupt.pdf");
    assert!(
        fixture_bytes("one-page-embedded.pdf")
            .windows(b"/FontFile2".len())
            .any(|window| window == b"/FontFile2"),
        "embedded-font fixture lost its FontFile2 stream"
    );

    let mut backend = NativePdfBackend::load(&runtime).expect("load staged PDFium");
    let one_id = DocumentId(1);
    let one_geometry = backend.open(one_id, &one_page).expect("open one-page PDF");
    assert_eq!(one_geometry.len(), 1);
    assert!((one_geometry[0].width_points - 320.0).abs() < 0.1);
    assert!((one_geometry[0].height_points - 240.0).abs() < 0.1);
    let raster = backend
        .render(one_id, 0, 640)
        .expect("render embedded-font PDF");
    assert_eq!(raster.width_px, 640);
    assert_eq!(
        raster.rgba.len(),
        raster.width_px as usize * raster.height_px as usize * 4
    );
    assert_eq!(
        backend.render(one_id, 1, 640),
        Err(PdfErrorKind::PageUnavailable)
    );
    backend.close(one_id);

    let mixed_id = DocumentId(2);
    let mixed_geometry = backend.open(mixed_id, &mixed).expect("open mixed-size PDF");
    assert_eq!(mixed_geometry.len(), 3);
    assert!(mixed_geometry[0].height_points > mixed_geometry[0].width_points);
    assert!(mixed_geometry[1].width_points > mixed_geometry[1].height_points);
    for page in 0..mixed_geometry.len() as u32 {
        let raster = backend
            .render(mixed_id, page, 512)
            .expect("render mixed-size page");
        assert!(raster.byte_len() <= markion_pdf_viewer::MAX_RASTER_BYTES);
    }
    backend.close(mixed_id);

    assert_eq!(
        backend.open(DocumentId(3), &encrypted),
        Err(PdfErrorKind::Encrypted)
    );
    assert_eq!(
        backend.open(DocumentId(4), &corrupt),
        Err(PdfErrorKind::CorruptOrUnsupported)
    );
}
