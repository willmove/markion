//! Public entry point (task 2.9): wires fonts → layout → emission.

use cosmic_text::FontSystem;

use crate::ir::PdfDocument;
use crate::{PdfError, emit, fonts, layout, text};

/// Renders the document IR to PDF bytes.
pub fn render(document: &PdfDocument) -> Result<Vec<u8>, PdfError> {
    fonts::with_pdf_font_system(|fs| render_with_fallback(fs, document))
}

fn render_with_fallback(fs: &mut FontSystem, document: &PdfDocument) -> Result<Vec<u8>, PdfError> {
    let mut cache = text::FontCache::default();
    loop {
        let error = match layout::layout_document(fs, &mut cache, document) {
            Ok(laid_out) => return emit::emit_document(&laid_out, &document.metadata),
            Err(error) => error,
        };
        let Some(id) = cache.rejected_face.take() else {
            return Err(error);
        };
        let mut db = fs.db().clone();
        db.remove_face(id);
        if db.is_empty() {
            return Err(error);
        }
        *fs = FontSystem::new_with_locale_and_db(fs.locale().to_owned(), db);
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use cosmic_text::fontdb::{Database, Source};
    use krilla::{Data, text::Font};

    use super::*;
    use crate::{Block, Run};

    #[test]
    fn rejected_face_is_excluded_and_text_is_reshaped_for_pdf() {
        let mut incompatible = include_bytes!("../fonts/NotoSansSC-Bold.otf").to_vec();
        let table_count = usize::from(u16::from_be_bytes([incompatible[4], incompatible[5]]));
        let directory = &mut incompatible[12..12 + table_count * 16];
        let head = directory
            .chunks_exact_mut(16)
            .find(|entry| &entry[..4] == b"head")
            .unwrap();
        head[12..16].fill(0);
        assert!(Font::new(Data::from(incompatible.clone()), 0).is_none());

        let mut db = Database::new();
        let rejected_id = db.load_font_source(Source::Binary(Arc::new(incompatible)))[0];
        let family = db.face(rejected_id).unwrap().families[0].0.clone();
        db.set_sans_serif_family(family);
        db.load_font_data(include_bytes!("../fonts/NotoSansSC-Regular.otf").to_vec());

        let mut fs = FontSystem::new_with_locale_and_db("en-US".to_string(), db);
        let document = PdfDocument {
            blocks: vec![Block::Heading {
                level: 1,
                content: vec![Run {
                    text: "你好".to_string(),
                    ..Run::default()
                }],
            }],
            ..PdfDocument::default()
        };
        let bytes = render_with_fallback(&mut fs, &document).unwrap();
        assert!(fs.db().face(rejected_id).is_none());
        assert!(bytes.starts_with(b"%PDF"));
        assert!(
            bytes
                .windows(b"NotoSansSC-Regular".len())
                .any(|window| window == b"NotoSansSC-Regular")
        );
    }
}
