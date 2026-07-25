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
/// Floor for fair-share decode edge so tiny icons stay legible under pressure.
pub(super) const PREVIEW_IMAGE_MIN_EDGE: u32 = 64;
/// Overall in-flight fetch/decode safety cap (parallel warm for typical docs).
pub(super) const PREVIEW_IMAGE_DECODE_CONCURRENCY: usize = 8;
/// Tighter cap for probed oversized ("heavy") sources only.
pub(super) const PREVIEW_IMAGE_HEAVY_DECODE_CONCURRENCY: usize = 3;

/// Per-claim byte allowance under the completed-raster budget.
fn fair_share_bytes(max_completed_bytes: usize, claim_count: usize) -> usize {
    (max_completed_bytes / claim_count.max(1)).max(4)
}

/// Decode/display longer-edge cap so `claim_count` full-frame images fit the budget.
pub(super) fn fair_share_max_edge(max_completed_bytes: usize, claim_count: usize) -> u32 {
    let fair = fair_share_bytes(max_completed_bytes, claim_count);
    let edge = ((fair / 4) as f64).sqrt().floor() as u32;
    edge.clamp(PREVIEW_IMAGE_MIN_EDGE, PREVIEW_IMAGE_MAX_EDGE)
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(super) struct PreviewImageKey {
    pub(super) identity: String,
}

impl PreviewImageKey {
    pub(super) fn from_url(url: &str, document_dir: Option<&Path>) -> Self {
        if is_remote_resource(url) {
            Self {
                identity: format!("remote:{}", remote_image_request_url(url)),
            }
        } else {
            let path = PathBuf::from(url);
            let path = if path.is_absolute() {
                path
            } else if let Some(document_dir) = document_dir {
                document_dir.join(path)
            } else {
                path
            };
            let canonical = path.canonicalize().unwrap_or(path);
            Self {
                identity: format!("local:{}", canonical.display()),
            }
        }
    }

    fn local_path(&self) -> Option<PathBuf> {
        self.identity.strip_prefix("local:").map(PathBuf::from)
    }

    fn remote_url(&self) -> Option<&str> {
        self.identity.strip_prefix("remote:")
    }
}

#[derive(Clone)]
pub(super) struct PreviewImageReady {
    pub(super) image: Arc<RenderImage>,
    pub(super) byte_len: usize,
    pub(super) width: u32,
    pub(super) height: u32,
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
        }
    }

    pub(super) fn get(&self, key: &PreviewImageKey) -> Option<PreviewImageEntry> {
        self.entries.get(key).cloned()
    }

    pub(super) fn claim(&mut self, key: PreviewImageKey) {
        *self.claims.entry(key).or_insert(0) += 1;
    }

    pub(super) fn release(&mut self, key: &PreviewImageKey) -> Option<Arc<RenderImage>> {
        let Some(count) = self.claims.get_mut(key) else {
            return None;
        };
        *count = count.saturating_sub(1);
        if *count > 0 {
            return None;
        }
        self.claims.remove(key);
        self.remove_entry(key)
    }

    pub(super) fn release_all<'a>(
        &mut self,
        keys: impl IntoIterator<Item = &'a PreviewImageKey>,
    ) -> Vec<Arc<RenderImage>> {
        let mut dropped = Vec::new();
        for key in keys {
            if let Some(image) = self.release(key) {
                dropped.push(image);
            }
        }
        dropped
    }

    pub(super) fn claim_count(&self, key: &PreviewImageKey) -> usize {
        self.claims.get(key).copied().unwrap_or(0)
    }

    pub(super) fn claimed_key_count(&self) -> usize {
        self.claims.len()
    }

    pub(super) fn max_completed_bytes(&self) -> usize {
        self.max_completed_bytes
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

    pub(super) fn complete(
        &mut self,
        key: &PreviewImageKey,
        result: Result<PreviewImageReady, String>,
    ) -> Vec<Arc<RenderImage>> {
        let mut dropped = Vec::new();
        if !matches!(self.entries.get(key), Some(PreviewImageEntry::Pending)) {
            return dropped;
        }
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
            Ok(ready) => match self.fit_ready_under_budget(key, ready, &mut dropped) {
                Ok(ready) => Ok(ready),
                Err(ready) => {
                    // Truly impossible to retain (no remaining bytes even at 1px).
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

    /// Make `ready` fit the completed-byte budget without evicting claimed images.
    ///
    /// Order: free unclaimed LRU → fair-share shrink of existing claimed ready
    /// entries → downscale the incoming raster → last-resort reject.
    fn fit_ready_under_budget(
        &mut self,
        key: &PreviewImageKey,
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

        dropped.extend(self.shrink_claimed_ready_to_fair_share(key));
        let fair = fair_share_bytes(self.max_completed_bytes, self.claims.len());
        if ready.byte_len > fair {
            let previous = ready.image.clone();
            ready = downscale_ready_to_max_bytes(ready, fair);
            dropped.push(previous);
        }
        if self.completed_bytes.saturating_add(ready.byte_len) <= self.max_completed_bytes {
            return Ok(ready);
        }

        let remaining = self
            .max_completed_bytes
            .saturating_sub(self.completed_bytes)
            .max(4);
        if ready.byte_len > remaining {
            let previous = ready.image.clone();
            ready = downscale_ready_to_max_bytes(ready, remaining);
            dropped.push(previous);
        }
        if self.completed_bytes.saturating_add(ready.byte_len) <= self.max_completed_bytes {
            Ok(ready)
        } else {
            Err(ready)
        }
    }

    /// Downscale oversized claimed ready entries toward the current fair share.
    fn shrink_claimed_ready_to_fair_share(
        &mut self,
        except: &PreviewImageKey,
    ) -> Vec<Arc<RenderImage>> {
        let fair = fair_share_bytes(self.max_completed_bytes, self.claims.len());
        let keys: Vec<PreviewImageKey> = self.completed_order.iter().cloned().collect();
        let mut dropped = Vec::new();
        for key in keys {
            if &key == except {
                continue;
            }
            let Some(PreviewImageEntry::Ready(ready)) = self.entries.get(&key).cloned() else {
                continue;
            };
            if ready.byte_len <= fair {
                continue;
            }
            let previous_bytes = ready.byte_len;
            let previous_image = ready.image.clone();
            let shrunk = downscale_ready_to_max_bytes(ready, fair);
            self.completed_bytes = self
                .completed_bytes
                .saturating_sub(previous_bytes)
                .saturating_add(shrunk.byte_len);
            self.entries
                .insert(key, PreviewImageEntry::Ready(shrunk));
            dropped.push(previous_image);
        }
        dropped
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

pub(super) fn load_preview_image(key: &PreviewImageKey) -> Result<PreviewImageReady, String> {
    load_preview_image_with_max_edge(key, PREVIEW_IMAGE_MAX_EDGE)
}

pub(super) fn load_preview_image_with_max_edge(
    key: &PreviewImageKey,
    max_edge: u32,
) -> Result<PreviewImageReady, String> {
    let bytes = if let Some(path) = key.local_path() {
        std::fs::read(&path).map_err(|err| format!("failed to read {}: {err}", path.display()))?
    } else if let Some(url) = key.remote_url() {
        network::fetch_url_bytes(url).map_err(|err| err.to_string())?
    } else {
        return Err("unsupported image identity".into());
    };

    let is_svg = key
        .local_path()
        .and_then(|path| {
            path.extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext.eq_ignore_ascii_case("svg"))
        })
        .unwrap_or(false)
        || bytes
            .windows(4)
            .take(64)
            .any(|window| window == b"<svg" || window == b"<SVG");

    let rgba = if is_svg {
        rasterize_svg_bytes(&bytes, max_edge)?
    } else {
        decode_raster_bytes(&bytes, max_edge)?
    };

    rgba_to_ready(rgba)
}

fn rgba_to_ready(rgba: RgbaImage) -> Result<PreviewImageReady, String> {
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
    while (new_w as usize).saturating_mul(new_h as usize).saturating_mul(4) > max_bytes {
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
    rgba_to_ready(resized).unwrap_or(ready)
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

fn rasterize_svg_bytes(bytes: &[u8], max_edge: u32) -> Result<RgbaImage, String> {
    let options = usvg::Options {
        fontdb: DIAGRAM_FONT_DB.clone(),
        ..usvg::Options::default()
    };
    let tree = usvg::Tree::from_data(bytes, &options)
        .map_err(|err| format!("SVG could not be parsed: {err}"))?;
    let tree_size = tree.size();
    let mut width = tree_size.width().ceil().max(1.0) as u32;
    let mut height = tree_size.height().ceil().max(1.0) as u32;
    let long_edge = width.max(height);
    let max_edge = max_edge.max(1);
    let scale = if long_edge > max_edge {
        max_edge as f32 / long_edge as f32
    } else {
        1.0
    };
    width = ((width as f32) * scale).round().max(1.0) as u32;
    height = ((height as f32) * scale).round().max(1.0) as u32;
    let mut pixmap = resvg::tiny_skia::Pixmap::new(width, height)
        .ok_or_else(|| format!("SVG raster size {width}x{height} is not valid"))?;
    resvg::render(
        &tree,
        resvg::tiny_skia::Transform::from_scale(scale, scale),
        &mut pixmap.as_mut(),
    );
    // tiny_skia is premultiplied RGBA; convert to straight-ish RGBA for swap path.
    let pixels = pixmap.take();
    RgbaImage::from_raw(width, height, pixels)
        .ok_or_else(|| "SVG raster buffer does not match dimensions".into())
}

impl MarkionApp {
    pub(super) fn preview_image_entry(
        &self,
        url: &str,
        document_dir: Option<&Path>,
    ) -> PreviewImageEntry {
        let key = PreviewImageKey::from_url(url, document_dir);
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
        let mut urls = Vec::new();
        collect_preview_image_urls(preview, visual, &mut urls);
        for url in &urls {
            let key = PreviewImageKey::from_url(url, document_dir);
            let _ = self.preview_image_cache.reserve_pending(key);
        }
        self.schedule_pending_preview_decodes(cx);
    }

    /// Start as many pending decodes as the overall / heavy caps allow.
    pub(super) fn schedule_pending_preview_decodes(&mut self, cx: &mut Context<Self>) {
        let candidates = self.preview_image_cache.pending_not_started();
        let max_edge = fair_share_max_edge(
            self.preview_image_cache.max_completed_bytes(),
            self.preview_image_cache.claimed_key_count(),
        );
        for key in candidates {
            let heavy = probe_is_heavy(&key);
            if !self.preview_image_cache.try_begin_decode(&key, heavy) {
                // Caps full; remaining pendings wait for a completion kick.
                if self.preview_image_cache.in_flight_count() >= PREVIEW_IMAGE_DECODE_CONCURRENCY {
                    break;
                }
                // Heavy-only blockage: keep scanning for light keys.
                continue;
            }
            let load_key = key.clone();
            cx.spawn(async move |this, cx| {
                let result = cx
                    .background_spawn(async move {
                        load_preview_image_with_max_edge(&load_key, max_edge)
                    })
                    .await;
                let _ = this.update(cx, |app, cx| {
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
        let mut urls = Vec::new();
        collect_preview_image_urls(preview, visual, &mut urls);
        let new_keys: HashSet<PreviewImageKey> = urls
            .iter()
            .map(|url| PreviewImageKey::from_url(url, document_dir))
            .collect();
        let old_keys = std::mem::take(&mut self.tabs[tab_index].claimed_preview_images);
        let mut dropped = Vec::new();
        for key in &old_keys {
            if !new_keys.contains(key)
                && let Some(image) = self.preview_image_cache.release(key)
            {
                dropped.push(image);
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
        let keys = std::mem::take(&mut self.tabs[tab_index].claimed_preview_images);
        let dropped = self.preview_image_cache.release_all(keys.iter());
        for image in dropped {
            cx.drop_image(image, None);
        }
    }
}

fn collect_preview_image_urls(
    preview: &[PreviewBlock],
    visual: &[VisualBlock],
    out: &mut Vec<String>,
) {
    for block in preview {
        match block {
            PreviewBlock::Image { url, .. } => out.push(url.clone()),
            PreviewBlock::Html { html, .. } => {
                for part in html_preview_parts(html) {
                    if let HtmlPreviewPart::Image { url, .. } = part {
                        out.push(url);
                    }
                }
            }
            _ => {}
        }
    }
    for block in visual {
        if let VisualBlockKind::Image { url, .. } = &block.kind {
            out.push(url.clone());
        }
    }
}

/// Present a cached preview image, or a compact pending/error placeholder.
pub(super) fn preview_image_view(app: &MarkionApp, url: &str, document_dir: Option<&Path>) -> Div {
    match app.preview_image_entry(url, document_dir) {
        PreviewImageEntry::Ready(ready) => div()
            .w_full()
            .child(img(ImageSource::Render(ready.image)).max_w_full()),
        PreviewImageEntry::Pending => div()
            .w_full()
            .min_h(px(64.))
            .flex()
            .items_center()
            .justify_center()
            .bg(rgb(0xf1f5f9)),
        PreviewImageEntry::Error(message) => div()
            .w_full()
            .min_h(px(64.))
            .flex()
            .items_center()
            .justify_center()
            .text_color(rgb(0xb91c1c))
            .text_size(px(12.))
            .child(message.to_string()),
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
    fn byte_budget_keeps_pending_and_shrinks_when_only_claimed_ready_exist() {
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
        assert!(cache.reserve_pending(newer.clone()));
        // Claimed `old` must not be evicted; both ready images shrink to fair-share.
        // Pending stays pending (never evicted for budget).
        cache.complete(&newer, Ok(ready(64)));
        assert!(matches!(
            cache.get(&pending),
            Some(PreviewImageEntry::Pending)
        ));
        assert!(matches!(cache.get(&old), Some(PreviewImageEntry::Ready(_))));
        assert!(matches!(
            cache.get(&newer),
            Some(PreviewImageEntry::Ready(_))
        ));
        assert!(cache.completed_bytes() <= 64);
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
    fn release_evicts_zero_claim_ready_entry() {
        let mut cache = PreviewImageCache::new(8);
        let k = key("solo.png");
        cache.claim(k.clone());
        assert!(cache.reserve_pending(k.clone()));
        cache.complete(&k, Ok(ready(16)));
        let dropped = cache.release(&k);
        assert!(dropped.is_some());
        assert!(cache.get(&k).is_none());
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
        let ready = load_preview_image(&k).expect("decode");
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
        let ready = load_preview_image(&k).expect("decode");
        assert_eq!(ready.width, 64);
        assert_eq!(ready.height, 48);
    }

    #[test]
    fn unsupported_identity_errors_cleanly() {
        let k = PreviewImageKey {
            identity: "other:not-a-source".into(),
        };
        let err = match load_preview_image(&k) {
            Err(message) => message,
            Ok(_) => panic!("unsupported identity must fail"),
        };
        assert!(err.contains("unsupported"));
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
        // Fair-share shrink must keep both Ready (no sticky Error, no claimed eviction).
        cache.complete(&b, Ok(ready(64)));
        assert!(
            matches!(cache.get(&a), Some(PreviewImageEntry::Ready(_))),
            "claimed ready image must survive budget pressure"
        );
        assert!(
            matches!(cache.get(&b), Some(PreviewImageEntry::Ready(_))),
            "incoming image must shrink to fair-share instead of sticky Error"
        );
        assert!(cache.completed_bytes() <= 64);
    }

    #[test]
    fn many_claimed_images_shrink_instead_of_sticky_budget_error() {
        // User-visible regression after memory bounds: a long article claims every
        // image, small/fast decodes fill the byte budget first, then large
        // headers complete into permanent "exceeds the preview image cache budget".
        let mut cache = PreviewImageCache::with_limits(16, 64);
        let keys: Vec<_> = (0..8).map(|i| key(&format!("img{i}.png"))).collect();
        for k in &keys {
            cache.claim(k.clone());
            assert!(cache.reserve_pending(k.clone()));
        }
        for k in &keys {
            cache.complete(k, Ok(ready(16)));
        }
        for k in &keys {
            assert!(
                matches!(cache.get(k), Some(PreviewImageEntry::Ready(_))),
                "{} should be Ready, got {:?}",
                k.identity,
                cache.get(k).map(|e| match e {
                    PreviewImageEntry::Pending => "pending",
                    PreviewImageEntry::Ready(_) => "ready",
                    PreviewImageEntry::Error(m) => {
                        let _ = m;
                        "error"
                    }
                })
            );
        }
        assert!(cache.completed_bytes() <= 64);
        assert_eq!(
            keys
                .iter()
                .filter(|k| matches!(cache.get(k), Some(PreviewImageEntry::Error(_))))
                .count(),
            0,
            "budget pressure must not sticky-error claimed preview images"
        );
    }
}
