use super::signature::{
    FileKind, JPEG_SIGNATURE, PDF_SIGNATURE, PNG_SIGNATURE, SUPPORTED_SIGNATURES,
};

// ── FileKind::name ────────────────────────────────────────────────────────────

#[test]
fn jpeg_kind_name_is_jpeg() {
    assert_eq!(FileKind::Jpeg.name(), "JPEG");
}

#[test]
fn png_kind_name_is_png() {
    assert_eq!(FileKind::Png.name(), "PNG");
}

#[test]
fn pdf_kind_name_is_pdf() {
    assert_eq!(FileKind::Pdf.name(), "PDF");
}

// ── FileKind::extension ───────────────────────────────────────────────────────

#[test]
fn jpeg_kind_extension_is_jpg() {
    assert_eq!(FileKind::Jpeg.extension(), "jpg");
}

#[test]
fn png_kind_extension_is_png() {
    assert_eq!(FileKind::Png.extension(), "png");
}

#[test]
fn pdf_kind_extension_is_pdf() {
    assert_eq!(FileKind::Pdf.extension(), "pdf");
}

// ── FileKind Display ──────────────────────────────────────────────────────────

#[test]
fn jpeg_kind_display_matches_name() {
    assert_eq!(FileKind::Jpeg.to_string(), FileKind::Jpeg.name());
}

#[test]
fn png_kind_display_matches_name() {
    assert_eq!(FileKind::Png.to_string(), FileKind::Png.name());
}

#[test]
fn pdf_kind_display_matches_name() {
    assert_eq!(FileKind::Pdf.to_string(), FileKind::Pdf.name());
}

// ── JPEG_SIGNATURE ────────────────────────────────────────────────────────────

#[test]
fn jpeg_signature_kind_is_jpeg() {
    assert_eq!(JPEG_SIGNATURE.kind, FileKind::Jpeg);
}

#[test]
fn jpeg_signature_header_is_soi_marker() {
    assert_eq!(JPEG_SIGNATURE.header_pattern, &[0xFF, 0xD8, 0xFF]);
}

#[test]
fn jpeg_signature_footer_is_eoi_marker() {
    assert_eq!(JPEG_SIGNATURE.footer_pattern, &[0xFF, 0xD9]);
}

// ── PNG_SIGNATURE ─────────────────────────────────────────────────────────────

#[test]
fn png_signature_kind_is_png() {
    assert_eq!(PNG_SIGNATURE.kind, FileKind::Png);
}

#[test]
fn png_signature_header_is_eight_byte_magic() {
    assert_eq!(
        PNG_SIGNATURE.header_pattern,
        &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]
    );
}

#[test]
fn png_signature_footer_is_iend_chunk() {
    assert_eq!(
        PNG_SIGNATURE.footer_pattern,
        &[0x49, 0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82]
    );
}

// ── PDF_SIGNATURE ─────────────────────────────────────────────────────────────

#[test]
fn pdf_signature_kind_is_pdf() {
    assert_eq!(PDF_SIGNATURE.kind, FileKind::Pdf);
}

#[test]
fn pdf_signature_header_spells_percent_pdf() {
    assert_eq!(PDF_SIGNATURE.header_pattern, b"%PDF");
}

#[test]
fn pdf_signature_footer_spells_double_percent_eof() {
    assert_eq!(PDF_SIGNATURE.footer_pattern, b"%%EOF");
}

// ── SUPPORTED_SIGNATURES ──────────────────────────────────────────────────────

#[test]
fn supported_signatures_includes_jpeg() {
    let kinds: Vec<FileKind> = SUPPORTED_SIGNATURES.iter().map(|sig| sig.kind).collect();
    assert!(kinds.contains(&FileKind::Jpeg));
}

#[test]
fn supported_signatures_includes_png() {
    let kinds: Vec<FileKind> = SUPPORTED_SIGNATURES.iter().map(|sig| sig.kind).collect();
    assert!(kinds.contains(&FileKind::Png));
}

#[test]
fn supported_signatures_includes_pdf() {
    let kinds: Vec<FileKind> = SUPPORTED_SIGNATURES.iter().map(|sig| sig.kind).collect();
    assert!(kinds.contains(&FileKind::Pdf));
}
