use std::fmt;
use std::path::PathBuf;

pub const MAX_SOURCE_BYTES: u64 = 512 * 1_048_576;
pub const MAX_PAGE_COUNT: u32 = 10_000;
pub const MAX_RASTER_BYTES: usize = 32 * 1_048_576;
pub const MAX_COMMAND_QUEUE: usize = 32;
pub const RASTER_WIDTH_BUCKET_PX: u32 = 64;
const RGBA_BYTES_PER_PIXEL: u64 = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct RequestId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct DocumentId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Generation(pub u64);

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PageGeometry {
    pub width_points: f32,
    pub height_points: f32,
}

impl PageGeometry {
    pub fn new(width_points: f32, height_points: f32) -> Result<Self, PdfErrorKind> {
        if !width_points.is_finite()
            || !height_points.is_finite()
            || width_points <= 0.0
            || height_points <= 0.0
        {
            return Err(PdfErrorKind::InvalidPageGeometry);
        }
        Ok(Self {
            width_points,
            height_points,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OpenedDocument {
    pub document_id: DocumentId,
    pub path: PathBuf,
    pub pages: Vec<PageGeometry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PageRaster {
    pub width_px: u32,
    pub height_px: u32,
    pub rgba: Vec<u8>,
}

impl PageRaster {
    pub fn new(width_px: u32, height_px: u32, rgba: Vec<u8>) -> Result<Self, PdfErrorKind> {
        let expected = raster_byte_len(width_px, height_px)?;
        if expected > MAX_RASTER_BYTES {
            return Err(PdfErrorKind::RasterTooLarge);
        }
        if rgba.len() != expected {
            return Err(PdfErrorKind::InvalidRaster);
        }
        Ok(Self {
            width_px,
            height_px,
            rgba,
        })
    }

    pub fn byte_len(&self) -> usize {
        self.rgba.len()
    }

    pub(crate) fn validate(&self) -> Result<(), PdfErrorKind> {
        let expected = raster_byte_len(self.width_px, self.height_px)?;
        if expected > MAX_RASTER_BYTES {
            return Err(PdfErrorKind::RasterTooLarge);
        }
        if expected != self.rgba.len() {
            return Err(PdfErrorKind::InvalidRaster);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum RenderPriority {
    Adjacent,
    Visible,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenRequest {
    pub request_id: RequestId,
    pub path: PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RenderRequest {
    pub document_id: DocumentId,
    pub generation: Generation,
    pub page_index: u32,
    pub target_width_px: u32,
    pub priority: RenderPriority,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PdfErrorKind {
    RuntimeMissing,
    RuntimeUnavailable,
    FileUnavailable,
    SourceTooLarge,
    PageLimitExceeded,
    InvalidPageGeometry,
    InvalidPageNumber,
    RasterTooLarge,
    InvalidRaster,
    Encrypted,
    CorruptOrUnsupported,
    PageUnavailable,
    QueueFull,
    StaleRequest,
    ServiceStopped,
    Internal,
}

impl fmt::Display for PdfErrorKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let safe = match self {
            Self::RuntimeMissing => "PDF runtime is missing",
            Self::RuntimeUnavailable => "PDF runtime is unavailable",
            Self::FileUnavailable => "PDF file is unavailable",
            Self::SourceTooLarge => "PDF file exceeds the supported size",
            Self::PageLimitExceeded => "PDF document has too many pages",
            Self::InvalidPageGeometry => "PDF page geometry is invalid",
            Self::InvalidPageNumber => "PDF page number is invalid",
            Self::RasterTooLarge => "PDF page exceeds the raster limit",
            Self::InvalidRaster => "PDF page raster is invalid",
            Self::Encrypted => "Encrypted PDFs are unavailable",
            Self::CorruptOrUnsupported => "PDF is corrupt or unsupported",
            Self::PageUnavailable => "PDF page is unavailable",
            Self::QueueFull => "PDF renderer queue is full",
            Self::StaleRequest => "PDF render request is stale",
            Self::ServiceStopped => "PDF renderer service has stopped",
            Self::Internal => "PDF rendering failed",
        };
        formatter.write_str(safe)
    }
}

impl std::error::Error for PdfErrorKind {}

pub fn target_width_bucket(target_width_px: u32) -> Result<u32, PdfErrorKind> {
    if target_width_px == 0 {
        return Err(PdfErrorKind::InvalidPageGeometry);
    }
    let bucket = (u64::from(target_width_px) + u64::from(RASTER_WIDTH_BUCKET_PX - 1))
        / u64::from(RASTER_WIDTH_BUCKET_PX)
        * u64::from(RASTER_WIDTH_BUCKET_PX);
    u32::try_from(bucket).map_err(|_| PdfErrorKind::RasterTooLarge)
}

pub fn bounded_raster_dimensions(
    geometry: PageGeometry,
    target_width_px: u32,
) -> Result<(u32, u32), PdfErrorKind> {
    let bucketed_width = target_width_bucket(target_width_px)?;
    let aspect = f64::from(geometry.height_points) / f64::from(geometry.width_points);
    if !aspect.is_finite() || aspect <= 0.0 {
        return Err(PdfErrorKind::InvalidPageGeometry);
    }

    let mut width = u64::from(bucketed_width);
    let mut height = ((width as f64 * aspect).ceil() as u64).max(1);
    let pixel_limit = (MAX_RASTER_BYTES as u64) / RGBA_BYTES_PER_PIXEL;
    let pixels = width
        .checked_mul(height)
        .ok_or(PdfErrorKind::RasterTooLarge)?;
    if pixels > pixel_limit {
        let scale = (pixel_limit as f64 / pixels as f64).sqrt();
        width = ((width as f64 * scale).floor() as u64).max(1);
        height = ((height as f64 * scale).floor() as u64).max(1);
        while width.saturating_mul(height) > pixel_limit {
            if width >= height {
                width -= 1;
            } else {
                height -= 1;
            }
        }
    }

    let width = u32::try_from(width).map_err(|_| PdfErrorKind::RasterTooLarge)?;
    let height = u32::try_from(height).map_err(|_| PdfErrorKind::RasterTooLarge)?;
    let _ = raster_byte_len(width, height)?;
    Ok((width, height))
}

fn raster_byte_len(width_px: u32, height_px: u32) -> Result<usize, PdfErrorKind> {
    let bytes = u64::from(width_px)
        .checked_mul(u64::from(height_px))
        .and_then(|pixels| pixels.checked_mul(RGBA_BYTES_PER_PIXEL))
        .ok_or(PdfErrorKind::RasterTooLarge)?;
    usize::try_from(bytes).map_err(|_| PdfErrorKind::RasterTooLarge)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn width_buckets_round_up_without_sub_pixel_enlargement() {
        assert_eq!(target_width_bucket(1), Ok(64));
        assert_eq!(target_width_bucket(64), Ok(64));
        assert_eq!(target_width_bucket(65), Ok(128));
        assert_eq!(
            target_width_bucket(0),
            Err(PdfErrorKind::InvalidPageGeometry)
        );
    }

    #[test]
    fn raster_dimensions_reduce_before_the_byte_limit() {
        let geometry = PageGeometry::new(1.0, 1000.0).unwrap();
        let (width, height) = bounded_raster_dimensions(geometry, 4096).unwrap();
        assert!(u64::from(width) * u64::from(height) * 4 <= MAX_RASTER_BYTES as u64);
        assert!(width < 4096);
    }

    #[test]
    fn raster_requires_exact_rgba_length_and_limit() {
        assert!(PageRaster::new(2, 2, vec![0; 16]).is_ok());
        assert_eq!(
            PageRaster::new(2, 2, vec![0; 15]),
            Err(PdfErrorKind::InvalidRaster)
        );
        assert_eq!(
            PageRaster::new(4096, 4096, Vec::new()),
            Err(PdfErrorKind::RasterTooLarge)
        );
    }
}
