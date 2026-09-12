use crate::types::{
    DocumentId, MAX_PAGE_COUNT, PageGeometry, PageRaster, PdfErrorKind, bounded_raster_dimensions,
};
use pdfium_render::prelude::{
    PdfDocument, PdfRenderConfig, Pdfium, PdfiumError, PdfiumInternalError,
};
use std::collections::HashMap;
use std::path::Path;

pub trait PdfRendererBackend {
    fn open(
        &mut self,
        document_id: DocumentId,
        path: &Path,
    ) -> Result<Vec<PageGeometry>, PdfErrorKind>;
    fn render(
        &mut self,
        document_id: DocumentId,
        page_index: u32,
        target_width_px: u32,
    ) -> Result<PageRaster, PdfErrorKind>;
    fn close(&mut self, document_id: DocumentId);
}

pub struct NativePdfBackend {
    pdfium: &'static Pdfium,
    documents: HashMap<DocumentId, PdfDocument<'static>>,
}

impl NativePdfBackend {
    pub fn load(runtime_path: &Path) -> Result<Self, PdfErrorKind> {
        if !runtime_path.is_file() {
            return Err(PdfErrorKind::RuntimeMissing);
        }
        let bindings =
            Pdfium::bind_to_library(runtime_path).map_err(|_| PdfErrorKind::RuntimeUnavailable)?;
        // pdfium-render promotes bindings into a process-global OnceCell. Leaking
        // this small facade gives documents the real process lifetime of those
        // bindings without exposing a self-referential owner.
        let pdfium = Box::leak(Box::new(Pdfium::new(bindings)));
        Ok(Self {
            pdfium,
            documents: HashMap::new(),
        })
    }
}

impl PdfRendererBackend for NativePdfBackend {
    fn open(
        &mut self,
        document_id: DocumentId,
        path: &Path,
    ) -> Result<Vec<PageGeometry>, PdfErrorKind> {
        if self.documents.contains_key(&document_id) {
            return Err(PdfErrorKind::Internal);
        }
        let document = self
            .pdfium
            .load_pdf_from_file(path, None)
            .map_err(map_open_error)?;
        let page_count =
            u32::try_from(document.pages().len()).map_err(|_| PdfErrorKind::PageLimitExceeded)?;
        if page_count == 0 {
            return Err(PdfErrorKind::CorruptOrUnsupported);
        }
        if page_count > MAX_PAGE_COUNT {
            return Err(PdfErrorKind::PageLimitExceeded);
        }

        let mut geometry = Vec::with_capacity(page_count as usize);
        for page_index in 0..page_count {
            let page = document
                .pages()
                .get(page_index as i32)
                .map_err(map_page_error)?;
            geometry.push(PageGeometry::new(page.width().value, page.height().value)?);
        }
        self.documents.insert(document_id, document);
        Ok(geometry)
    }

    fn render(
        &mut self,
        document_id: DocumentId,
        page_index: u32,
        target_width_px: u32,
    ) -> Result<PageRaster, PdfErrorKind> {
        let document = self
            .documents
            .get(&document_id)
            .ok_or(PdfErrorKind::StaleRequest)?;
        let page = document
            .pages()
            .get(i32::try_from(page_index).map_err(|_| PdfErrorKind::InvalidPageNumber)?)
            .map_err(map_page_error)?;
        let geometry = PageGeometry::new(page.width().value, page.height().value)?;
        let (width, height) = bounded_raster_dimensions(geometry, target_width_px)?;
        let config = PdfRenderConfig::new()
            .set_target_size(width as i32, height as i32)
            .render_annotations(false)
            .render_form_data(false);
        let bitmap = page.render_with_config(&config).map_err(map_page_error)?;
        let width = u32::try_from(bitmap.width()).map_err(|_| PdfErrorKind::InvalidRaster)?;
        let height = u32::try_from(bitmap.height()).map_err(|_| PdfErrorKind::InvalidRaster)?;
        PageRaster::new(width, height, bitmap.as_rgba_bytes())
    }

    fn close(&mut self, document_id: DocumentId) {
        self.documents.remove(&document_id);
    }
}

fn map_open_error(error: PdfiumError) -> PdfErrorKind {
    match error {
        PdfiumError::PdfiumLibraryInternalError(PdfiumInternalError::PasswordError)
        | PdfiumError::PdfiumLibraryInternalError(PdfiumInternalError::SecurityError) => {
            PdfErrorKind::Encrypted
        }
        PdfiumError::PdfiumLibraryInternalError(PdfiumInternalError::FileError)
        | PdfiumError::IoError(_) => PdfErrorKind::FileUnavailable,
        PdfiumError::PdfiumLibraryInternalError(PdfiumInternalError::FormatError) => {
            PdfErrorKind::CorruptOrUnsupported
        }
        _ => PdfErrorKind::CorruptOrUnsupported,
    }
}

fn map_page_error(_error: PdfiumError) -> PdfErrorKind {
    PdfErrorKind::PageUnavailable
}
