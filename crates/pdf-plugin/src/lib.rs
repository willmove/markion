use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

use markion_pdf_viewer::{
    DocumentId, MAX_PAGE_COUNT, MAX_RASTER_BYTES, MAX_SOURCE_BYTES, PageGeometry as PdfGeometry,
    PdfErrorKind, PdfRendererBackend,
};
use markion_plugin_protocol::{
    PageGeometry, PagedDocumentRequest, PagedDocumentResponse, PixelFormat, PluginError,
    PluginErrorCode, PluginMessage, RasterDescriptor,
};

#[derive(Debug)]
pub struct WorkerReply {
    pub message: PluginMessage,
    pub body: Vec<u8>,
}

pub struct PdfWorker<B: PdfRendererBackend> {
    backend: B,
    next_document_token: u64,
    documents: BTreeSet<u64>,
}

impl<B> PdfWorker<B>
where
    B: PdfRendererBackend,
{
    pub fn new(backend: B) -> Self {
        Self {
            backend,
            next_document_token: 1,
            documents: BTreeSet::new(),
        }
    }

    pub fn handle(&mut self, request: PagedDocumentRequest) -> WorkerReply {
        match self.handle_inner(request) {
            Ok((response, body)) => WorkerReply {
                message: PluginMessage::PagedDocument(response),
                body,
            },
            Err(error) => WorkerReply {
                message: PluginMessage::Error(plugin_error(error)),
                body: Vec::new(),
            },
        }
    }

    fn handle_inner(
        &mut self,
        request: PagedDocumentRequest,
    ) -> Result<(PagedDocumentResponse, Vec<u8>), PdfErrorKind> {
        match request {
            PagedDocumentRequest::Open {
                path,
                max_source_bytes,
                max_pages,
            } => {
                if max_source_bytes == 0 || max_pages == 0 {
                    return Err(PdfErrorKind::Internal);
                }
                let path = canonical_pdf_path(Path::new(&path), max_source_bytes)?;
                let token = self.next_document_token;
                self.next_document_token = self
                    .next_document_token
                    .checked_add(1)
                    .ok_or(PdfErrorKind::Internal)?;
                let document_id = DocumentId(token);
                let pages = self.backend.open(document_id, &path)?;
                let page_limit = usize::try_from(max_pages.min(MAX_PAGE_COUNT))
                    .map_err(|_| PdfErrorKind::PageLimitExceeded)?;
                if pages.is_empty() || pages.len() > page_limit {
                    self.backend.close(document_id);
                    return Err(if pages.is_empty() {
                        PdfErrorKind::CorruptOrUnsupported
                    } else {
                        PdfErrorKind::PageLimitExceeded
                    });
                }
                let pages = match pages.into_iter().map(protocol_geometry).collect() {
                    Ok(pages) => pages,
                    Err(error) => {
                        self.backend.close(document_id);
                        return Err(error);
                    }
                };
                self.documents.insert(token);
                Ok((
                    PagedDocumentResponse::Opened {
                        document_token: token,
                        pages,
                    },
                    Vec::new(),
                ))
            }
            PagedDocumentRequest::Render {
                document_token,
                generation,
                page_index,
                width_px,
                max_raster_bytes,
            } => {
                if !self.documents.contains(&document_token) {
                    return Err(PdfErrorKind::StaleRequest);
                }
                let admitted = usize::try_from(max_raster_bytes)
                    .unwrap_or(usize::MAX)
                    .min(MAX_RASTER_BYTES);
                if admitted == 0 {
                    return Err(PdfErrorKind::RasterTooLarge);
                }
                let raster =
                    self.backend
                        .render(DocumentId(document_token), page_index, width_px)?;
                if raster.byte_len() > admitted {
                    return Err(PdfErrorKind::RasterTooLarge);
                }
                let stride = raster
                    .width_px
                    .checked_mul(4)
                    .ok_or(PdfErrorKind::InvalidRaster)?;
                let body_length =
                    u64::try_from(raster.byte_len()).map_err(|_| PdfErrorKind::RasterTooLarge)?;
                Ok((
                    PagedDocumentResponse::Rendered {
                        document_token,
                        generation,
                        page_index,
                        raster: RasterDescriptor {
                            width: raster.width_px,
                            height: raster.height_px,
                            stride,
                            pixel_format: PixelFormat::Rgba8,
                            body_length,
                        },
                    },
                    raster.rgba,
                ))
            }
            PagedDocumentRequest::Close { document_token } => {
                if self.documents.remove(&document_token) {
                    self.backend.close(DocumentId(document_token));
                }
                Ok((PagedDocumentResponse::Closed { document_token }, Vec::new()))
            }
        }
    }

    pub fn close_all(&mut self) {
        for token in std::mem::take(&mut self.documents) {
            self.backend.close(DocumentId(token));
        }
    }
}

impl<B: PdfRendererBackend> Drop for PdfWorker<B> {
    fn drop(&mut self) {
        for token in std::mem::take(&mut self.documents) {
            self.backend.close(DocumentId(token));
        }
    }
}

pub fn runtime_path_from_executable(executable: &Path) -> Result<PathBuf, PdfErrorKind> {
    let bin = executable.parent().ok_or(PdfErrorKind::RuntimeMissing)?;
    let root = bin.parent().ok_or(PdfErrorKind::RuntimeMissing)?;
    let runtime = root
        .join("resources")
        .join(markion_pdf_viewer::runtime_library_name());
    if runtime.is_file() {
        Ok(runtime)
    } else {
        Err(PdfErrorKind::RuntimeMissing)
    }
}

fn canonical_pdf_path(path: &Path, host_limit: u64) -> Result<PathBuf, PdfErrorKind> {
    let metadata = fs::metadata(path).map_err(|_| PdfErrorKind::FileUnavailable)?;
    if !metadata.is_file() {
        return Err(PdfErrorKind::FileUnavailable);
    }
    if metadata.len() > host_limit.min(MAX_SOURCE_BYTES) {
        return Err(PdfErrorKind::SourceTooLarge);
    }
    path.canonicalize()
        .map_err(|_| PdfErrorKind::FileUnavailable)
}

fn protocol_geometry(geometry: PdfGeometry) -> Result<PageGeometry, PdfErrorKind> {
    let width_points = bounded_points(geometry.width_points)?;
    let height_points = bounded_points(geometry.height_points)?;
    Ok(PageGeometry {
        width_points,
        height_points,
    })
}

fn bounded_points(points: f32) -> Result<u32, PdfErrorKind> {
    if !points.is_finite() || points <= 0.0 || points > u32::MAX as f32 {
        return Err(PdfErrorKind::InvalidPageGeometry);
    }
    Ok(points.round().max(1.0) as u32)
}

fn plugin_error(error: PdfErrorKind) -> PluginError {
    let (code, retryable) = match error {
        PdfErrorKind::SourceTooLarge => (PluginErrorCode::SourceTooLarge, false),
        PdfErrorKind::PageLimitExceeded => (PluginErrorCode::TooManyPages, false),
        PdfErrorKind::RasterTooLarge | PdfErrorKind::InvalidRaster => {
            (PluginErrorCode::RasterTooLarge, false)
        }
        PdfErrorKind::Encrypted => (PluginErrorCode::EncryptedDocument, false),
        PdfErrorKind::CorruptOrUnsupported => (PluginErrorCode::CorruptDocument, false),
        PdfErrorKind::RuntimeMissing | PdfErrorKind::RuntimeUnavailable => {
            (PluginErrorCode::RuntimeUnavailable, true)
        }
        PdfErrorKind::QueueFull => (PluginErrorCode::Internal, true),
        PdfErrorKind::StaleRequest => (PluginErrorCode::Cancelled, false),
        PdfErrorKind::FileUnavailable
        | PdfErrorKind::InvalidPageGeometry
        | PdfErrorKind::InvalidPageNumber
        | PdfErrorKind::PageUnavailable => (PluginErrorCode::InvalidRequest, false),
        PdfErrorKind::ServiceStopped | PdfErrorKind::Internal => (PluginErrorCode::Internal, true),
    };
    PluginError { code, retryable }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use markion_pdf_viewer::PageRaster;
    use markion_plugin_protocol::PagedDocumentResponse;

    use super::*;

    #[derive(Default)]
    struct FakeBackend {
        documents: BTreeMap<DocumentId, PathBuf>,
        closed: Vec<DocumentId>,
    }

    impl PdfRendererBackend for FakeBackend {
        fn open(
            &mut self,
            document_id: DocumentId,
            path: &Path,
        ) -> Result<Vec<PdfGeometry>, PdfErrorKind> {
            self.documents.insert(document_id, path.to_path_buf());
            Ok(vec![PdfGeometry::new(612.0, 792.0).unwrap()])
        }

        fn render(
            &mut self,
            document_id: DocumentId,
            _page_index: u32,
            _target_width_px: u32,
        ) -> Result<PageRaster, PdfErrorKind> {
            if !self.documents.contains_key(&document_id) {
                return Err(PdfErrorKind::StaleRequest);
            }
            PageRaster::new(2, 2, vec![255; 16])
        }

        fn close(&mut self, document_id: DocumentId) {
            self.documents.remove(&document_id);
            self.closed.push(document_id);
        }
    }

    #[test]
    fn open_render_and_close_map_to_the_generic_capability() {
        let temporary = tempfile::tempdir().unwrap();
        let pdf = temporary.path().join("fixture.pdf");
        fs::write(&pdf, b"fixture").unwrap();
        let mut worker = PdfWorker::new(FakeBackend::default());
        let opened = worker.handle(PagedDocumentRequest::Open {
            path: pdf.display().to_string(),
            max_source_bytes: MAX_SOURCE_BYTES,
            max_pages: MAX_PAGE_COUNT,
        });
        assert!(matches!(
            opened.message,
            PluginMessage::PagedDocument(PagedDocumentResponse::Opened {
                document_token: 1,
                ref pages,
            }) if pages == &[PageGeometry { width_points: 612, height_points: 792 }]
        ));

        let rendered = worker.handle(PagedDocumentRequest::Render {
            document_token: 1,
            generation: 9,
            page_index: 0,
            width_px: 512,
            max_raster_bytes: MAX_RASTER_BYTES as u64,
        });
        assert_eq!(rendered.body, vec![255; 16]);
        assert!(matches!(
            rendered.message,
            PluginMessage::PagedDocument(PagedDocumentResponse::Rendered {
                generation: 9,
                raster: RasterDescriptor {
                    body_length: 16,
                    ..
                },
                ..
            })
        ));

        let closed = worker.handle(PagedDocumentRequest::Close { document_token: 1 });
        assert!(matches!(
            closed.message,
            PluginMessage::PagedDocument(PagedDocumentResponse::Closed { document_token: 1 })
        ));
    }

    #[test]
    fn source_and_raster_limits_fail_closed() {
        let temporary = tempfile::tempdir().unwrap();
        let pdf = temporary.path().join("fixture.pdf");
        fs::write(&pdf, b"too large").unwrap();
        let mut worker = PdfWorker::new(FakeBackend::default());
        assert!(matches!(
            worker
                .handle(PagedDocumentRequest::Open {
                    path: pdf.display().to_string(),
                    max_source_bytes: 1,
                    max_pages: 1,
                })
                .message,
            PluginMessage::Error(PluginError {
                code: PluginErrorCode::SourceTooLarge,
                ..
            })
        ));
    }
}
