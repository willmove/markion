//! Public entry point (task 2.9): wires fonts → layout → emission.

use crate::ir::PdfDocument;
use crate::{PdfError, emit, fonts, layout, text};

/// Renders the document IR to PDF bytes.
pub fn render(document: &PdfDocument) -> Result<Vec<u8>, PdfError> {
    fonts::with_font_system(|fs| {
        let mut cache = text::FontCache::default();
        let laid_out = layout::layout_document(fs, &mut cache, document)?;
        emit::emit_document(&laid_out, &document.metadata)
    })
}
