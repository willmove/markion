//! Bounded Markdown preview-image cache.
//!
//! Decoded bitmaps are owned by Markion and presented via `ImageSource::Render`,
//! so they are not retained forever in GPUI's unbounded `loading_assets` table.

use super::*;
use gpui::RenderImage;
use image::imageops::FilterType;
use image::{DynamicImage, GenericImageView, ImageReader, RgbaImage};
use std::io::Cursor;

pub(super) const PREVIEW_IMAGE_CACHE_CAPACITY: usize = 64;
pub(super) const PREVIEW_IMAGE_CACHE_MAX_BYTES: usize = 64 * 1024 * 1024;
/// Longer edge of retained preview bitmaps, in device pixels.
pub(super) const PREVIEW_IMAGE_MAX_EDGE: u32 = 2048;
/// Overall in-flight fetch/decode safety cap (parallel warm for typical docs).
pub(super) const PREVIEW_IMAGE_DECODE_CONCURRENCY: usize = 8;
/// Tighter cap for probed oversized ("heavy") sources only.
pub(super) const PREVIEW_IMAGE_HEAVY_DECODE_CONCURRENCY: usize = 3;
/// SVG rasters carry this many device pixels per presented logical pixel.
/// `RenderImage::scale_factor` is `pub(crate)` in gpui, so pixel density is
/// expressed by supersampling plus an explicit presentation width instead
/// (same approach as the diagram pipeline's `DIAGRAM_SUPERSAMPLE`).
pub(super) const PREVIEW_SVG_SUPERSAMPLE: u32 = 2;

/// Claimed (on-screen) images are never degraded; when they alone exceed the
/// byte budget, retained bytes may overshoot up to this multiple of the budget
/// before the incoming raster is downscaled as a last resort.
const PREVIEW_IMAGE_OVERSHOOT_FACTOR: usize = 2;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(super) struct PreviewImageKey {
    /// `local:` / `remote:` identities hold the full (small) locator. `data:`
    /// identities are `data:{len}:{sha256-hex}` — constant-size, derived from
    /// the complete URI once per document version. The full URI bytes live
    /// next to the pending cache entry (see `PreviewImageCache::data_payloads`)
    /// for decode only.
    pub(super) identity: String,
}

impl PreviewImageKey {
    pub(super) fn from_local_path(path: &Path) -> Self {
        let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
        Self {
            identity: format!("local:{}", canonical.display()),
        }
    }

    pub(super) fn from_data(byte_len: usize, sha256: &[u8; 32]) -> Self {
        let mut hex = String::with_capacity(64);
        for byte in sha256 {
            hex.push_str(&format!("{byte:02x}"));
        }
        Self {
            identity: format!("data:{byte_len}:{hex}"),
        }
    }

    pub(super) fn from_source(
        url: &str,
        identity: &ImageSourceIdentity,
        document_dir: Option<&Path>,
    ) -> Self {
        match identity {
            ImageSourceIdentity::Data { byte_len, sha256 } => Self::from_data(*byte_len, sha256),
            ImageSourceIdentity::FromUrl => Self::from_url(url, document_dir),
        }
    }

    pub(super) fn from_url(url: &str, document_dir: Option<&Path>) -> Self {
        if url.starts_with("data:") {
            // Tests and non-derived call sites still accept a raw data URI.
            // Paint/claim paths must pass a precomputed `ImageSourceIdentity`
            // so they never scan the payload.
            match ImageSourceIdentity::for_url(url) {
                ImageSourceIdentity::Data { byte_len, sha256 } => {
                    Self::from_data(byte_len, &sha256)
                }
                ImageSourceIdentity::FromUrl => unreachable!("data: URIs produce Data identities"),
            }
        } else if is_remote_resource(url) {
            Self {
                identity: format!("remote:{}", remote_image_request_url(url)),
            }
        } else {
            let path = PathBuf::from(url);
            let path = if path.is_absolute() {
                path
            } else if let Some(document_dir) = document_dir {
                document_dir.join(path)
            } else if path.is_file() {
                path
            } else if let Some(bundled) = bundled_resource_path(&path) {
                bundled
            } else {
                path
            };
            Self::from_local_path(&path)
        }
    }

    fn local_path(&self) -> Option<PathBuf> {
        self.identity.strip_prefix("local:").map(PathBuf::from)
    }

    fn remote_url(&self) -> Option<&str> {
        // `from_url` only emits the `remote:` prefix for non-`data:` remote
        // resources, so a successful strip guarantees an HTTP(S)-style URL
        // safe to feed reqwest — `data:` URIs ride the payload side map.
        self.identity.strip_prefix("remote:")
    }

    fn is_data_uri(&self) -> bool {
        self.identity.starts_with("data:")
    }

    fn data_sha256(&self) -> Option<[u8; 32]> {
        let rest = self.identity.strip_prefix("data:")?;
        let hex = rest.split_once(':')?.1;
        if hex.len() != 64 {
            return None;
        }
        let mut out = [0u8; 32];
        for (i, slot) in out.iter_mut().enumerate() {
            *slot = u8::from_str_radix(&hex[i * 2..i * 2 + 2], 16).ok()?;
        }
        Some(out)
    }
}

#[derive(Clone)]
pub(super) struct PreviewImageReady {
    pub(super) image: Arc<RenderImage>,
    pub(super) byte_len: usize,
    pub(super) width: u32,
    pub(super) height: u32,
    /// Presented size in logical pixels. Equal to `width`/`height` for raster
    /// sources; `width / PREVIEW_SVG_SUPERSAMPLE` for supersampled SVG rasters.
    pub(super) display_width: u32,
    pub(super) display_height: u32,
}

#[derive(Clone)]
pub(super) enum PreviewImageEntry {
    Pending,
    Ready(PreviewImageReady),
    Error(Arc<str>),
}

pub(super) struct PreviewImageCache {
    capacity: usize,
    max_completed_bytes: usize,
    completed_bytes: usize,
    entries: HashMap<PreviewImageKey, PreviewImageEntry>,
    /// Ready/error keys in LRU order (front = oldest).
    completed_order: VecDeque<PreviewImageKey>,
    /// Claim counts per key (tabs that currently reference the source).
    claims: HashMap<PreviewImageKey, usize>,
    /// Keys with a fetch/decode task currently running (`true` = heavy).
    in_flight: HashMap<PreviewImageKey, bool>,
    /// Full source bytes for pending data-URI entries. The key identity is a
    /// bounded fingerprint (cheap to rebuild per frame); the decode task reads
    /// the real URI from here. Retained only while the entry is pending —
    /// `complete` and `remove_entry` drop it — so a decoded image does not
    /// keep its multi-megabyte base64 source alive.
    data_payloads: HashMap<PreviewImageKey, Arc<str>>,
}

impl PreviewImageCache {
    pub(super) fn new(capacity: usize) -> Self {
        Self::with_limits(capacity, PREVIEW_IMAGE_CACHE_MAX_BYTES)
    }

    fn with_limits(capacity: usize, max_completed_bytes: usize) -> Self {
        Self {
            capacity,
            max_completed_bytes,
            completed_bytes: 0,
            entries: HashMap::new(),
            completed_order: VecDeque::new(),
            claims: HashMap::new(),
            in_flight: HashMap::new(),
            data_payloads: HashMap::new(),
        }
    }

    pub(super) fn get(&self, key: &PreviewImageKey) -> Option<PreviewImageEntry> {
        self.entries.get(key).cloned()
    }

    pub(super) fn claim(&mut self, key: PreviewImageKey) {
        *self.claims.entry(key).or_insert(0) += 1;
    }

    /// Release one claim. A ready entry whose claim count reaches zero stays
    /// cached as unclaimed LRU (so tab switches reuse decoded images) and only
    /// yields when capacity or the byte budget actually needs it — including
    /// paying down any overshoot accepted while everything was claimed.
    pub(super) fn release(&mut self, key: &PreviewImageKey) -> Vec<Arc<RenderImage>> {
        let mut dropped = Vec::new();
        let Some(count) = self.claims.get_mut(key) else {
            return dropped;
        };
        *count = count.saturating_sub(1);
        if *count > 0 {
            return dropped;
        }
        self.claims.remove(key);
        // A still-pending unclaimed key keeps its slot; `complete` drops
        // unclaimed late completions without retaining them.
        self.enforce_budget(&mut dropped);
        dropped
    }

    pub(super) fn release_all<'a>(
        &mut self,
        keys: impl IntoIterator<Item = &'a PreviewImageKey>,
    ) -> Vec<Arc<RenderImage>> {
        let mut dropped = Vec::new();
        for key in keys {
            dropped.extend(self.release(key));
        }
        dropped
    }

    /// Evict unclaimed completed entries until retained bytes are back under
    /// the budget (or only claimed entries remain).
    fn enforce_budget(&mut self, dropped: &mut Vec<Arc<RenderImage>>) {
        while self.completed_bytes > self.max_completed_bytes {
            if !self.evict_oldest_unclaimed(dropped) {
                break;
            }
        }
    }

    pub(super) fn claim_count(&self, key: &PreviewImageKey) -> usize {
        self.claims.get(key).copied().unwrap_or(0)
    }

    pub(super) fn in_flight_count(&self) -> usize {
        self.in_flight.len()
    }

    pub(super) fn heavy_in_flight_count(&self) -> usize {
        self.in_flight.values().filter(|heavy| **heavy).count()
    }

    /// Pending entries that do not yet have a running decode task.
    pub(super) fn pending_not_started(&self) -> Vec<PreviewImageKey> {
        self.entries
            .iter()
            .filter_map(|(key, entry)| {
                if matches!(entry, PreviewImageEntry::Pending) && !self.in_flight.contains_key(key)
                {
                    Some(key.clone())
                } else {
                    None
                }
            })
            .collect()
    }

    /// Try to mark `key` as in-flight under the overall / heavy caps.
    pub(super) fn try_begin_decode(&mut self, key: &PreviewImageKey, heavy: bool) -> bool {
        if self.in_flight.contains_key(key) {
            return false;
        }
        if !matches!(self.entries.get(key), Some(PreviewImageEntry::Pending)) {
            return false;
        }
        if self.in_flight.len() >= PREVIEW_IMAGE_DECODE_CONCURRENCY {
            return false;
        }
        if heavy && self.heavy_in_flight_count() >= PREVIEW_IMAGE_HEAVY_DECODE_CONCURRENCY {
            return false;
        }
        self.in_flight.insert(key.clone(), heavy);
        true
    }

    pub(super) fn end_decode(&mut self, key: &PreviewImageKey) {
        self.in_flight.remove(key);
    }

    /// Reserve a pending slot. Returns false if the key already exists or the
    /// cache is full of non-evictable pending work.
    pub(super) fn reserve_pending(&mut self, key: PreviewImageKey) -> bool {
        if self.capacity == 0 || self.entries.contains_key(&key) {
            return false;
        }
        while self.entries.len() >= self.capacity {
            if !self.evict_oldest_unclaimed(&mut Vec::new()) {
                return false;
            }
        }
        self.entries.insert(key, PreviewImageEntry::Pending);
        true
    }

    /// Retain the full data-URI source for a freshly reserved pending entry
    /// so the decode task can read it without the key carrying it. Called
    /// once per reservation (not per frame); ignored when the entry vanished.
    pub(super) fn attach_data_payload(&mut self, key: PreviewImageKey, payload: Arc<str>) {
        if key.is_data_uri() && self.entries.contains_key(&key) {
            markion::record_data_uri_payload_clone(payload.len());
            self.data_payloads.insert(key, payload);
        }
    }

    /// The retained data-URI source for a pending entry, if any.
    pub(super) fn data_payload(&self, key: &PreviewImageKey) -> Option<Arc<str>> {
        self.data_payloads.get(key).cloned()
    }

    /// Retained source bytes across pending data-URI entries (memory report).
    pub(super) fn retained_data_payload_bytes(&self) -> usize {
        self.data_payloads
            .values()
            .map(|payload| payload.len())
            .sum()
    }

    pub(super) fn complete(
        &mut self,
        key: &PreviewImageKey,
        result: Result<PreviewImageReady, String>,
    ) -> Vec<Arc<RenderImage>> {
        let mut dropped = Vec::new();
        if !matches!(self.entries.get(key), Some(PreviewImageEntry::Pending)) {
            return dropped;
        }
        // The decode ran (successfully or not); the retained source has no
        // further use either way.
        self.data_payloads.remove(key);
        // Late completion for an unclaimed key: drop without retaining.
        if self.claim_count(key) == 0 {
            self.entries.remove(key);
            if let Ok(ready) = result {
                dropped.push(ready.image);
            }
            return dropped;
        }

        let result = match result {
            Ok(ready) if ready.byte_len > self.max_completed_bytes => Err(format!(
                "image raster ({} bytes) exceeds the preview image cache budget",
                ready.byte_len
            )),
            other => other,
        };

        let result = match result {
            Ok(ready) => match self.fit_ready_under_budget(ready, &mut dropped) {
                Ok(ready) => Ok(ready),
                Err(ready) => {
                    // Even the overshoot ceiling is exhausted by claimed images.
                    dropped.push(ready.image);
                    Err("image raster exceeds the preview image cache budget".into())
                }
            },
            Err(message) => Err(message),
        };

        let entry = match result {
            Ok(ready) => {
                self.completed_bytes = self.completed_bytes.saturating_add(ready.byte_len);
                PreviewImageEntry::Ready(ready)
            }
            Err(message) => PreviewImageEntry::Error(Arc::from(message)),
        };
        if let Some(PreviewImageEntry::Ready(old)) = self.entries.insert(key.clone(), entry) {
            self.completed_bytes = self.completed_bytes.saturating_sub(old.byte_len);
            dropped.push(old.image);
        }
        self.touch_completed(key);
        dropped
    }

    /// Make `ready` fit without touching claimed (on-screen) images.
    ///
    /// Order: free unclaimed LRU → accept bounded budget overshoot while the
    /// claimed set alone exceeds the budget → last-resort downscale of the
    /// *incoming* raster only (a single resample from its freshly decoded
    /// bitmap; existing entries are never mutated).
    fn fit_ready_under_budget(
        &mut self,
        mut ready: PreviewImageReady,
        dropped: &mut Vec<Arc<RenderImage>>,
    ) -> Result<PreviewImageReady, PreviewImageReady> {
        while self.completed_bytes.saturating_add(ready.byte_len) > self.max_completed_bytes {
            if !self.evict_oldest_unclaimed(dropped) {
                break;
            }
        }
        if self.completed_bytes.saturating_add(ready.byte_len) <= self.max_completed_bytes {
            return Ok(ready);
        }

        let ceiling = self
            .max_completed_bytes
            .saturating_mul(PREVIEW_IMAGE_OVERSHOOT_FACTOR);
        if self.completed_bytes.saturating_add(ready.byte_len) <= ceiling {
            return Ok(ready);
        }

        let remaining = ceiling.saturating_sub(self.completed_bytes).max(4);
        if ready.byte_len > remaining {
            let previous = ready.image.clone();
            ready = downscale_ready_to_max_bytes(ready, remaining);
            dropped.push(previous);
        }
        if self.completed_bytes.saturating_add(ready.byte_len) <= ceiling {
            Ok(ready)
        } else {
            Err(ready)
        }
    }

    fn touch_completed(&mut self, key: &PreviewImageKey) {
        self.completed_order.retain(|k| k != key);
        self.completed_order.push_back(key.clone());
    }

    /// Evict the oldest unclaimed completed entry. Returns whether progress was made.
    /// Ready images are appended to `dropped`; Error entries free a slot only.
    fn evict_oldest_unclaimed(&mut self, dropped: &mut Vec<Arc<RenderImage>>) -> bool {
        // Only evict unclaimed completed entries. Evicting a still-claimed
        // (on-screen) ready image forces ensure→redecode→notify in a loop and
        // looks like continuous flicker in image-heavy documents.
        let index = self.completed_order.iter().position(|key| {
            self.claim_count(key) == 0
                && matches!(
                    self.entries.get(key),
                    Some(PreviewImageEntry::Ready(_) | PreviewImageEntry::Error(_))
                )
        });
        let Some(index) = index else {
            return false;
        };
        let oldest = self.completed_order.remove(index).expect("index in range");
        if let Some(image) = self.remove_entry(&oldest) {
            dropped.push(image);
        }
        true
    }

    fn remove_entry(&mut self, key: &PreviewImageKey) -> Option<Arc<RenderImage>> {
        self.completed_order.retain(|k| k != key);
        self.data_payloads.remove(key);
        match self.entries.remove(key) {
            Some(PreviewImageEntry::Ready(ready)) => {
                self.completed_bytes = self.completed_bytes.saturating_sub(ready.byte_len);
                Some(ready.image)
            }
            _ => None,
        }
    }

    pub(super) fn accounting_counts(&self) -> (usize, usize, usize, usize, usize) {
        let mut pending = 0usize;
        let mut ready = 0usize;
        for entry in self.entries.values() {
            match entry {
                PreviewImageEntry::Pending => pending += 1,
                PreviewImageEntry::Ready(_) => ready += 1,
                PreviewImageEntry::Error(_) => {}
            }
        }
        (
            self.entries.len(),
            pending,
            ready,
            self.completed_bytes,
            self.max_completed_bytes,
        )
    }

    #[cfg(test)]
    pub(super) fn completed_bytes(&self) -> usize {
        self.completed_bytes
    }
}

/// Cheap size probe for heavy-slot classification. Remote / unreadable sources
/// return `false` so they favor parallel warm under the overall cap only.
pub(super) fn probe_is_heavy(key: &PreviewImageKey) -> bool {
    let Some(path) = key.local_path() else {
        return false;
    };
    let Ok(reader) = ImageReader::open(&path) else {
        return false;
    };
    let Ok(reader) = reader.with_guessed_format() else {
        return false;
    };
    let Ok((width, height)) = reader.into_dimensions() else {
        return false;
    };
    width.max(height) > PREVIEW_IMAGE_MAX_EDGE
}

/// Decode the source identified by `key`. Data-URI keys are bounded
/// fingerprints, so the full URI bytes arrive separately as `data_payload`
/// (retained next to the pending entry by the cache).
pub(super) fn load_preview_image(
    key: &PreviewImageKey,
    data_payload: Option<&str>,
) -> Result<PreviewImageReady, String> {
    let (bytes, is_svg) = if let Some(path) = key.local_path() {
        let bytes = std::fs::read(&path)
            .map_err(|err| format!("failed to read {}: {err}", path.display()))?;
        let is_svg = path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.eq_ignore_ascii_case("svg"))
            .unwrap_or(false)
            || looks_like_svg(&bytes);
        (bytes, is_svg)
    } else if key.is_data_uri() {
        let url = data_payload.ok_or_else(|| "data URI payload was not retained".to_string())?;
        let (bytes, mime_type) = decode_data_url(url)?;
        let is_svg = mime_type
            .map(|m| m.eq_ignore_ascii_case("image/svg+xml"))
            .unwrap_or(false)
            || looks_like_svg(&bytes);
        (bytes, is_svg)
    } else if let Some(url) = key.remote_url() {
        let bytes = network::fetch_url_bytes(url).map_err(|err| err.to_string())?;
        let is_svg = looks_like_svg(&bytes);
        (bytes, is_svg)
    } else {
        return Err("unsupported image identity".into());
    };

    if is_svg {
        let (rgba, display_width, display_height) = rasterize_svg_bytes(&bytes)?;
        rgba_to_ready(rgba, display_width, display_height)
    } else {
        let rgba = decode_raster_bytes(&bytes, PREVIEW_IMAGE_MAX_EDGE)?;
        let (width, height) = rgba.dimensions();
        rgba_to_ready(rgba, width, height)
    }
}

/// Cheap leading-byte heuristic for SVG payloads, shared across the local /
/// data-URI / remote load branches.
fn looks_like_svg(bytes: &[u8]) -> bool {
    bytes
        .windows(4)
        .take(64)
        .any(|window| window == b"<svg" || window == b"<SVG")
}

/// Decode an RFC 2397 `data:` URL to bytes, returning the parsed MIME type
/// (its `type/subtype` essence, e.g. `image/png` or `image/svg+xml`) so the
/// caller can pick the SVG rasterization path without re-probing the body.
/// Both `;base64` and URL-encoded payloads are supported; a malformed URI
/// yields a `String` error that flows into the missing-resource placeholder.
fn decode_data_url(url: &str) -> Result<(Vec<u8>, Option<String>), String> {
    let processed =
        data_url::DataUrl::process(url).map_err(|err| format!("invalid data URL: {err}"))?;
    let mime_essence = {
        let m = processed.mime_type();
        format!("{}/{}", m.type_, m.subtype)
    };
    let (bytes, _fragment) = processed
        .decode_to_vec()
        .map_err(|err| format!("failed to decode data URL body: {err}"))?;
    Ok((bytes, Some(mime_essence)))
}

fn rgba_to_ready(
    rgba: RgbaImage,
    display_width: u32,
    display_height: u32,
) -> Result<PreviewImageReady, String> {
    let (width, height) = rgba.dimensions();
    let mut pixels = rgba.into_raw();
    // GPUI RenderImage expects BGRA; image crate produces RGBA.
    for pixel in pixels.chunks_exact_mut(4) {
        pixel.swap(0, 2);
    }
    let buffer = image::ImageBuffer::from_raw(width, height, pixels)
        .ok_or_else(|| "decoded buffer does not match dimensions".to_string())?;
    let byte_len = (width as usize)
        .saturating_mul(height as usize)
        .saturating_mul(4);
    Ok(PreviewImageReady {
        image: Arc::new(RenderImage::new(vec![image::Frame::new(buffer)])),
        byte_len,
        width,
        height,
        display_width,
        display_height,
    })
}

/// Shrink a ready raster so its BGRA footprint is at most `max_bytes`.
fn downscale_ready_to_max_bytes(ready: PreviewImageReady, max_bytes: usize) -> PreviewImageReady {
    if ready.byte_len <= max_bytes || ready.width == 0 || ready.height == 0 {
        return ready;
    }
    let max_pixels = (max_bytes / 4).max(1);
    let current_pixels = (ready.width as usize).saturating_mul(ready.height as usize);
    if current_pixels <= max_pixels {
        return ready;
    }
    let scale = (max_pixels as f32 / current_pixels as f32).sqrt();
    let mut new_w = ((ready.width as f32) * scale).floor().max(1.0) as u32;
    let mut new_h = ((ready.height as f32) * scale).floor().max(1.0) as u32;
    while (new_w as usize)
        .saturating_mul(new_h as usize)
        .saturating_mul(4)
        > max_bytes
    {
        if new_w >= new_h && new_w > 1 {
            new_w -= 1;
        } else if new_h > 1 {
            new_h -= 1;
        } else {
            break;
        }
    }
    let Some(bgra) = ready.image.as_bytes(0) else {
        return ready;
    };
    let mut rgba_bytes = bgra.to_vec();
    for pixel in rgba_bytes.chunks_exact_mut(4) {
        pixel.swap(0, 2);
    }
    let Some(rgba) = RgbaImage::from_raw(ready.width, ready.height, rgba_bytes) else {
        return ready;
    };
    let resized = DynamicImage::ImageRgba8(rgba)
        .resize_exact(new_w, new_h, FilterType::Triangle)
        .into_rgba8();
    // Presentation size follows the raster down for plain rasters; a
    // supersampled SVG keeps its intrinsic size while density degrades.
    let display_width = ready.display_width.min(new_w);
    let display_height = ready.display_height.min(new_h);
    rgba_to_ready(resized, display_width, display_height).unwrap_or(ready)
}

/// Decode a non-SVG image: prefer header probe + resize on `DynamicImage` before
/// consuming into RGBA, so a full-resolution RGBA intermediate is not retained
/// only to downsample it afterward.
fn decode_raster_bytes(bytes: &[u8], max_edge: u32) -> Result<RgbaImage, String> {
    // Opportunistic path: guess format, read dimensions, decode, then resize at
    // native depth before into_rgba8. image 0.25's JPEG backend (zune) does not
    // expose DCT scale factors through the public API, so "subsampled decode"
    // here means avoiding full-res RGBA — still the dominant peak win.
    let reader = ImageReader::new(Cursor::new(bytes));
    let dyn_image = match reader.with_guessed_format() {
        Ok(reader) => reader.decode().or_else(|_| image::load_from_memory(bytes)),
        Err(_) => image::load_from_memory(bytes),
    }
    .map_err(|err| format!("failed to decode image: {err}"))?;

    // JPEG DCT scale is not exposed by image 0.25's public API; resize at native
    // depth before into_rgba8 still avoids a full-resolution RGBA intermediate.
    Ok(resize_dynamic_to_display_edge(dyn_image, max_edge).into_rgba8())
}

fn resize_dynamic_to_display_edge(image: DynamicImage, max_edge: u32) -> DynamicImage {
    let (width, height) = image.dimensions();
    let long_edge = width.max(height);
    let max_edge = max_edge.max(1);
    if long_edge <= max_edge {
        return image;
    }
    let scale = max_edge as f32 / long_edge as f32;
    let new_w = ((width as f32) * scale).round().max(1.0) as u32;
    let new_h = ((height as f32) * scale).round().max(1.0) as u32;
    image.resize_exact(new_w, new_h, FilterType::Triangle)
}

/// Rasterize an SVG at `PREVIEW_SVG_SUPERSAMPLE`× its (display-edge-clamped)
/// intrinsic size. Returns the raster plus the intrinsic display size the view
/// must present it at — full pixel density on 2×-scale displays, matching the
/// diagram pipeline's approach.
fn rasterize_svg_bytes(bytes: &[u8]) -> Result<(RgbaImage, u32, u32), String> {
    let options = usvg::Options {
        fontdb: DIAGRAM_FONT_DB.clone(),
        ..usvg::Options::default()
    };
    let tree = usvg::Tree::from_data(bytes, &options)
        .map_err(|err| format!("SVG could not be parsed: {err}"))?;
    let tree_size = tree.size();
    let intrinsic_width = tree_size.width().ceil().max(1.0) as u32;
    let intrinsic_height = tree_size.height().ceil().max(1.0) as u32;
    let long_edge = intrinsic_width.max(intrinsic_height);
    let display_scale = if long_edge > PREVIEW_IMAGE_MAX_EDGE {
        PREVIEW_IMAGE_MAX_EDGE as f32 / long_edge as f32
    } else {
        1.0
    };
    let display_width = ((intrinsic_width as f32) * display_scale).round().max(1.0) as u32;
    let display_height = ((intrinsic_height as f32) * display_scale).round().max(1.0) as u32;
    let raster_scale = display_scale * PREVIEW_SVG_SUPERSAMPLE as f32;
    let width = display_width.saturating_mul(PREVIEW_SVG_SUPERSAMPLE).max(1);
    let height = display_height
        .saturating_mul(PREVIEW_SVG_SUPERSAMPLE)
        .max(1);
    let mut pixmap = resvg::tiny_skia::Pixmap::new(width, height)
        .ok_or_else(|| format!("SVG raster size {width}x{height} is not valid"))?;
    resvg::render(
        &tree,
        resvg::tiny_skia::Transform::from_scale(raster_scale, raster_scale),
        &mut pixmap.as_mut(),
    );
    // tiny_skia is premultiplied RGBA; convert to straight-ish RGBA for swap path.
    let pixels = pixmap.take();
    let rgba = RgbaImage::from_raw(width, height, pixels)
        .ok_or_else(|| "SVG raster buffer does not match dimensions".to_string())?;
    Ok((rgba, display_width, display_height))
}

impl MarkionApp {
    pub(super) fn image_tab_entry(&self, key: &PreviewImageKey) -> PreviewImageEntry {
        self.preview_image_cache
            .get(key)
            .unwrap_or(PreviewImageEntry::Pending)
    }

    pub(super) fn ensure_image_tab(
        &mut self,
        tab_index: usize,
        key: PreviewImageKey,
        cx: &mut Context<Self>,
    ) {
        if tab_index >= self.tabs.len() {
            return;
        }
        let should_claim = match self.tabs[tab_index].image_mut() {
            Some(image) if image.key == key => !std::mem::replace(&mut image.claimed, true),
            _ => return,
        };
        if should_claim {
            self.preview_image_cache.claim(key.clone());
        }
        let _ = self.preview_image_cache.reserve_pending(key);
        self.schedule_pending_preview_decodes(cx);
    }

    pub(super) fn preview_image_entry(
        &self,
        url: &str,
        identity: &ImageSourceIdentity,
        document_dir: Option<&Path>,
    ) -> PreviewImageEntry {
        let key = PreviewImageKey::from_source(url, identity, document_dir);
        self.preview_image_cache
            .get(&key)
            .unwrap_or(PreviewImageEntry::Pending)
    }

    pub(super) fn ensure_preview_images(
        &mut self,
        preview: &[PreviewBlock],
        visual: &[VisualBlock],
        document_dir: Option<&Path>,
        cx: &mut Context<Self>,
    ) {
        let mut sources = Vec::new();
        collect_preview_image_sources(preview, visual, &mut sources);
        for source in &sources {
            let key = PreviewImageKey::from_source(source.url, source.identity, document_dir);
            // Only a fresh reservation needs the payload retained: the copy
            // happens once per image, not once per frame.
            if self.preview_image_cache.reserve_pending(key.clone()) && key.is_data_uri() {
                self.preview_image_cache
                    .attach_data_payload(key, Arc::from(source.url));
            }
        }
        self.schedule_pending_preview_decodes(cx);
    }

    /// Start as many pending decodes as the overall / heavy caps allow.
    pub(super) fn schedule_pending_preview_decodes(&mut self, cx: &mut Context<Self>) {
        let candidates = self.preview_image_cache.pending_not_started();
        for key in candidates {
            // The heavy-slot probe opens the image header on disk, so it must
            // not run on the UI thread (a stalled file would freeze every
            // frame that schedules decodes). Local files get a one-off
            // background probe whose result is memoized; the key stays
            // pending until the probe lands and re-kicks scheduling.
            let heavy = if key.local_path().is_some() {
                match self.preview_probe_results.get(&key) {
                    Some(&heavy) => heavy,
                    None => {
                        if self.preview_probes_in_flight.insert(key.clone()) {
                            let probe_key = key.clone();
                            cx.spawn(async move |this, cx| {
                                let heavy_key = probe_key.clone();
                                let heavy = cx
                                    .background_spawn(async move { probe_is_heavy(&heavy_key) })
                                    .await;
                                let _ = this.update(cx, |app, cx| {
                                    app.preview_probes_in_flight.remove(&probe_key);
                                    // Unbounded sessions could grow this map
                                    // one bool per image ever seen; reprobing
                                    // is cheap, so just reset when large.
                                    if app.preview_probe_results.len() >= 4096 {
                                        app.preview_probe_results.clear();
                                    }
                                    app.preview_probe_results.insert(probe_key, heavy);
                                    app.schedule_pending_preview_decodes(cx);
                                });
                            })
                            .detach();
                        }
                        continue;
                    }
                }
            } else {
                // Remote / data-URI sources never probe the filesystem and
                // classify as light without I/O.
                false
            };
            if !self.preview_image_cache.try_begin_decode(&key, heavy) {
                // Caps full; remaining pendings wait for a completion kick.
                if self.preview_image_cache.in_flight_count() >= PREVIEW_IMAGE_DECODE_CONCURRENCY {
                    break;
                }
                // Heavy-only blockage: keep scanning for light keys.
                continue;
            }
            let load_key = key.clone();
            // Data-URI keys are bounded fingerprints; the decode and the
            // failure fingerprint both need the retained full URI bytes.
            let data_payload = if key.is_data_uri() {
                self.preview_image_cache.data_payload(&key)
            } else {
                None
            };
            cx.spawn(async move |this, cx| {
                let load_payload = data_payload.clone();
                let result = cx
                    .background_spawn(async move {
                        load_preview_image(&load_key, load_payload.as_deref())
                    })
                    .await;
                let _ = this.update(cx, |app, cx| {
                    if result.is_err()
                        && let Some(fingerprint) = key.data_sha256()
                    {
                        // Record the failure by complete identity so the
                        // image source toggle can force its payload visible
                        // without rebuilding the multi-megabyte key per frame.
                        if app.failed_data_uri_fingerprints.len() >= 4096 {
                            app.failed_data_uri_fingerprints.clear();
                        }
                        app.failed_data_uri_fingerprints.insert(fingerprint);
                    }
                    for image in app.preview_image_cache.complete(&key, result) {
                        cx.drop_image(image, None);
                    }
                    app.preview_image_cache.end_decode(&key);
                    // Kick remaining pendings without requiring a user edit.
                    app.schedule_pending_preview_decodes(cx);
                    cx.notify();
                });
            })
            .detach();
        }
    }

    pub(super) fn refresh_tab_image_claims(
        &mut self,
        tab_index: usize,
        preview: &[PreviewBlock],
        visual: &[VisualBlock],
        document_dir: Option<&Path>,
        cx: &mut Context<Self>,
    ) {
        if tab_index >= self.tabs.len() {
            return;
        }
        if self.tabs[tab_index].is_image() {
            return;
        }
        let mut sources = Vec::new();
        collect_preview_image_sources(preview, visual, &mut sources);
        let new_keys: HashSet<PreviewImageKey> = sources
            .iter()
            .map(|source| PreviewImageKey::from_source(source.url, source.identity, document_dir))
            .collect();
        let old_keys = std::mem::take(&mut self.tabs[tab_index].claimed_preview_images);
        let mut dropped = Vec::new();
        for key in &old_keys {
            if !new_keys.contains(key) {
                dropped.extend(self.preview_image_cache.release(key));
            }
        }
        for key in &new_keys {
            if !old_keys.contains(key) {
                self.preview_image_cache.claim(key.clone());
            }
        }
        self.tabs[tab_index].claimed_preview_images = new_keys;
        for image in dropped {
            cx.drop_image(image, None);
        }
    }

    pub(super) fn release_tab_image_claims(&mut self, tab_index: usize, cx: &mut Context<Self>) {
        if tab_index >= self.tabs.len() {
            return;
        }
        let keys = self.tabs[tab_index].take_image_claims();
        let dropped = self.preview_image_cache.release_all(keys.iter());
        for image in dropped {
            cx.drop_image(image, None);
        }
    }
}

struct PreviewImageSource<'a> {
    url: &'a str,
    identity: &'a ImageSourceIdentity,
}

/// Collect every image source referenced by the blocks. Borrows URLs and
/// precomputed identities from derived state so per-frame collection never
/// copies or hashes a multi-megabyte data URI.
fn collect_preview_image_sources<'a>(
    preview: &'a [PreviewBlock],
    visual: &'a [VisualBlock],
    out: &mut Vec<PreviewImageSource<'a>>,
) {
    for block in preview {
        match block {
            PreviewBlock::Image { url, identity, .. } => out.push(PreviewImageSource {
                url: url.as_str(),
                identity,
            }),
            PreviewBlock::Paragraph { text, .. }
            | PreviewBlock::Heading { text, .. }
            | PreviewBlock::ListItem { text, .. }
            | PreviewBlock::FootnoteDefinition { text, .. } => {
                for span in &text.spans {
                    if let Some(image) = &span.image {
                        out.push(PreviewImageSource {
                            url: image.url.as_str(),
                            identity: &image.identity,
                        });
                    }
                }
            }
            PreviewBlock::BlockQuote { children, .. } => {
                collect_preview_image_sources(children, &[], out);
            }
            PreviewBlock::Table { rows, .. } => {
                for row in rows {
                    for cell in row {
                        for span in &cell.spans {
                            if let Some(image) = &span.image {
                                out.push(PreviewImageSource {
                                    url: image.url.as_str(),
                                    identity: &image.identity,
                                });
                            }
                        }
                    }
                }
            }
            PreviewBlock::Html { images, .. } => {
                for image in images {
                    out.push(PreviewImageSource {
                        url: image.url.as_ref(),
                        identity: &image.identity,
                    });
                }
            }
            _ => {}
        }
    }
    for block in visual {
        match &block.kind {
            VisualBlockKind::Image { url, identity, .. } => out.push(PreviewImageSource {
                url: url.as_str(),
                identity,
            }),
            VisualBlockKind::Html { images, .. } => {
                for image in images {
                    out.push(PreviewImageSource {
                        url: image.url.as_ref(),
                        identity: &image.identity,
                    });
                }
            }
            _ => {
                for run in &block.editable_runs {
                    if let Some(image) = &run.html_image {
                        out.push(PreviewImageSource {
                            url: image.url.as_str(),
                            identity: &image.identity,
                        });
                    }
                }
            }
        }
    }
}

/// Present a cached preview image, or a compact pending/error placeholder.
pub(super) fn preview_image_view(
    app: &MarkionApp,
    url: &str,
    identity: &ImageSourceIdentity,
    document_dir: Option<&Path>,
    width: Option<HtmlImgLength>,
    height: Option<HtmlImgLength>,
) -> Div {
    match app.preview_image_entry(url, identity, document_dir) {
        PreviewImageEntry::Ready(ready) => {
            // Supersampled entries (SVG) present at their intrinsic size via an
            // explicit width, exactly like `visual_diagram_editor`; plain
            // rasters keep implicit sizing (gpui lays them out at pixel size).
            let supersampled = ready.display_width != ready.width;
            let sized = resolve_html_img_display_size(
                width,
                height,
                ready.display_width as f32,
                ready.display_height as f32,
            );
            let image = img(ImageSource::Render(ready.image)).max_w_full();
            let image = if let Some((width, height)) = sized {
                image.w(px(width)).h(px(height))
            } else if supersampled {
                image.w(px(ready.display_width as f32))
            } else {
                image
            };
            div().w_full().child(image)
        }
        PreviewImageEntry::Pending => div()
            .w_full()
            .min_h(px(64.))
            .flex()
            .items_center()
            .justify_center()
            .rounded_md()
            .border_1()
            .border_color(rgb(0xcbd5e1))
            .bg(rgb(0xf1f5f9)),
        PreviewImageEntry::Error(message) => div()
            .w_full()
            .min_h(px(64.))
            .flex()
            .items_center()
            .justify_center()
            .rounded_md()
            .border_1()
            .border_color(rgb(0xcbd5e1))
            .bg(rgb(0xf8fafc))
            .text_color(rgb(0xb91c1c))
            .text_size(px(12.))
            .child(p0_tf(
                app.language,
                P0Msg::MissingImage,
                &[url, message.as_ref()],
            )),
    }
}

/// Compact inline Markdown / HTML image for mixed prose (same line as
/// adjacent text). Unlike [`preview_image_view`], this is `flex_none` and
/// does not stretch to the full column width.
pub(super) fn preview_inline_image_view(
    app: &MarkionApp,
    url: &str,
    identity: &ImageSourceIdentity,
    alt: &str,
    document_dir: Option<&Path>,
    width: Option<HtmlImgLength>,
    height: Option<HtmlImgLength>,
) -> Div {
    match app.preview_image_entry(url, identity, document_dir) {
        PreviewImageEntry::Ready(ready) => {
            let supersampled = ready.display_width != ready.width;
            let sized = resolve_html_img_display_size(
                width,
                height,
                ready.display_width as f32,
                ready.display_height as f32,
            );
            let image = img(ImageSource::Render(ready.image)).max_w_full();
            let image = if let Some((width, height)) = sized {
                image.w(px(width)).h(px(height))
            } else if supersampled {
                image.w(px(ready.display_width as f32))
            } else {
                image
            };
            div().flex_none().max_w_full().child(image)
        }
        PreviewImageEntry::Pending | PreviewImageEntry::Error(_) => {
            let label = if alt.is_empty() { url } else { alt };
            div()
                .flex_none()
                .max_w(px(240.))
                .overflow_x_hidden()
                .truncate()
                .px_2()
                .py_1()
                .rounded_md()
                .border_1()
                .border_color(rgb(0xcbd5e1))
                .bg(rgb(0xf8fafc))
                .text_size(px(11.))
                .text_color(rgb(0x64748b))
                .child(label.to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Rgba;

    fn ready(bytes: usize) -> PreviewImageReady {
        let side = ((bytes / 4) as f32).sqrt().ceil().max(1.0) as u32;
        let pixels = vec![0u8; (side as usize) * (side as usize) * 4];
        let buffer = image::ImageBuffer::from_raw(side, side, pixels).expect("buffer");
        PreviewImageReady {
            image: Arc::new(RenderImage::new(vec![image::Frame::new(buffer)])),
            byte_len: (side as usize) * (side as usize) * 4,
            width: side,
            height: side,
            display_width: side,
            display_height: side,
        }
    }

    fn ready_dims(cache: &PreviewImageCache, key: &PreviewImageKey) -> (u32, u32) {
        match cache.get(key) {
            Some(PreviewImageEntry::Ready(ready)) => (ready.width, ready.height),
            other => panic!(
                "{} should be Ready, got {}",
                key.identity,
                entry_label(&other)
            ),
        }
    }

    fn entry_label(entry: &Option<PreviewImageEntry>) -> &'static str {
        match entry {
            Some(PreviewImageEntry::Pending) => "pending",
            Some(PreviewImageEntry::Ready(_)) => "ready",
            Some(PreviewImageEntry::Error(_)) => "error",
            None => "absent",
        }
    }

    fn key(name: &str) -> PreviewImageKey {
        PreviewImageKey {
            identity: format!("local:{name}"),
        }
    }

    fn write_png(path: &Path, width: u32, height: u32) {
        let img = RgbaImage::from_pixel(width, height, Rgba([10, 20, 30, 255]));
        img.save(path).expect("write png");
    }

    #[test]
    fn identical_keys_reuse_and_pending_dedupes() {
        let mut cache = PreviewImageCache::new(8);
        let k = key("a.png");
        cache.claim(k.clone());
        assert!(cache.reserve_pending(k.clone()));
        assert!(!cache.reserve_pending(k.clone()));
        cache.complete(&k, Ok(ready(16)));
        assert!(matches!(cache.get(&k), Some(PreviewImageEntry::Ready(_))));
        assert!(!cache.reserve_pending(k));
    }

    #[test]
    fn byte_budget_overshoots_for_claimed_entries_without_degrading_them() {
        let mut cache = PreviewImageCache::with_limits(8, 64);
        let pending = key("pending.png");
        let old = key("old.png");
        let newer = key("new.png");
        cache.claim(pending.clone());
        cache.claim(old.clone());
        cache.claim(newer.clone());
        assert!(cache.reserve_pending(pending.clone()));
        assert!(cache.reserve_pending(old.clone()));
        cache.complete(&old, Ok(ready(64)));
        let (old_w, old_h) = ready_dims(&cache, &old);
        assert!(cache.reserve_pending(newer.clone()));
        // Claimed `old` must be neither evicted nor shrunk; `newer` is stored
        // at full size under the overshoot ceiling. Pending stays pending.
        cache.complete(&newer, Ok(ready(64)));
        assert!(matches!(
            cache.get(&pending),
            Some(PreviewImageEntry::Pending)
        ));
        assert_eq!(ready_dims(&cache, &old), (old_w, old_h));
        let incoming = ready(64);
        assert_eq!(
            ready_dims(&cache, &newer),
            (incoming.width, incoming.height),
            "incoming image keeps full decode size within the overshoot ceiling"
        );
        assert!(cache.completed_bytes() <= 128, "bounded by 2x budget");
    }

    #[test]
    fn single_raster_larger_than_budget_becomes_error() {
        let mut cache = PreviewImageCache::with_limits(8, 32);
        let k = key("huge.png");
        cache.claim(k.clone());
        assert!(cache.reserve_pending(k.clone()));
        cache.complete(&k, Ok(ready(64)));
        assert!(matches!(cache.get(&k), Some(PreviewImageEntry::Error(_))));
        assert_eq!(cache.completed_bytes(), 0);
    }

    #[test]
    fn unclaimed_late_completion_is_dropped() {
        let mut cache = PreviewImageCache::new(8);
        let k = key("gone.png");
        assert!(cache.reserve_pending(k.clone()));
        // No claims — completion must not retain.
        assert!(cache.complete(&k, Ok(ready(16))).len() == 1);
        assert!(cache.get(&k).is_none());
    }

    #[test]
    fn release_demotes_ready_entry_to_unclaimed_lru() {
        // Tab switches must reuse decoded images: releasing the last claim
        // keeps the entry cached (no re-decode, no placeholder flash)…
        let mut cache = PreviewImageCache::new(8);
        let k = key("solo.png");
        cache.claim(k.clone());
        assert!(cache.reserve_pending(k.clone()));
        cache.complete(&k, Ok(ready(16)));
        assert!(cache.release(&k).is_empty());
        assert_eq!(cache.claim_count(&k), 0);
        assert!(matches!(cache.get(&k), Some(PreviewImageEntry::Ready(_))));
        assert_eq!(cache.completed_bytes(), 16);
    }

    #[test]
    fn release_pays_down_overshoot_by_evicting_unclaimed() {
        // …but overshoot accepted while everything was claimed is paid down as
        // soon as claims drop.
        let mut cache = PreviewImageCache::with_limits(8, 64);
        let a = key("a.png");
        let b = key("b.png");
        for k in [&a, &b] {
            cache.claim(k.clone());
            assert!(cache.reserve_pending(k.clone()));
            cache.complete(k, Ok(ready(64)));
        }
        assert!(cache.completed_bytes() > 64, "claimed set overshoots");
        let dropped = cache.release(&a);
        assert_eq!(dropped.len(), 1, "unclaimed `a` yields to the budget");
        assert!(cache.get(&a).is_none());
        assert!(matches!(cache.get(&b), Some(PreviewImageEntry::Ready(_))));
        assert!(cache.completed_bytes() <= 64);
    }

    #[test]
    fn resize_dynamic_clamps_long_edge() {
        let dyn_image =
            DynamicImage::ImageRgba8(RgbaImage::from_pixel(4000, 1000, Rgba([1, 2, 3, 255])));
        let out = resize_dynamic_to_display_edge(dyn_image, PREVIEW_IMAGE_MAX_EDGE);
        assert_eq!(out.width(), PREVIEW_IMAGE_MAX_EDGE);
        assert_eq!(out.height(), 512);
    }

    #[test]
    fn decode_oversized_png_clamps_ready_dimensions() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("big.png");
        write_png(&path, 4000, 1000);
        let k = PreviewImageKey {
            identity: format!("local:{}", path.display()),
        };
        let ready = load_preview_image(&k, None).expect("decode");
        assert_eq!(ready.width, PREVIEW_IMAGE_MAX_EDGE);
        assert_eq!(ready.height, 512);
        assert!(ready.width.max(ready.height) <= PREVIEW_IMAGE_MAX_EDGE);
    }

    #[test]
    fn decode_small_png_keeps_dimensions() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("small.png");
        write_png(&path, 64, 48);
        let k = PreviewImageKey {
            identity: format!("local:{}", path.display()),
        };
        let ready = load_preview_image(&k, None).expect("decode");
        assert_eq!(ready.width, 64);
        assert_eq!(ready.height, 48);
    }

    #[test]
    fn unsupported_identity_errors_cleanly() {
        let k = PreviewImageKey {
            identity: "other:not-a-source".into(),
        };
        let err = match load_preview_image(&k, None) {
            Err(message) => message,
            Ok(_) => panic!("unsupported identity must fail"),
        };
        assert!(err.contains("unsupported"));
    }

    #[test]
    fn missing_local_image_reports_the_resolved_resource() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("missing.png");
        let key = PreviewImageKey {
            identity: format!("local:{}", path.display()),
        };
        let err = match load_preview_image(&key, None) {
            Err(message) => message,
            Ok(_) => panic!("missing image must not decode"),
        };
        assert!(err.contains("missing.png"), "unexpected error: {err}");
    }

    #[test]
    fn overall_concurrency_cap_limits_in_flight() {
        let mut cache = PreviewImageCache::new(64);
        let mut started = 0usize;
        for i in 0..(PREVIEW_IMAGE_DECODE_CONCURRENCY + 4) {
            let k = key(&format!("img{i}.png"));
            cache.claim(k.clone());
            assert!(cache.reserve_pending(k.clone()));
            if cache.try_begin_decode(&k, false) {
                started += 1;
            }
        }
        assert_eq!(started, PREVIEW_IMAGE_DECODE_CONCURRENCY);
        assert_eq!(
            cache.pending_not_started().len(),
            4,
            "excess stay pending not started"
        );

        // Completing one frees a slot for a waiting key.
        let finished = key("img0.png");
        cache.end_decode(&finished);
        cache.complete(&finished, Ok(ready(16)));
        let waiting = cache.pending_not_started();
        assert!(!waiting.is_empty());
        assert!(cache.try_begin_decode(&waiting[0], false));
        assert_eq!(cache.in_flight_count(), PREVIEW_IMAGE_DECODE_CONCURRENCY);
    }

    #[test]
    fn heavy_cap_does_not_block_small_images() {
        let mut cache = PreviewImageCache::new(64);
        // Saturate heavy slots.
        for i in 0..PREVIEW_IMAGE_HEAVY_DECODE_CONCURRENCY {
            let k = key(&format!("heavy{i}.png"));
            cache.claim(k.clone());
            assert!(cache.reserve_pending(k.clone()));
            assert!(cache.try_begin_decode(&k, true));
        }
        let blocked = key("heavy-extra.png");
        cache.claim(blocked.clone());
        assert!(cache.reserve_pending(blocked.clone()));
        assert!(!cache.try_begin_decode(&blocked, true));

        // Small/unclassified images still start under the overall cap.
        let small = key("small.png");
        cache.claim(small.clone());
        assert!(cache.reserve_pending(small.clone()));
        assert!(cache.try_begin_decode(&small, false));
        assert_eq!(
            cache.heavy_in_flight_count(),
            PREVIEW_IMAGE_HEAVY_DECODE_CONCURRENCY
        );
        assert_eq!(
            cache.in_flight_count(),
            PREVIEW_IMAGE_HEAVY_DECODE_CONCURRENCY + 1
        );
    }

    #[test]
    fn probe_classifies_local_oversized_png_as_heavy() {
        let dir = tempfile::tempdir().expect("tempdir");
        let big = dir.path().join("big.png");
        let small = dir.path().join("small.png");
        write_png(&big, 3000, 2000);
        write_png(&small, 128, 128);
        let big_key = PreviewImageKey {
            identity: format!("local:{}", big.display()),
        };
        let small_key = PreviewImageKey {
            identity: format!("local:{}", small.display()),
        };
        assert!(probe_is_heavy(&big_key));
        assert!(!probe_is_heavy(&small_key));
        assert!(!probe_is_heavy(&PreviewImageKey {
            identity: "remote:https://example.com/x.png".into(),
        }));
    }

    #[test]
    fn decode_raster_bytes_uses_display_edge() {
        let img = RgbaImage::from_pixel(3000, 1500, Rgba([9, 8, 7, 255]));
        let mut encoded = Vec::new();
        {
            let mut cursor = Cursor::new(&mut encoded);
            image::write_buffer_with_format(
                &mut cursor,
                img.as_raw(),
                3000,
                1500,
                image::ExtendedColorType::Rgba8,
                image::ImageFormat::Png,
            )
            .expect("encode");
        }
        let out = decode_raster_bytes(&encoded, PREVIEW_IMAGE_MAX_EDGE).expect("decode");
        assert_eq!(out.width(), PREVIEW_IMAGE_MAX_EDGE);
        assert_eq!(out.height(), 1024);
    }

    #[test]
    fn budget_pressure_never_evicts_claimed_ready_images() {
        // Reproduces the on-screen flicker loop: evicting a claimed ready entry
        // makes the next ensure() re-reserve it as Pending forever.
        let mut cache = PreviewImageCache::with_limits(8, 64);
        let a = key("a.png");
        let b = key("b.png");
        cache.claim(a.clone());
        cache.claim(b.clone());
        assert!(cache.reserve_pending(a.clone()));
        assert!(cache.reserve_pending(b.clone()));
        cache.complete(&a, Ok(ready(64)));
        assert!(matches!(cache.get(&a), Some(PreviewImageEntry::Ready(_))));

        // Completing `b` cannot fit at full size without displacing claimed `a`.
        // Overshoot must keep both Ready at full size (no sticky Error, no
        // claimed eviction, no degradation of either image).
        let (a_w, a_h) = ready_dims(&cache, &a);
        cache.complete(&b, Ok(ready(64)));
        assert_eq!(
            ready_dims(&cache, &a),
            (a_w, a_h),
            "claimed ready image must survive budget pressure untouched"
        );
        let incoming = ready(64);
        assert_eq!(
            ready_dims(&cache, &b),
            (incoming.width, incoming.height),
            "incoming image is stored full-size under the overshoot ceiling"
        );
        assert!(cache.completed_bytes() <= 128);
    }

    #[test]
    fn overshoot_ceiling_downscales_only_the_incoming_raster() {
        let mut cache = PreviewImageCache::with_limits(8, 64);
        let a = key("a.png");
        let b = key("b.png");
        let c = key("c.png");
        for k in [&a, &b, &c] {
            cache.claim(k.clone());
            assert!(cache.reserve_pending(k.clone()));
        }
        cache.complete(&a, Ok(ready(64)));
        cache.complete(&b, Ok(ready(64)));
        let (a_dims, b_dims) = (ready_dims(&cache, &a), ready_dims(&cache, &b));
        // The ceiling (128) is exhausted by claimed a+b; `c` must not displace
        // or shrink them — only `c` itself may be downscaled, else error.
        cache.complete(&c, Ok(ready(64)));
        assert_eq!(ready_dims(&cache, &a), a_dims);
        assert_eq!(ready_dims(&cache, &b), b_dims);
        match cache.get(&c) {
            Some(PreviewImageEntry::Ready(ready)) => {
                let incoming = self::ready(64);
                assert!(
                    ready.width < incoming.width,
                    "past the ceiling only the incoming raster shrinks"
                );
            }
            Some(PreviewImageEntry::Error(_)) => {}
            other => panic!("unexpected state for c: {}", entry_label(&other)),
        }
        assert!(cache.completed_bytes() <= 128);
    }

    #[test]
    fn many_claimed_images_stay_full_size_instead_of_sticky_budget_error() {
        // User-visible regression after memory bounds: a long article claims every
        // image, small/fast decodes fill the byte budget first, then large
        // headers complete into permanent "exceeds the preview image cache budget".
        // Overshoot keeps them all Ready at full decode size.
        let mut cache = PreviewImageCache::with_limits(16, 64);
        let keys: Vec<_> = (0..8).map(|i| key(&format!("img{i}.png"))).collect();
        for k in &keys {
            cache.claim(k.clone());
            assert!(cache.reserve_pending(k.clone()));
        }
        for k in &keys {
            cache.complete(k, Ok(ready(16)));
        }
        let full = ready(16);
        for k in &keys {
            assert_eq!(
                ready_dims(&cache, k),
                (full.width, full.height),
                "{} must stay Ready at full size",
                k.identity
            );
        }
        assert!(cache.completed_bytes() <= 128, "bounded by 2x budget");
    }

    #[test]
    fn svg_rasterizes_at_supersample_of_display_size() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("logo.svg");
        std::fs::write(
            &path,
            "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"120\" height=\"80\">\
             <rect width=\"120\" height=\"80\" fill=\"#3366ff\"/></svg>",
        )
        .expect("write svg");
        let k = PreviewImageKey {
            identity: format!("local:{}", path.display()),
        };
        let ready = load_preview_image(&k, None).expect("rasterize");
        assert_eq!((ready.display_width, ready.display_height), (120, 80));
        assert_eq!(
            (ready.width, ready.height),
            (120 * PREVIEW_SVG_SUPERSAMPLE, 80 * PREVIEW_SVG_SUPERSAMPLE),
            "SVG raster carries {PREVIEW_SVG_SUPERSAMPLE}x pixels per presented logical pixel"
        );
    }

    #[test]
    fn raster_display_size_matches_pixels() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("plain.png");
        write_png(&path, 96, 32);
        let k = PreviewImageKey {
            identity: format!("local:{}", path.display()),
        };
        let ready = load_preview_image(&k, None).expect("decode");
        assert_eq!((ready.display_width, ready.display_height), (96, 32));
        assert_eq!((ready.width, ready.height), (96, 32));
    }

    #[test]
    fn local_viewer_decodes_every_supported_raster_family() {
        let dir = tempfile::tempdir().expect("tempdir");
        let formats = [
            ("PNG", image::ImageFormat::Png),
            ("jpg", image::ImageFormat::Jpeg),
            ("gif", image::ImageFormat::Gif),
            ("webp", image::ImageFormat::WebP),
            ("bmp", image::ImageFormat::Bmp),
            ("tif", image::ImageFormat::Tiff),
        ];
        for (extension, format) in formats {
            let path = dir.path().join(format!("sample.{extension}"));
            let image = image::DynamicImage::ImageRgb8(image::RgbImage::from_pixel(
                7,
                5,
                image::Rgb([20, 40, 60]),
            ));
            image
                .save_with_format(&path, format)
                .unwrap_or_else(|error| panic!("encode {extension}: {error}"));
            let ready = load_preview_image(&PreviewImageKey::from_local_path(&path), None)
                .unwrap_or_else(|error| panic!("decode {extension}: {error}"));
            assert_eq!((ready.display_width, ready.display_height), (7, 5));
        }

        let svg = dir.path().join("sample.SvG");
        std::fs::write(
            &svg,
            br#"<svg xmlns="http://www.w3.org/2000/svg" width="9" height="6"><rect width="9" height="6" fill="red"/></svg>"#,
        )
        .expect("write svg");
        let ready = load_preview_image(&PreviewImageKey::from_local_path(&svg), None)
            .expect("decode local SVG");
        assert_eq!((ready.display_width, ready.display_height), (9, 6));
    }

    #[test]
    fn animated_gif_uses_a_static_decoded_frame() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("animated.gif");
        let file = std::fs::File::create(&path).expect("gif file");
        let mut encoder = image::codecs::gif::GifEncoder::new(file);
        encoder
            .encode_frames([
                image::Frame::new(RgbaImage::from_pixel(3, 2, Rgba([255, 0, 0, 255]))),
                image::Frame::new(RgbaImage::from_pixel(3, 2, Rgba([0, 0, 255, 255]))),
            ])
            .expect("encode animation");
        let ready = load_preview_image(&PreviewImageKey::from_local_path(&path), None)
            .expect("decode static presentation");
        assert_eq!((ready.display_width, ready.display_height), (3, 2));
    }

    #[test]
    fn local_viewer_contains_missing_corrupt_and_oversized_sources() {
        let dir = tempfile::tempdir().expect("tempdir");
        let missing = dir.path().join("missing.png");
        assert!(load_preview_image(&PreviewImageKey::from_local_path(&missing), None).is_err());

        let corrupt = dir.path().join("corrupt.svg");
        std::fs::write(&corrupt, b"<svg not-valid").expect("write corrupt");
        assert!(load_preview_image(&PreviewImageKey::from_local_path(&corrupt), None).is_err());

        let large = dir.path().join("large.png");
        write_png(&large, PREVIEW_IMAGE_MAX_EDGE * 2, 8);
        let ready = load_preview_image(&PreviewImageKey::from_local_path(&large), None)
            .expect("downscale oversized image");
        assert_eq!(ready.width, PREVIEW_IMAGE_MAX_EDGE);
        assert!(ready.height >= 1);
    }

    // --- data: URI (RFC 2397) support --------------------------------------

    /// Build a `;base64` data URI from raw bytes and a MIME essence.
    fn data_url_base64(mime: &str, bytes: &[u8]) -> String {
        use base64::{Engine, engine::general_purpose::STANDARD};
        format!("data:{mime};base64,{}", STANDARD.encode(bytes))
    }

    /// Build a URL-encoded (non-base64) data URI from raw bytes.
    fn data_url_urlencoded(mime: &str, bytes: &[u8]) -> String {
        let mut body = String::new();
        for &byte in bytes {
            body.push_str(&format!("%{byte:02X}"));
        }
        format!("data:{mime},{body}")
    }

    #[test]
    fn from_url_routes_data_uri_to_bounded_data_identity() {
        let url = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mNk+M8AAAMBAQDJ/pLvAAAAAElFTkSuQmCC";
        let k = PreviewImageKey::from_url(url, None);
        assert!(k.is_data_uri());
        // The identity is a bounded fingerprint — it never embeds the URI.
        assert!(
            k.identity.len() <= 96,
            "identity must stay bounded, got {} bytes",
            k.identity.len()
        );
        assert!(!url.contains(k.identity.as_str()) && !k.identity.contains("iVBOR"));
        assert_eq!(k.local_path(), None);
        assert_eq!(k.remote_url(), None);
    }

    #[test]
    fn data_uri_key_is_stable_bounded_and_content_sensitive() {
        // Multi-megabyte base64 URI: key construction must stay O(bounded).
        let body = "QUJDRA".repeat(500_000); // 3 MB of base64
        let url_a = format!("data:image/png;base64,{body}");
        let k1 = PreviewImageKey::from_url(&url_a, None);
        let k2 = PreviewImageKey::from_url(&url_a, None);
        assert_eq!(k1, k2, "same content hashes to the same key every frame");
        assert!(
            k1.identity.len() <= 96,
            "identity is independent of the {}-byte URI: {}",
            url_a.len(),
            k1.identity.len()
        );

        // Same length, one byte different inside the sampled middle region.
        let mut body_b = body.clone();
        let mid = body_b.len() / 2;
        body_b.replace_range(mid..mid + 1, "Z");
        let url_b = format!("data:image/png;base64,{body_b}");
        assert_ne!(
            PreviewImageKey::from_url(&url_b, None),
            k1,
            "different content must key differently"
        );

        // Same length, one byte different inside the tail sample region.
        let mut body_c = body.clone();
        let tail = body_c.len() - 100;
        body_c.replace_range(tail..tail + 1, "Z");
        let url_c = format!("data:image/png;base64,{body_c}");
        assert_ne!(
            PreviewImageKey::from_url(&url_c, None),
            k1,
            "a tail-sample difference must key differently"
        );

        // Different length also differs (length is part of the identity).
        let url_d = format!("data:image/png;base64,{body}QQ");
        assert_ne!(PreviewImageKey::from_url(&url_d, None), k1);
    }

    /// Two valid equal-length SVG data URIs that differ only outside the former
    /// head/middle/tail 2 KiB samples used by the sampled cache key.
    ///
    /// The SVG payload is percent-encoded so markdown destinations, HTML
    /// attributes, and `data_url` parsing do not truncate on `#`, `<`, or `>`.
    fn adversarial_equal_length_data_uri_svgs() -> (String, String) {
        const FORMER_SAMPLE: usize = 2048;
        const FORMER_FULL_HASH_MAX: usize = 64 * 1024;
        const TARGET_LEN: usize = FORMER_FULL_HASH_MAX + 8 * 1024;

        fn pct(s: &str) -> String {
            let mut out = String::with_capacity(s.len() * 3);
            for &b in s.as_bytes() {
                match b {
                    b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                        out.push(b as char);
                    }
                    _ => out.push_str(&format!("%{b:02X}")),
                }
            }
            out
        }

        let prefix = "data:image/svg+xml,";
        let head = pct("<svg xmlns='http://www.w3.org/2000/svg' width='8' height='8'><!--");
        let rect_a = pct("<rect width='8' height='8' fill='red'/>");
        let rect_b = pct("<rect width='8' height='8' fill='tan'/>");
        let after_comment = pct("-->");
        let comment_open = pct("<!--");
        let tail = pct("--></svg>");
        assert_eq!(rect_a.len(), rect_b.len());

        let build = |rect: &str| {
            let mut s = String::with_capacity(TARGET_LEN);
            s.push_str(prefix);
            s.push_str(&head);
            let rect_at = FORMER_SAMPLE + 128;
            while s.len() + after_comment.len() < rect_at {
                s.push('x');
            }
            s.push_str(&after_comment);
            s.push_str(rect);
            s.push_str(&comment_open);
            while s.len() + tail.len() < TARGET_LEN {
                s.push('x');
            }
            s.push_str(&tail);
            assert_eq!(s.len(), TARGET_LEN, "fixture length drifted to {}", s.len());
            s
        };

        let a = build(&rect_a);
        let b = build(&rect_b);
        assert_eq!(a.len(), b.len());
        assert!(a.len() > FORMER_FULL_HASH_MAX);
        assert_eq!(&a[..FORMER_SAMPLE], &b[..FORMER_SAMPLE]);
        let mid = a.len() / 2;
        let half = FORMER_SAMPLE / 2;
        assert_eq!(&a[mid - half..mid + half], &b[mid - half..mid + half]);
        assert_eq!(&a[a.len() - FORMER_SAMPLE..], &b[b.len() - FORMER_SAMPLE..]);
        assert_ne!(a, b);
        (a, b)
    }

    /// Replica of the removed sampled hasher so the adversarial pair stays
    /// locked as a collision for that algorithm even after complete SHA-256
    /// identity replaced it.
    fn former_sampled_data_uri_fingerprint(url: &str) -> u64 {
        use std::hash::{Hash, Hasher};
        const FORMER_FULL_HASH_MAX: usize = 64 * 1024;
        const FORMER_SAMPLE_LEN: usize = 2048;
        let bytes = url.as_bytes();
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        bytes.len().hash(&mut hasher);
        if bytes.len() <= FORMER_FULL_HASH_MAX {
            bytes.hash(&mut hasher);
        } else {
            let mid = bytes.len() / 2;
            let half = FORMER_SAMPLE_LEN / 2;
            bytes[..FORMER_SAMPLE_LEN].hash(&mut hasher);
            bytes[mid - half..mid + half].hash(&mut hasher);
            bytes[bytes.len() - FORMER_SAMPLE_LEN..].hash(&mut hasher);
        }
        hasher.finish()
    }

    #[test]
    fn former_sampled_keys_alias_adversarial_equal_length_svgs() {
        let (url_a, url_b) = adversarial_equal_length_data_uri_svgs();
        assert_eq!(
            former_sampled_data_uri_fingerprint(&url_a),
            former_sampled_data_uri_fingerprint(&url_b),
            "the removed sampled hasher must still collide on this fixture"
        );
        assert_ne!(
            PreviewImageKey::from_url(&url_a, None),
            PreviewImageKey::from_url(&url_b, None),
            "complete SHA-256 identity must not inherit that alias"
        );
    }

    #[test]
    fn adversarial_equal_length_svgs_keep_distinct_keys_and_rasters() {
        let (url_a, url_b) = adversarial_equal_length_data_uri_svgs();
        let k1 = PreviewImageKey::from_url(&url_a, None);
        let k2 = PreviewImageKey::from_url(&url_b, None);
        assert_ne!(k1, k2, "complete identities must not alias this pair");
        let ready_a = load_preview_image(&k1, Some(&url_a)).expect("decode svg a");
        let ready_b = load_preview_image(&k2, Some(&url_b)).expect("decode svg b");
        assert_eq!(
            (ready_a.display_width, ready_a.display_height),
            (ready_b.display_width, ready_b.display_height)
        );
        assert_ne!(
            ready_a.image.as_ref() as *const _,
            ready_b.image.as_ref() as *const _,
            "distinct sources must not share a raster allocation"
        );
    }

    #[test]
    fn unsampled_same_length_edit_invalidates_data_uri_key() {
        let (url_a, url_b) = adversarial_equal_length_data_uri_svgs();
        assert_eq!(url_a.len(), url_b.len());
        assert_ne!(
            PreviewImageKey::from_url(&url_a, None),
            PreviewImageKey::from_url(&url_b, None)
        );
    }

    #[test]
    fn repeated_source_collection_does_not_hash_or_clone_data_uri_payloads() {
        let (url_a, _) = adversarial_equal_length_data_uri_svgs();
        let markdown = format!("![red]({url_a})");
        let doc = markion::MarkdownDocument::from_text(markdown);
        let preview = doc.preview_blocks_shared();
        let visual = doc.visual_blocks_shared();
        markion::reset_data_uri_work_counters();
        let mut sources = Vec::new();
        collect_preview_image_sources(&preview, &visual, &mut sources);
        assert!(!sources.is_empty());
        for _ in 0..8 {
            let mut again = Vec::new();
            collect_preview_image_sources(&preview, &visual, &mut again);
            let keys: Vec<_> = again
                .iter()
                .map(|source| PreviewImageKey::from_source(source.url, source.identity, None))
                .collect();
            assert_eq!(keys.len(), sources.len());
        }
        assert_eq!(
            markion::DATA_URI_IDENTITY_HASH_BYTES.load(std::sync::atomic::Ordering::Relaxed),
            0,
            "repaint collection must reuse derived identities"
        );
        assert_eq!(
            markion::DATA_URI_PAYLOAD_CLONE_BYTES.load(std::sync::atomic::Ordering::Relaxed),
            0,
            "repaint collection must not copy data-URI payloads"
        );
    }

    #[test]
    fn data_uri_decode_requires_the_retained_payload() {
        // The bounded key cannot be decoded by itself; the payload side map
        // supplies the bytes. A missing payload errors instead of panicking.
        let url = data_url_base64("image/png", &[1, 2, 3]);
        let k = PreviewImageKey::from_url(&url, None);
        match load_preview_image(&k, None) {
            Err(err) => assert!(err.contains("payload"), "unexpected error: {err}"),
            Ok(_) => panic!("a data-URI key without its payload must not decode"),
        }
    }

    #[test]
    fn data_uri_claim_decode_release_cycle_via_payload_map() {
        // End-to-end over the cache: claim → reserve → attach → payload lookup
        // → decode → complete → release, with the payload freed at completion.
        let png = {
            let img = RgbaImage::from_pixel(24, 12, Rgba([1, 2, 3, 255]));
            let mut buf = std::io::Cursor::new(Vec::new());
            img.write_to(&mut buf, image::ImageFormat::Png)
                .expect("encode png");
            buf.into_inner()
        };
        let url = data_url_base64("image/png", &png);
        let key = PreviewImageKey::from_url(&url, None);

        let mut cache = PreviewImageCache::new(8);
        cache.claim(key.clone());
        assert!(cache.reserve_pending(key.clone()));
        cache.attach_data_payload(key.clone(), Arc::from(url.as_str()));
        let payload = cache.data_payload(&key).expect("retained payload");
        assert_eq!(payload.as_ref(), url);
        assert_eq!(cache.retained_data_payload_bytes(), url.len());

        let ready = load_preview_image(&key, Some(&payload)).expect("decode via payload");
        assert_eq!((ready.width, ready.height), (24, 12));
        cache.complete(&key, Ok(ready));
        assert!(matches!(cache.get(&key), Some(PreviewImageEntry::Ready(_))));
        assert_eq!(
            cache.retained_data_payload_bytes(),
            0,
            "payload is dropped once the decode lands"
        );
        assert_eq!(cache.claim_count(&key), 1);

        // Release demotes to unclaimed LRU; the decoded raster survives.
        assert!(cache.release(&key).is_empty());
        assert!(matches!(cache.get(&key), Some(PreviewImageEntry::Ready(_))));
    }

    #[test]
    fn data_uri_payload_follows_entry_removal() {
        // An unclaimed late completion drops the entry and its payload.
        let url = data_url_base64("image/png", &[9, 9]);
        let key = PreviewImageKey::from_url(&url, None);
        let mut cache = PreviewImageCache::new(8);
        assert!(cache.reserve_pending(key.clone()));
        cache.attach_data_payload(key.clone(), Arc::from(url.as_str()));
        assert!(cache.data_payload(&key).is_some());
        assert!(cache.complete(&key, Ok(ready(16))).len() == 1);
        assert!(cache.get(&key).is_none());
        assert!(cache.data_payload(&key).is_none());
        assert_eq!(cache.retained_data_payload_bytes(), 0);
    }

    #[test]
    fn remote_url_never_returns_data_scheme() {
        // A data URI must route through the `data:` identity, never `remote:`,
        // so reqwest can never be handed a `data:` scheme.
        let k = PreviewImageKey::from_url(
            "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mNk+M8AAAMBAQDJ/pLvAAAAAElFTkSuQmCC",
            None,
        );
        assert!(k.remote_url().is_none());
        // Sanity: a real http(s) URL still routes to `remote:`.
        let r = PreviewImageKey::from_url("https://example.com/a.png", None);
        assert_eq!(r.remote_url(), Some("https://example.com/a.png"));
    }

    #[test]
    fn identical_data_uris_share_key() {
        let url = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mNk+M8AAAMBAQDJ/pLvAAAAAElFTkSuQmCC";
        let k1 = PreviewImageKey::from_url(url, None);
        let k2 = PreviewImageKey::from_url(url, Some(Path::new("/any/dir")));
        assert_eq!(k1, k2, "data URIs are document-dir independent & dedupe");
    }

    #[test]
    fn load_base64_png_data_uri() {
        let png = {
            let img = RgbaImage::from_pixel(48, 24, Rgba([10, 20, 30, 255]));
            let mut buf = std::io::Cursor::new(Vec::new());
            img.write_to(&mut buf, image::ImageFormat::Png)
                .expect("encode png");
            buf.into_inner()
        };
        let url = data_url_base64("image/png", &png);
        let k = PreviewImageKey::from_url(&url, None);
        let ready = load_preview_image(&k, Some(&url)).expect("decode data-uri png");
        assert_eq!((ready.width, ready.height), (48, 24));
        assert_eq!((ready.display_width, ready.display_height), (48, 24));
        assert!(ready.byte_len > 0);
    }

    #[test]
    fn load_base64_svg_data_uri_uses_mime_for_svg_path() {
        let svg = "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"60\" height=\"40\">\
                   <rect width=\"60\" height=\"40\" fill=\"#3366ff\"/></svg>"
            .as_bytes()
            .to_vec();
        // The base64 body starts with `PHN2...` (no leading `<svg` bytes), so
        // the MIME type — not the byte scan — must select the SVG path.
        let url = data_url_base64("image/svg+xml", &svg);
        let k = PreviewImageKey::from_url(&url, None);
        let ready = load_preview_image(&k, Some(&url)).expect("rasterize data-uri svg");
        assert_eq!((ready.display_width, ready.display_height), (60, 40));
        assert_eq!(
            (ready.width, ready.height),
            (60 * PREVIEW_SVG_SUPERSAMPLE, 40 * PREVIEW_SVG_SUPERSAMPLE)
        );
    }

    #[test]
    fn load_url_encoded_data_uri_matches_base64_equivalent() {
        let png = {
            let img = RgbaImage::from_pixel(16, 16, Rgba([200, 100, 50, 255]));
            let mut buf = std::io::Cursor::new(Vec::new());
            img.write_to(&mut buf, image::ImageFormat::Png)
                .expect("encode png");
            buf.into_inner()
        };
        let b64_url = data_url_base64("image/png", &png);
        let encoded_url = data_url_urlencoded("image/png", &png);
        let b64_key = PreviewImageKey::from_url(&b64_url, None);
        let url_key = PreviewImageKey::from_url(&encoded_url, None);
        let b64_ready = load_preview_image(&b64_key, Some(&b64_url)).expect("decode base64");
        let url_ready =
            load_preview_image(&url_key, Some(&encoded_url)).expect("decode url-encoded");
        assert_eq!((b64_ready.width, b64_ready.height), (16, 16));
        assert_eq!(
            (url_ready.width, url_ready.height),
            (b64_ready.width, b64_ready.height),
            "non-base64 data URI decodes to the same pixels"
        );
    }

    #[test]
    fn malformed_data_uri_returns_err_for_placeholder() {
        // Missing comma between header and body.
        let url = "data:image/png;base64";
        let k = PreviewImageKey::from_url(url, None);
        match load_preview_image(&k, Some(url)) {
            Err(err) => assert!(err.contains("data URL"), "error mentions data URL: {err}"),
            Ok(_) => panic!("no-comma data URI must not decode"),
        }

        // Truncated/invalid base64 payload.
        let url = "data:image/png;base64,!!!!not-base64!!!!";
        let k = PreviewImageKey::from_url(url, None);
        match load_preview_image(&k, Some(url)) {
            Err(_) => {}
            Ok(_) => panic!("invalid base64 data URI must not decode"),
        }
    }

    #[test]
    fn untitled_welcome_logo_resolves_to_bundled_png() {
        let bundled = bundled_resource_path("assets/markion.png").expect("bundled logo");
        assert!(bundled.is_file());

        let key = PreviewImageKey::from_url("assets/markion.png", None);
        let resolved = key.local_path().expect("local identity");
        assert!(
            Path::new(&resolved).is_file(),
            "untitled welcome logo must resolve to a real file, got {resolved:?}"
        );
        let ready = load_preview_image(&key, None).expect("decode bundled welcome png");
        assert!(ready.width > 0 && ready.height > 0);
    }

    #[test]
    fn untitled_missing_relative_image_does_not_use_bundled_assets() {
        let key = PreviewImageKey::from_url("no-such-markion-welcome.png", None);
        let display = key
            .local_path()
            .expect("local identity")
            .display()
            .to_string()
            .replace('\\', "/");
        assert!(
            !display.contains("/assets/no-such-markion-welcome.png"),
            "missing relative URL must not be rewritten under bundled assets, got {display}"
        );
        assert!(
            bundled_resource_path("no-such-markion-welcome.png").is_none(),
            "a missing name must not invent a bundled path"
        );
    }

    #[test]
    fn saved_document_relative_image_still_joins_document_dir() {
        let document_dir = Path::new("not-a-real-welcome-doc-dir");
        let key = PreviewImageKey::from_url("assets/markion.png", Some(document_dir));
        let display = key
            .local_path()
            .expect("local identity")
            .display()
            .to_string()
            .replace('\\', "/");
        assert!(
            display.contains("not-a-real-welcome-doc-dir"),
            "named documents must keep document-dir resolution, got {display}"
        );
    }
}
