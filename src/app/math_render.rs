use super::*;

use gpui::{RenderImage, Size};
use typune_markdown::{
    MathError, MathErrorKind, MathRenderOptions, MathRenderer, RENDERER_VERSION,
    RenderedMath as SvgMath,
};

pub(super) const MATH_CACHE_CAPACITY: usize = 256;
pub(super) const MATH_CACHE_MAX_BYTES: usize = 128 * 1024 * 1024;
pub(super) const MATH_INLINE_FONT_SIZE: f32 = 16.0;
pub(super) const MATH_DISPLAY_FONT_SIZE: f32 = 20.0;
const MATH_SUPERSAMPLE: f32 = 2.0;
const MAX_RASTER_EDGE: u32 = 8_192;
const MAX_RASTER_PIXELS: u64 = 32 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(super) struct MathCacheKey {
    pub(super) latex: String,
    pub(super) style: MathLayoutStyle,
    pub(super) font_size_milli: u32,
    pub(super) foreground: [u8; 4],
    pub(super) zoom_milli: u32,
    pub(super) display_scale_milli: u32,
    pub(super) renderer_version: &'static str,
}

impl MathCacheKey {
    fn new(
        latex: &str,
        style: MathLayoutStyle,
        font_size: f32,
        foreground: Rgba,
        zoom: f32,
        display_scale: f32,
    ) -> Self {
        Self {
            latex: latex.to_string(),
            style,
            font_size_milli: quantize_positive(font_size, 6.0, 256.0),
            foreground: rgba_bytes(foreground),
            zoom_milli: quantize_positive(zoom, 0.25, 8.0),
            display_scale_milli: quantize_positive(display_scale, 0.5, 8.0),
            renderer_version: RENDERER_VERSION,
        }
    }

    fn font_size(&self) -> f32 {
        self.font_size_milli as f32 / 1000.0
    }

    fn raster_scale(&self) -> f32 {
        self.zoom_milli as f32 / 1000.0 * self.display_scale_milli as f32 / 1000.0
    }

    fn render_options(&self) -> MathRenderOptions {
        match self.style {
            MathLayoutStyle::Text => MathRenderOptions::inline(self.font_size(), self.foreground),
            MathLayoutStyle::Display => {
                MathRenderOptions::display(self.font_size(), self.foreground)
            }
        }
    }
}

fn quantize_positive(value: f32, min: f32, max: f32) -> u32 {
    let value = if value.is_finite() { value } else { min };
    (value.clamp(min, max) * 1000.0).round() as u32
}

fn rgba_bytes(color: Rgba) -> [u8; 4] {
    let channel = |value: f32| (value.clamp(0.0, 1.0) * 255.0).round() as u8;
    [
        channel(color.r),
        channel(color.g),
        channel(color.b),
        channel(color.a),
    ]
}

#[derive(Clone)]
pub(super) struct MathImage {
    pub(super) image: Arc<RenderImage>,
    pub(super) size: Size<Pixels>,
    pub(super) ascent: Pixels,
    pub(super) descent: Pixels,
    pub(super) byte_len: usize,
}

#[derive(Clone)]
pub(super) enum MathCacheEntry {
    Pending,
    Ready(Arc<MathImage>),
    Error(Arc<MathError>),
}

pub(super) struct MathCache {
    capacity: usize,
    max_completed_bytes: usize,
    completed_bytes: usize,
    entries: HashMap<MathCacheKey, MathCacheEntry>,
    completed_order: VecDeque<MathCacheKey>,
}

impl MathCache {
    pub(super) fn new(capacity: usize) -> Self {
        Self::with_limits(capacity, MATH_CACHE_MAX_BYTES)
    }

    fn with_limits(capacity: usize, max_completed_bytes: usize) -> Self {
        Self {
            capacity,
            max_completed_bytes,
            completed_bytes: 0,
            entries: HashMap::new(),
            completed_order: VecDeque::new(),
        }
    }

    pub(super) fn reserve_pending(&mut self, key: MathCacheKey) -> bool {
        if self.capacity == 0 || self.entries.contains_key(&key) {
            return false;
        }
        while self.entries.len() >= self.capacity {
            if !self.evict_oldest_completed() {
                return false;
            }
        }
        self.entries.insert(key, MathCacheEntry::Pending);
        true
    }

    pub(super) fn complete(&mut self, key: &MathCacheKey, result: Result<MathImage, MathError>) {
        if !matches!(self.entries.get(key), Some(MathCacheEntry::Pending)) {
            return;
        }
        let result = match result {
            Ok(image) if image.byte_len > self.max_completed_bytes => Err(MathError::new(
                MathErrorKind::OutputTooLarge,
                "formula raster exceeds the total math cache memory bound",
            )),
            result => result,
        };
        if let Ok(image) = &result {
            while self.completed_bytes.saturating_add(image.byte_len) > self.max_completed_bytes {
                if !self.evict_oldest_completed() {
                    break;
                }
            }
        }
        let entry = match result {
            Ok(image) => {
                self.completed_bytes = self.completed_bytes.saturating_add(image.byte_len);
                MathCacheEntry::Ready(Arc::new(image))
            }
            Err(error) => MathCacheEntry::Error(Arc::new(error)),
        };
        self.entries.insert(key.clone(), entry);
        self.completed_order.push_back(key.clone());
    }

    pub(super) fn get(&self, key: &MathCacheKey) -> Option<MathCacheEntry> {
        self.entries.get(key).cloned()
    }

    fn evict_oldest_completed(&mut self) -> bool {
        while let Some(oldest) = self.completed_order.pop_front() {
            let Some(entry) = self.entries.get(&oldest) else {
                continue;
            };
            if matches!(entry, MathCacheEntry::Pending) {
                continue;
            }
            if let Some(MathCacheEntry::Ready(image)) = self.entries.remove(&oldest) {
                self.completed_bytes = self.completed_bytes.saturating_sub(image.byte_len);
            } else {
                self.entries.remove(&oldest);
            }
            return true;
        }
        false
    }

    #[cfg(test)]
    fn len(&self) -> usize {
        self.entries.len()
    }
}

fn rasterize_math(rendered: SvgMath, effective_scale: f32) -> Result<MathImage, MathError> {
    let scale = effective_scale.clamp(0.5, 8.0) * MATH_SUPERSAMPLE;
    let width = (rendered.dimensions.width * scale).ceil() as u32;
    let height = (rendered.dimensions.height * scale).ceil() as u32;
    if width == 0
        || height == 0
        || width > MAX_RASTER_EDGE
        || height > MAX_RASTER_EDGE
        || u64::from(width) * u64::from(height) > MAX_RASTER_PIXELS
    {
        return Err(MathError::new(
            MathErrorKind::OutputTooLarge,
            format!("formula raster size {width}x{height} exceeds the configured bound"),
        ));
    }

    let tree = usvg::Tree::from_data(rendered.svg.as_bytes(), &usvg::Options::default()).map_err(
        |error| {
            MathError::new(
                MathErrorKind::UnsafeSvg,
                format!("formula SVG could not be parsed: {error}"),
            )
        },
    )?;
    let tree_size = tree.size();
    let mut pixmap = resvg::tiny_skia::Pixmap::new(width, height).ok_or_else(|| {
        MathError::new(
            MathErrorKind::OutputTooLarge,
            format!("formula raster size {width}x{height} is not valid"),
        )
    })?;
    let transform = resvg::tiny_skia::Transform::from_scale(
        width as f32 / tree_size.width(),
        height as f32 / tree_size.height(),
    );
    resvg::render(&tree, transform, &mut pixmap.as_mut());

    let mut pixels = pixmap.take();
    for pixel in pixels.chunks_exact_mut(4) {
        pixel.swap(0, 2);
    }
    let buffer = image::ImageBuffer::from_raw(width, height, pixels).ok_or_else(|| {
        MathError::new(
            MathErrorKind::Layout,
            "formula raster buffer does not match its dimensions",
        )
    })?;

    Ok(MathImage {
        image: Arc::new(RenderImage::new(vec![image::Frame::new(buffer)])),
        size: size(
            px(rendered.dimensions.width),
            px(rendered.dimensions.height),
        ),
        ascent: px(rendered.ascent),
        descent: px(rendered.descent),
        byte_len: width as usize * height as usize * 4,
    })
}

impl MarkionApp {
    fn math_key(
        &self,
        latex: &str,
        style: MathLayoutStyle,
        font_size: f32,
        zoom: f32,
        display_scale: f32,
        foreground: Rgba,
    ) -> MathCacheKey {
        MathCacheKey::new(latex, style, font_size, foreground, zoom, display_scale)
    }

    pub(super) fn math_entry(
        &self,
        latex: &str,
        style: MathLayoutStyle,
        font_size: f32,
        zoom: f32,
        display_scale: f32,
        foreground: Rgba,
    ) -> MathCacheEntry {
        let key = self.math_key(latex, style, font_size, zoom, display_scale, foreground);
        self.math_cache.get(&key).unwrap_or(MathCacheEntry::Pending)
    }

    pub(super) fn ensure_math_renders(
        &mut self,
        preview: &[PreviewBlock],
        visual: &[VisualBlock],
        zoom: f32,
        display_scale: f32,
        foreground: Rgba,
        cx: &mut Context<Self>,
    ) {
        let typography = self.typography_metrics();
        let mut requested = HashSet::new();
        for block in preview {
            match block {
                PreviewBlock::Heading { text, .. }
                | PreviewBlock::Paragraph { text, .. }
                | PreviewBlock::ListItem { text, .. }
                | PreviewBlock::BlockQuote { text, .. }
                | PreviewBlock::FootnoteDefinition { text, .. } => {
                    for span in &text.spans {
                        if let Some(math) = &span.math {
                            requested.insert(self.math_key(
                                &math.latex,
                                math.style,
                                typography.math_font_size(math.style),
                                zoom,
                                display_scale,
                                foreground,
                            ));
                        }
                    }
                }
                PreviewBlock::MathBlock { latex, .. } => {
                    requested.insert(self.math_key(
                        latex,
                        MathLayoutStyle::Display,
                        typography.display_math_font_size,
                        zoom,
                        display_scale,
                        foreground,
                    ));
                }
                _ => {}
            }
        }
        for block in visual {
            if let VisualBlockKind::MathBlock { latex, .. } = &block.kind {
                requested.insert(self.math_key(
                    latex,
                    MathLayoutStyle::Display,
                    typography.display_math_font_size,
                    zoom,
                    display_scale,
                    foreground,
                ));
            }
            for run in &block.editable_runs {
                if let Some(math) = &run.math {
                    requested.insert(self.math_key(
                        &math.latex,
                        math.style,
                        typography.math_font_size(math.style),
                        zoom,
                        display_scale,
                        foreground,
                    ));
                }
            }
        }

        let mut missing = Vec::new();
        for key in requested {
            if self.math_cache.reserve_pending(key.clone()) {
                missing.push(key);
            }
        }

        for key in missing {
            let render_key = key.clone();
            cx.spawn(async move |this, cx| {
                let result = cx
                    .background_spawn(async move {
                        MathRenderer::new()
                            .render(&render_key.latex, render_key.render_options())
                            .and_then(|rendered| {
                                rasterize_math(rendered, render_key.raster_scale())
                            })
                    })
                    .await;
                let _ = this.update(cx, |app, cx| {
                    app.math_cache.complete(&key, result);
                    cx.notify();
                });
            })
            .detach();
        }
    }

    pub(super) fn math_error_message(&self, error: &MathError) -> &'static str {
        let message = match error.kind {
            MathErrorKind::Empty | MathErrorKind::Delimiter | MathErrorKind::Parse => {
                Msg::MathInvalid
            }
            MathErrorKind::InputTooLarge | MathErrorKind::OutputTooLarge => Msg::MathTooLarge,
            MathErrorKind::UnsafeSvg => Msg::MathUnsupported,
            MathErrorKind::InvalidOptions | MathErrorKind::Layout => Msg::MathRenderFailed,
        };
        t(self.language, message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(latex: &str, style: MathLayoutStyle, color: Rgba, scale: f32) -> MathCacheKey {
        MathCacheKey::new(latex, style, 16.0, color, 1.0, scale)
    }

    fn image(seed: u8) -> MathImage {
        let buffer = image::ImageBuffer::from_raw(1, 1, vec![seed, seed, seed, 255]).unwrap();
        MathImage {
            image: Arc::new(RenderImage::new(vec![image::Frame::new(buffer)])),
            size: size(px(1.0), px(1.0)),
            ascent: px(0.8),
            descent: px(0.2),
            byte_len: 4,
        }
    }

    #[test]
    fn rasterized_math_is_bgra_scaled_and_logically_sized() {
        let rendered = MathRenderer::new()
            .render("x", MathRenderOptions::inline(16.0, [255, 0, 0, 255]))
            .unwrap();
        let expected = rendered.dimensions;
        let rasterized = rasterize_math(rendered, 2.0).unwrap();
        assert_eq!(rasterized.size.width, px(expected.width));
        assert_eq!(rasterized.size.height, px(expected.height));
        assert!(rasterized.image.size(0).width.0 > expected.width as i32);
        assert!(
            rasterized
                .image
                .as_bytes(0)
                .unwrap()
                .chunks_exact(4)
                .any(|pixel| pixel[2] > 0 && pixel[0] == 0 && pixel[1] == 0 && pixel[3] > 0)
        );
    }

    #[test]
    fn cache_coalesces_reuses_and_separates_all_presentation_inputs() {
        let base = key("x", MathLayoutStyle::Text, rgb(0x112233), 1.0);
        let display = key("x", MathLayoutStyle::Display, rgb(0x112233), 1.0);
        let color = key("x", MathLayoutStyle::Text, rgb(0x445566), 1.0);
        let scale = key("x", MathLayoutStyle::Text, rgb(0x112233), 2.0);
        let font_size =
            MathCacheKey::new("x", MathLayoutStyle::Text, 20.0, rgb(0x112233), 1.0, 1.0);
        let zoom = MathCacheKey::new("x", MathLayoutStyle::Text, 16.0, rgb(0x112233), 1.25, 1.0);
        let mut cache = MathCache::new(8);
        assert!(cache.reserve_pending(base.clone()));
        assert!(!cache.reserve_pending(base.clone()));
        cache.complete(&base, Ok(image(1)));
        assert!(!cache.reserve_pending(base.clone()));
        assert!(cache.reserve_pending(display));
        assert!(cache.reserve_pending(color));
        assert!(cache.reserve_pending(scale));
        assert!(cache.reserve_pending(font_size));
        assert!(cache.reserve_pending(zoom));
        assert_eq!(base.renderer_version, RENDERER_VERSION);
    }

    #[test]
    fn deterministic_eviction_never_removes_pending_work() {
        let pending = key("pending", MathLayoutStyle::Text, rgb(0), 1.0);
        let done = key("done", MathLayoutStyle::Text, rgb(0), 1.0);
        let replacement = key("replacement", MathLayoutStyle::Text, rgb(0), 1.0);
        let mut cache = MathCache::new(2);
        assert!(cache.reserve_pending(pending.clone()));
        assert!(cache.reserve_pending(done.clone()));
        cache.complete(&done, Ok(image(1)));
        assert!(cache.reserve_pending(replacement));
        assert_eq!(cache.len(), 2);
        assert!(matches!(cache.get(&pending), Some(MathCacheEntry::Pending)));
        assert!(cache.get(&done).is_none());
    }

    #[test]
    fn completed_rasters_obey_the_total_memory_budget() {
        let first = key("first", MathLayoutStyle::Text, rgb(0), 1.0);
        let second = key("second", MathLayoutStyle::Text, rgb(0), 1.0);
        let mut cache = MathCache::with_limits(8, 6);
        assert!(cache.reserve_pending(first.clone()));
        assert!(cache.reserve_pending(second.clone()));
        cache.complete(&first, Ok(image(1)));
        cache.complete(&second, Ok(image(2)));
        assert!(cache.get(&first).is_none());
        assert!(matches!(cache.get(&second), Some(MathCacheEntry::Ready(_))));
        assert!(cache.completed_bytes <= 6);
    }

    #[test]
    fn stale_completion_is_key_scoped_and_document_neutral() {
        let old = key("x", MathLayoutStyle::Text, rgb(0), 1.0);
        let current = key("y", MathLayoutStyle::Text, rgb(0), 1.0);
        let mut cache = MathCache::new(4);
        assert!(cache.reserve_pending(old.clone()));
        assert!(cache.reserve_pending(current.clone()));

        let mut tab = EditorTab::new(MarkdownDocument::from_text("before $y$ after"));
        tab.push_undo_snapshot();
        let version = tab.document.version();
        let text = tab.document.text().to_string();
        let preview = tab.document.preview_blocks_shared();
        let visual = tab.document.visual_blocks_shared();
        tab.sync_preview_list(&preview);
        let preview_item_count = tab.preview_list.item_count();
        let preview_scroll_top = tab.preview_list.logical_scroll_top();
        let undo_len = tab.undo_stack.len();

        cache.complete(&old, Ok(image(2)));

        assert!(matches!(cache.get(&current), Some(MathCacheEntry::Pending)));
        assert_eq!(tab.document.version(), version);
        assert_eq!(tab.document.text(), text);
        assert_eq!(tab.undo_stack.len(), undo_len);
        assert_eq!(tab.preview_list.item_count(), preview_item_count);
        let scroll_top_after = tab.preview_list.logical_scroll_top();
        assert_eq!(scroll_top_after.item_ix, preview_scroll_top.item_ix);
        assert_eq!(
            scroll_top_after.offset_in_item,
            preview_scroll_top.offset_in_item
        );
        assert!(Arc::ptr_eq(&preview, &tab.document.preview_blocks_shared()));
        assert!(Arc::ptr_eq(&visual, &tab.document.visual_blocks_shared()));
    }

    #[test]
    fn renderer_and_raster_bounds_reject_pathological_formula() {
        let error = MathRenderer::new()
            .render(
                &"x+".repeat(2_000),
                MathRenderOptions::display(64.0, [0, 0, 0, 255]),
            )
            .unwrap_err();
        assert_eq!(error.kind, MathErrorKind::OutputTooLarge);
    }
}
