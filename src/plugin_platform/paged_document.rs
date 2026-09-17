use std::{
    fs,
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};

#[cfg(test)]
use markion_plugin_protocol::PAGED_DOCUMENT_CAPABILITY;
use markion_plugin_protocol::{
    HostMessage, PageGeometry, PagedDocumentRequest, PagedDocumentResponse, PixelFormat,
    PluginError, PluginMessage, ProcessResponse,
};
use thiserror::Error;

use super::{PluginSession, PluginSupervisorError, SupervisedRequest};

pub const DEFAULT_MAX_SOURCE_BYTES: u64 = 512 * 1_048_576;
pub const DEFAULT_MAX_PAGES: u32 = 10_000;
pub const DEFAULT_MAX_RASTER_BYTES: u64 = 32 * 1_048_576;
const MAX_PAGE_POINTS: u32 = 1_000_000;
const MAX_RASTER_DIMENSION: u32 = 32_768;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PagedDocumentLimits {
    pub max_source_bytes: u64,
    pub max_pages: u32,
    pub max_raster_bytes: u64,
    pub request_timeout: Duration,
}

impl Default for PagedDocumentLimits {
    fn default() -> Self {
        Self {
            max_source_bytes: DEFAULT_MAX_SOURCE_BYTES,
            max_pages: DEFAULT_MAX_PAGES,
            max_raster_bytes: DEFAULT_MAX_RASTER_BYTES,
            request_timeout: Duration::from_secs(30),
        }
    }
}

#[derive(Clone)]
pub struct PagedDocument {
    session: PluginSession,
    path: PathBuf,
    document_token: u64,
    pages: Arc<[PageGeometry]>,
    limits: PagedDocumentLimits,
}

impl PagedDocument {
    pub fn open(
        session: PluginSession,
        path: impl AsRef<Path>,
        limits: PagedDocumentLimits,
    ) -> Result<Self, PagedDocumentError> {
        if limits.max_source_bytes == 0
            || limits.max_pages == 0
            || limits.max_raster_bytes == 0
            || limits.request_timeout.is_zero()
        {
            return Err(PagedDocumentError::InvalidLimits);
        }
        let path = canonical_source(path.as_ref(), limits.max_source_bytes)?;
        let response = session.request(
            HostMessage::PagedDocument(PagedDocumentRequest::Open {
                path: path.display().to_string(),
                max_source_bytes: limits.max_source_bytes,
                max_pages: limits.max_pages,
            }),
            Vec::new(),
            limits.request_timeout,
        )?;
        let (document_token, pages) = validate_open_response(response, limits.max_pages)?;
        Ok(Self {
            session,
            path,
            document_token,
            pages: pages.into(),
            limits,
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn document_token(&self) -> u64 {
        self.document_token
    }

    pub fn pages(&self) -> &Arc<[PageGeometry]> {
        &self.pages
    }

    pub fn begin_render(
        &self,
        generation: u64,
        page_index: u32,
        width_px: u32,
    ) -> Result<PagedRenderRequest, PagedDocumentError> {
        if page_index as usize >= self.pages.len() || width_px == 0 {
            return Err(PagedDocumentError::InvalidRequest);
        }
        let pending = self.session.begin_request(
            HostMessage::PagedDocument(PagedDocumentRequest::Render {
                document_token: self.document_token,
                generation,
                page_index,
                width_px,
                max_raster_bytes: self.limits.max_raster_bytes,
            }),
            Vec::new(),
        )?;
        Ok(PagedRenderRequest {
            pending: Some(pending),
            document_token: self.document_token,
            generation,
            page_index,
            max_raster_bytes: self.limits.max_raster_bytes,
            timeout: self.limits.request_timeout,
        })
    }

    pub fn render(
        &self,
        generation: u64,
        page_index: u32,
        width_px: u32,
    ) -> Result<PagedRender, PagedDocumentError> {
        self.begin_render(generation, page_index, width_px)?.wait()
    }

    pub fn close(&self) -> Result<(), PagedDocumentError> {
        let response = self.session.request(
            HostMessage::PagedDocument(PagedDocumentRequest::Close {
                document_token: self.document_token,
            }),
            Vec::new(),
            self.limits.request_timeout,
        )?;
        match response.message {
            PluginMessage::PagedDocument(PagedDocumentResponse::Closed { document_token })
                if document_token == self.document_token && response.body.is_empty() =>
            {
                Ok(())
            }
            PluginMessage::Error(error) => Err(PagedDocumentError::Plugin(error)),
            _ => Err(PagedDocumentError::InvalidResponse),
        }
    }
}

pub struct PagedRenderRequest {
    pending: Option<SupervisedRequest>,
    document_token: u64,
    generation: u64,
    page_index: u32,
    max_raster_bytes: u64,
    timeout: Duration,
}

impl PagedRenderRequest {
    pub fn request_id(&self) -> u64 {
        self.pending
            .as_ref()
            .map(SupervisedRequest::request_id)
            .unwrap_or(0)
    }

    pub fn cancel(mut self) -> Result<(), PagedDocumentError> {
        if let Some(pending) = self.pending.take() {
            pending.cancel()?;
        }
        Ok(())
    }

    pub fn wait(mut self) -> Result<PagedRender, PagedDocumentError> {
        let response = self
            .pending
            .take()
            .expect("paged render request consumed")
            .wait(self.timeout)?;
        validate_render_response(
            response,
            self.document_token,
            self.generation,
            self.page_index,
            self.max_raster_bytes,
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PagedRender {
    pub document_token: u64,
    pub generation: u64,
    pub page_index: u32,
    pub width: u32,
    pub height: u32,
    pub stride: u32,
    pub rgba: Vec<u8>,
}

fn canonical_source(path: &Path, max_source_bytes: u64) -> Result<PathBuf, PagedDocumentError> {
    let metadata = fs::metadata(path).map_err(|_| PagedDocumentError::SourceUnavailable)?;
    if !metadata.is_file() {
        return Err(PagedDocumentError::SourceUnavailable);
    }
    if metadata.len() > max_source_bytes {
        return Err(PagedDocumentError::SourceTooLarge);
    }
    path.canonicalize()
        .map_err(|_| PagedDocumentError::SourceUnavailable)
}

fn validate_open_response(
    response: ProcessResponse,
    max_pages: u32,
) -> Result<(u64, Vec<PageGeometry>), PagedDocumentError> {
    if !response.body.is_empty() {
        return Err(PagedDocumentError::InvalidResponse);
    }
    match response.message {
        PluginMessage::PagedDocument(PagedDocumentResponse::Opened {
            document_token,
            pages,
        }) => {
            if document_token == 0
                || pages.is_empty()
                || pages.len() > max_pages as usize
                || pages.iter().any(|page| {
                    page.width_points == 0
                        || page.height_points == 0
                        || page.width_points > MAX_PAGE_POINTS
                        || page.height_points > MAX_PAGE_POINTS
                })
            {
                return Err(PagedDocumentError::InvalidResponse);
            }
            Ok((document_token, pages))
        }
        PluginMessage::Error(error) => Err(PagedDocumentError::Plugin(error)),
        _ => Err(PagedDocumentError::InvalidResponse),
    }
}

fn validate_render_response(
    response: ProcessResponse,
    document_token: u64,
    generation: u64,
    page_index: u32,
    max_raster_bytes: u64,
) -> Result<PagedRender, PagedDocumentError> {
    match response.message {
        PluginMessage::PagedDocument(PagedDocumentResponse::Rendered {
            document_token: actual_document,
            generation: actual_generation,
            page_index: actual_page,
            raster,
        }) => {
            if actual_document != document_token
                || actual_generation != generation
                || actual_page != page_index
                || raster.width == 0
                || raster.height == 0
                || raster.width > MAX_RASTER_DIMENSION
                || raster.height > MAX_RASTER_DIMENSION
                || raster.pixel_format != PixelFormat::Rgba8
            {
                return Err(PagedDocumentError::InvalidResponse);
            }
            let stride = raster
                .width
                .checked_mul(4)
                .ok_or(PagedDocumentError::InvalidResponse)?;
            let length = u64::from(stride)
                .checked_mul(u64::from(raster.height))
                .ok_or(PagedDocumentError::InvalidResponse)?;
            if raster.stride != stride
                || raster.body_length != length
                || length > max_raster_bytes
                || usize::try_from(length).ok() != Some(response.body.len())
            {
                return Err(PagedDocumentError::InvalidResponse);
            }
            Ok(PagedRender {
                document_token,
                generation,
                page_index,
                width: raster.width,
                height: raster.height,
                stride,
                rgba: response.body,
            })
        }
        PluginMessage::Error(error) => Err(PagedDocumentError::Plugin(error)),
        _ => Err(PagedDocumentError::InvalidResponse),
    }
}

#[derive(Debug, Error)]
pub enum PagedDocumentError {
    #[error("paged-document limits are invalid")]
    InvalidLimits,
    #[error("source file is unavailable")]
    SourceUnavailable,
    #[error("source file exceeds its admitted size")]
    SourceTooLarge,
    #[error("paged-document request is invalid")]
    InvalidRequest,
    #[error("paged-document provider returned an invalid response")]
    InvalidResponse,
    #[error("paged-document provider returned {0:?}")]
    Plugin(PluginError),
    #[error("paged-document supervisor failed")]
    Supervisor(#[from] PluginSupervisorError),
}

#[cfg(test)]
mod tests {
    use markion_plugin_protocol::{PluginErrorCode, ProtocolVersion, RasterDescriptor};

    use super::*;

    fn response(message: PluginMessage, body: Vec<u8>) -> ProcessResponse {
        ProcessResponse {
            request_id: 1,
            generation: 7,
            message,
            body,
        }
    }

    #[test]
    fn open_response_rejects_empty_oversized_and_invalid_geometry() {
        for pages in [
            Vec::new(),
            vec![PageGeometry {
                width_points: 0,
                height_points: 10,
            }],
            vec![PageGeometry {
                width_points: MAX_PAGE_POINTS + 1,
                height_points: 10,
            }],
        ] {
            assert!(matches!(
                validate_open_response(
                    response(
                        PluginMessage::PagedDocument(PagedDocumentResponse::Opened {
                            document_token: 1,
                            pages,
                        }),
                        Vec::new(),
                    ),
                    10,
                ),
                Err(PagedDocumentError::InvalidResponse)
            ));
        }
    }

    #[test]
    fn render_response_enforces_identity_stride_body_and_limit() {
        let descriptor = RasterDescriptor {
            width: 2,
            height: 2,
            stride: 8,
            pixel_format: PixelFormat::Rgba8,
            body_length: 16,
        };
        let valid = response(
            PluginMessage::PagedDocument(PagedDocumentResponse::Rendered {
                document_token: 4,
                generation: 9,
                page_index: 1,
                raster: descriptor.clone(),
            }),
            vec![0; 16],
        );
        assert!(validate_render_response(valid, 4, 9, 1, 32).is_ok());

        for (raster, body) in [
            (
                RasterDescriptor {
                    stride: 7,
                    ..descriptor.clone()
                },
                vec![0; 16],
            ),
            (descriptor.clone(), vec![0; 15]),
        ] {
            assert!(matches!(
                validate_render_response(
                    response(
                        PluginMessage::PagedDocument(PagedDocumentResponse::Rendered {
                            document_token: 4,
                            generation: 9,
                            page_index: 1,
                            raster,
                        }),
                        body,
                    ),
                    4,
                    9,
                    1,
                    32,
                ),
                Err(PagedDocumentError::InvalidResponse)
            ));
        }
    }

    #[test]
    fn provider_errors_remain_typed_and_safe() {
        let error = PluginError {
            code: PluginErrorCode::EncryptedDocument,
            retryable: false,
        };
        assert!(matches!(
            validate_open_response(response(PluginMessage::Error(error), Vec::new()), 10),
            Err(PagedDocumentError::Plugin(PluginError {
                code: PluginErrorCode::EncryptedDocument,
                ..
            }))
        ));
        assert_eq!(ProtocolVersion::V1_0.major, 1);
        assert_eq!(PAGED_DOCUMENT_CAPABILITY, "paged-document/v1");
    }
}
