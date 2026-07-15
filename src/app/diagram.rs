use super::*;
use markion_diagram::{DiagramError, DiagramErrorKind, DiagramTheme};

pub(super) const DIAGRAM_CACHE_CAPACITY: usize = 128;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(super) struct DiagramCacheKey {
    pub(super) backend_id: String,
    pub(super) source: String,
    pub(super) theme: DiagramTheme,
}

#[derive(Debug, Clone)]
pub(super) enum DiagramCacheEntry {
    Pending,
    Ready(Arc<Image>),
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
        result: Result<Arc<Image>, DiagramError>,
    ) {
        if !matches!(self.entries.get(key), Some(DiagramCacheEntry::Pending)) {
            return;
        }
        let entry = match result {
            Ok(image) => DiagramCacheEntry::Ready(image),
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
                            .map(|render| {
                                Arc::new(Image::from_bytes(
                                    ImageFormat::Svg,
                                    render.into_svg().into_bytes(),
                                ))
                            })
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

    fn image(seed: u8) -> Arc<Image> {
        Arc::new(Image::from_bytes(ImageFormat::Svg, vec![seed]))
    }

    #[test]
    fn identical_pending_and_completed_keys_are_shared() {
        let key = key("A --> B", DiagramTheme::Light);
        let mut cache = DiagramCache::new(2);
        assert!(cache.reserve_pending(key.clone()));
        assert!(!cache.reserve_pending(key.clone()));
        cache.complete(&key, Ok(image(1)));
        assert!(!cache.reserve_pending(key.clone()));
        assert!(matches!(cache.get(&key), Some(DiagramCacheEntry::Ready(_))));
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
