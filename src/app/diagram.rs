use std::sync::LazyLock;

use super::*;
use gpui::{RenderImage, Size};
use markion_diagram::{DiagramError, DiagramErrorKind, DiagramRender, DiagramTheme};

pub(super) const DIAGRAM_CACHE_CAPACITY: usize = 128;

/// Diagrams are rasterized above one device pixel per SVG unit and presented at
/// their intrinsic size, because `RenderImage::scale_factor` is `pub(crate)` and
/// cannot tell GPUI these pixels are supersampled. Matches the factor GPUI uses
/// for its own SVG resources.
const DIAGRAM_SUPERSAMPLE: f32 = 2.0;

/// `usvg::Options::default()` carries an empty font database, and usvg converts
/// text to paths at parse time — every `<text>` node would be dropped without a
/// diagnostic, leaving diagrams as unlabelled boxes and arrows.
static DIAGRAM_FONT_DB: LazyLock<Arc<usvg::fontdb::Database>> = LazyLock::new(|| {
    let mut db = usvg::fontdb::Database::new();
    db.load_system_fonts();
    Arc::new(db)
});

fn raster_failure(detail: impl Into<String>) -> DiagramError {
    DiagramError::new(DiagramErrorKind::RenderFailed, detail)
}

/// Rasterizes sanitized diagram SVG into a GPUI-ready image and the size it
/// should be presented at.
///
/// Markion rasterizes rather than handing GPUI `Image::from_bytes(Svg, ..)`:
/// that path uploads `tiny_skia`'s premultiplied-RGBA pixmap as BGRA without
/// swapping channels, and rasterizes at 1x. GPUI exports no way to reuse or
/// configure the renderer it uses for its own SVG resources.
fn rasterize_diagram(
    render: DiagramRender,
) -> Result<(Arc<RenderImage>, Size<Pixels>), DiagramError> {
    let intrinsic = render.intrinsic_size();
    let options = usvg::Options {
        fontdb: DIAGRAM_FONT_DB.clone(),
        ..usvg::Options::default()
    };
    let tree = usvg::Tree::from_data(render.svg().as_bytes(), &options)
        .map_err(|error| raster_failure(format!("diagram SVG could not be parsed: {error}")))?;

    let tree_size = tree.size();
    let width = (tree_size.width() * DIAGRAM_SUPERSAMPLE).ceil() as u32;
    let height = (tree_size.height() * DIAGRAM_SUPERSAMPLE).ceil() as u32;
    let mut pixmap = resvg::tiny_skia::Pixmap::new(width, height).ok_or_else(|| {
        raster_failure(format!("diagram raster size {width}x{height} is not valid"))
    })?;
    resvg::render(
        &tree,
        resvg::tiny_skia::Transform::from_scale(DIAGRAM_SUPERSAMPLE, DIAGRAM_SUPERSAMPLE),
        &mut pixmap.as_mut(),
    );

    // `RenderImage` is BGRA; `tiny_skia` produces premultiplied RGBA.
    let mut pixels = pixmap.take();
    for pixel in pixels.chunks_exact_mut(4) {
        pixel.swap(0, 2);
    }
    let buffer = image::ImageBuffer::from_raw(width, height, pixels)
        .ok_or_else(|| raster_failure("diagram raster buffer does not match its dimensions"))?;

    // The sanitized intrinsic size is authoritative; fall back to the parsed
    // tree so a backend that omits width/height still presents at 1x.
    let presentation = intrinsic.map_or_else(
        || size(px(tree_size.width()), px(tree_size.height())),
        |intrinsic| size(px(intrinsic.width), px(intrinsic.height)),
    );
    Ok((
        Arc::new(RenderImage::new(vec![image::Frame::new(buffer)])),
        presentation,
    ))
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(super) struct DiagramCacheKey {
    pub(super) backend_id: String,
    pub(super) source: String,
    pub(super) theme: DiagramTheme,
}

#[derive(Debug, Clone)]
pub(super) enum DiagramCacheEntry {
    Pending,
    /// The rasterized diagram and the size it is presented at, which is the
    /// sanitized intrinsic size rather than the supersampled pixel count.
    Ready(Arc<RenderImage>, Size<Pixels>),
    Error(Arc<DiagramError>),
}

pub(super) struct DiagramCache {
    capacity: usize,
    entries: HashMap<DiagramCacheKey, DiagramCacheEntry>,
    completed_order: VecDeque<DiagramCacheKey>,
}

impl DiagramCache {
    pub(super) fn new(capacity: usize) -> Self {
        Self {
            capacity,
            entries: HashMap::new(),
            completed_order: VecDeque::new(),
        }
    }

    /// Reserves a render exactly once. If all capacity is occupied by work in
    /// flight, the caller can retry on the next notified frame.
    pub(super) fn reserve_pending(&mut self, key: DiagramCacheKey) -> bool {
        if self.entries.contains_key(&key) || self.capacity == 0 {
            return false;
        }
        while self.entries.len() >= self.capacity {
            let Some(oldest) = self.completed_order.pop_front() else {
                return false;
            };
            if !matches!(self.entries.get(&oldest), Some(DiagramCacheEntry::Pending)) {
                self.entries.remove(&oldest);
            }
        }
        self.entries.insert(key, DiagramCacheEntry::Pending);
        true
    }

    pub(super) fn complete(
        &mut self,
        key: &DiagramCacheKey,
        result: Result<(Arc<RenderImage>, Size<Pixels>), DiagramError>,
    ) {
        if !matches!(self.entries.get(key), Some(DiagramCacheEntry::Pending)) {
            return;
        }
        let entry = match result {
            Ok((image, size)) => DiagramCacheEntry::Ready(image, size),
            Err(error) => DiagramCacheEntry::Error(Arc::new(error)),
        };
        self.entries.insert(key.clone(), entry);
        self.completed_order.push_back(key.clone());
    }

    pub(super) fn get(&self, key: &DiagramCacheKey) -> Option<DiagramCacheEntry> {
        self.entries.get(key).cloned()
    }

    #[cfg(test)]
    fn len(&self) -> usize {
        self.entries.len()
    }
}

impl MarkionApp {
    fn diagram_theme(&self) -> DiagramTheme {
        if self.active_theme_definition().is_dark {
            DiagramTheme::Dark
        } else {
            DiagramTheme::Light
        }
    }

    fn diagram_key(&self, language: Option<&str>, source: &str) -> Option<DiagramCacheKey> {
        Some(DiagramCacheKey {
            backend_id: diagram_backend_id(language)?,
            source: source.to_string(),
            theme: self.diagram_theme(),
        })
    }

    pub(super) fn diagram_entry(
        &self,
        language: Option<&str>,
        source: &str,
    ) -> Option<DiagramCacheEntry> {
        let key = self.diagram_key(language, source)?;
        Some(
            self.diagram_cache
                .get(&key)
                .unwrap_or(DiagramCacheEntry::Pending),
        )
    }

    pub(super) fn ensure_diagram_renders(
        &mut self,
        blocks: &[PreviewBlock],
        cx: &mut Context<Self>,
    ) {
        let mut missing = Vec::new();
        for block in blocks {
            let PreviewBlock::CodeBlock { language, code, .. } = block else {
                continue;
            };
            let Some(key) = self.diagram_key(language.as_deref(), code) else {
                continue;
            };
            if self.diagram_cache.reserve_pending(key.clone()) {
                missing.push(key);
            }
        }

        for key in missing {
            let registry = builtin_diagram_registry();
            let render_key = key.clone();
            cx.spawn(async move |this, cx| {
                let result = cx
                    .background_spawn(async move {
                        registry
                            .render(&render_key.backend_id, &render_key.source, render_key.theme)
                            .and_then(rasterize_diagram)
                    })
                    .await;
                let _ = this.update(cx, |app, cx| {
                    app.diagram_cache.complete(&key, result);
                    cx.notify();
                });
            })
            .detach();
        }
    }

    pub(super) fn diagram_error_message(&self, error: &DiagramError) -> &'static str {
        let message = match error.kind() {
            DiagramErrorKind::UnsupportedBackend => Msg::DiagramUnsupported,
            DiagramErrorKind::InputTooLarge => Msg::DiagramInputTooLarge,
            DiagramErrorKind::InvalidSource => Msg::DiagramInvalidSource,
            DiagramErrorKind::UnsafeOutput => Msg::DiagramUnsafeOutput,
            DiagramErrorKind::RenderFailed => Msg::DiagramRenderFailed,
        };
        t(self.language, message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(source: &str, theme: DiagramTheme) -> DiagramCacheKey {
        DiagramCacheKey {
            backend_id: "mermaid".into(),
            source: source.into(),
            theme,
        }
    }

    fn image(seed: u8) -> (Arc<RenderImage>, Size<Pixels>) {
        let buffer = image::ImageBuffer::from_raw(1, 1, vec![seed, seed, seed, 255])
            .expect("1x1 BGRA buffer");
        (
            Arc::new(RenderImage::new(vec![image::Frame::new(buffer)])),
            size(px(1.), px(1.)),
        )
    }

    fn rasterize(svg: &str) -> (Arc<RenderImage>, Size<Pixels>) {
        let mut registry = markion_diagram::DiagramRegistry::new();
        registry
            .register(StubBackend(svg.to_string()))
            .expect("stub backend registers");
        let render = registry
            .render("stub", "source", DiagramTheme::Light)
            .expect("stub SVG sanitizes");
        rasterize_diagram(render).expect("stub SVG rasterizes")
    }

    /// Feeds fixed SVG through the real sanitize boundary, so these tests cover
    /// the bytes rasterization actually receives.
    struct StubBackend(String);

    impl markion_diagram::DiagramBackend for StubBackend {
        fn id(&self) -> &'static str {
            "stub"
        }

        fn aliases(&self) -> &'static [&'static str] {
            &["stub"]
        }

        fn render(
            &self,
            _request: &markion_diagram::DiagramRenderRequest,
        ) -> Result<markion_diagram::RawDiagramRender, DiagramError> {
            Ok(markion_diagram::RawDiagramRender::svg(self.0.clone()))
        }
    }

    /// GPUI renders `RenderImage` bytes as BGRA, so a red fill must reach it as
    /// B=0, G=0, R=255. Uploading `tiny_skia`'s RGBA unswapped — as GPUI's own
    /// `Image::from_bytes(Svg, ..)` path does — would present red as blue.
    #[test]
    fn rasterized_diagram_pixels_are_bgra() {
        let (image, _) = rasterize(
            r##"<svg xmlns="http://www.w3.org/2000/svg" width="4" height="4"><rect width="4" height="4" fill="#FF0000"/></svg>"##,
        );
        let pixels = image.as_bytes(0).expect("frame 0 has bytes");
        assert_eq!(&pixels[0..4], &[0, 0, 255, 255]);
    }

    /// `usvg::Options::default()` has an empty font database and drops every
    /// `<text>` node without an error, which would render diagrams unlabelled.
    #[test]
    fn rasterized_diagram_draws_text_labels() {
        // Name a family this machine actually has, so the test exercises the
        // font database rather than the host's font inventory. Generic families
        // ("sans-serif") are deliberately avoided: fontdb maps them to names
        // like "FreeSans" that a system need not have installed.
        let Some(family) = DIAGRAM_FONT_DB
            .faces()
            .next()
            .and_then(|face| face.families.first().map(|(name, _)| name.clone()))
        else {
            return; // No fonts installed; nothing to assert about glyph ink.
        };

        // The fixture holds no shapes, so any opaque pixel can only be glyph ink.
        let (image, _) = rasterize(&format!(
            r##"<svg xmlns="http://www.w3.org/2000/svg" width="80" height="40"><text x="4" y="24" font-family="{family}" font-size="16" fill="#000000">Hi</text></svg>"##
        ));
        let ink = image
            .as_bytes(0)
            .expect("frame 0 has bytes")
            .chunks_exact(4)
            .filter(|pixel| pixel[3] > 0)
            .count();
        assert!(
            ink > 0,
            "text labels must survive rasterization; an empty fontdb silently drops them"
        );
    }

    /// The raster is supersampled but must still be presented at 1x, since
    /// `RenderImage::scale_factor` is `pub(crate)` and cannot carry the factor.
    #[test]
    fn rasterized_diagram_is_supersampled_but_presented_at_intrinsic_size() {
        let (image, size) = rasterize(
            r##"<svg xmlns="http://www.w3.org/2000/svg" width="30" height="10"><rect width="30" height="10" fill="#00FF00"/></svg>"##,
        );
        assert_eq!(size.width, px(30.));
        assert_eq!(size.height, px(10.));
        let pixels = image.size(0);
        assert_eq!(pixels.width.0, (30. * DIAGRAM_SUPERSAMPLE) as i32);
        assert_eq!(pixels.height.0, (10. * DIAGRAM_SUPERSAMPLE) as i32);
    }

    #[test]
    fn identical_pending_and_completed_keys_are_shared() {
        let key = key("A --> B", DiagramTheme::Light);
        let mut cache = DiagramCache::new(2);
        assert!(cache.reserve_pending(key.clone()));
        assert!(!cache.reserve_pending(key.clone()));
        cache.complete(&key, Ok(image(1)));
        assert!(!cache.reserve_pending(key.clone()));
        assert!(matches!(
            cache.get(&key),
            Some(DiagramCacheEntry::Ready(_, _))
        ));
    }

    #[test]
    fn completed_eviction_is_fifo_and_never_evicts_pending() {
        let pending = key("pending", DiagramTheme::Light);
        let completed = key("completed", DiagramTheme::Light);
        let replacement = key("replacement", DiagramTheme::Light);
        let mut cache = DiagramCache::new(2);
        assert!(cache.reserve_pending(pending.clone()));
        assert!(cache.reserve_pending(completed.clone()));
        cache.complete(&completed, Ok(image(2)));
        assert!(cache.reserve_pending(replacement.clone()));
        assert_eq!(cache.len(), 2);
        assert!(matches!(
            cache.get(&pending),
            Some(DiagramCacheEntry::Pending)
        ));
        assert!(cache.get(&completed).is_none());
    }

    #[test]
    fn themes_use_independent_keys_and_switching_back_reuses_the_result() {
        let light = key("A --> B", DiagramTheme::Light);
        let dark = key("A --> B", DiagramTheme::Dark);
        let mut cache = DiagramCache::new(2);
        assert!(cache.reserve_pending(light.clone()));
        cache.complete(&light, Ok(image(1)));
        assert!(cache.reserve_pending(dark));
        assert!(!cache.reserve_pending(light));
    }

    #[test]
    fn late_completion_is_key_scoped_and_cannot_mutate_document_state() {
        let old = key("A --> B", DiagramTheme::Light);
        let edited = key("A --> C", DiagramTheme::Light);
        let mut cache = DiagramCache::new(4);
        assert!(cache.reserve_pending(old.clone()));
        assert!(cache.reserve_pending(edited.clone()));

        let closed_tab = EditorTab::new(MarkdownDocument::from_text("```mermaid\nA --> B\n```"));
        let mut live_tab = EditorTab::new(MarkdownDocument::from_text("# Live\n\nbody"));
        live_tab.push_undo_snapshot();
        live_tab.document.replace_range(8..12, "edited");
        let version = live_tab.document.version();
        let dirty = live_tab.document.is_dirty();
        let text = live_tab.document.text().to_string();
        let undo_len = live_tab.undo_stack.len();
        let preview = live_tab.document.preview_blocks_shared();
        let outline = live_tab.document.outline();
        let stats = live_tab.document.stats();
        let display_text = live_tab.shared_document_text();
        let line_offsets = live_tab.shared_line_offsets();

        // A render may outlive the tab that requested it. Completion only
        // updates the shared cache entry selected by its immutable key.
        drop(closed_tab);

        cache.complete(&old, Ok(image(3)));

        assert!(matches!(
            cache.get(&edited),
            Some(DiagramCacheEntry::Pending)
        ));
        assert_eq!(live_tab.document.version(), version);
        assert_eq!(live_tab.document.is_dirty(), dirty);
        assert_eq!(live_tab.document.text(), text);
        assert_eq!(live_tab.undo_stack.len(), undo_len);
        assert_eq!(live_tab.document.outline(), outline);
        assert_eq!(live_tab.document.stats(), stats);
        assert!(Arc::ptr_eq(
            &preview,
            &live_tab.document.preview_blocks_shared()
        ));
        assert_eq!(
            display_text.as_ptr(),
            live_tab.shared_document_text().as_ptr()
        );
        assert!(Rc::ptr_eq(&line_offsets, &live_tab.shared_line_offsets()));
    }
}
