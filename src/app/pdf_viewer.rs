//! Root-crate PDF presentation integration.
//!
//! The native renderer and its document handles live in the GPUI-free
//! `markion-pdf-viewer` member. This module owns only application scheduling,
//! bounded GPUI image caching, and the PDF surface.

use super::*;
use image::{Frame, RgbaImage};
use markion_pdf_viewer::{
    MAX_RASTER_BYTES, OpenRequest, PdfEvent, PdfService, RenderPriority, RenderRequest,
    RuntimeDiscoveryError, discover_runtime, target_width_bucket,
};
use std::sync::atomic::{AtomicU64, Ordering};

pub(super) const PDF_CACHE_MAX_BYTES: usize = 64 * 1_048_576;
const PDF_CACHE_MAX_ENTRIES: usize = 128;
const PDF_EVENT_POLL_INTERVAL: Duration = Duration::from_millis(16);
const PDF_DEFAULT_RENDER_WIDTH_PX: u32 = 1024;
pub(super) const PDF_PAGE_ROW_PADDING_Y: f32 = 12.0;
pub(super) const PDF_PAGE_BORDER_WIDTH: f32 = 1.0;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(super) struct PdfPagePlacement {
    pub(super) top: f32,
    pub(super) logical_width: f32,
    pub(super) logical_height: f32,
    pub(super) row_height: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(super) struct PdfPageKey {
    pub(super) document_id: DocumentId,
    pub(super) generation: Generation,
    pub(super) page_index: u32,
    pub(super) width_bucket_px: u32,
}

#[derive(Clone)]
pub(super) struct PdfPageReady {
    pub(super) image: Arc<RenderImage>,
    pub(super) byte_len: usize,
    pub(super) width_px: u32,
    pub(super) height_px: u32,
}

#[derive(Clone)]
pub(super) enum PdfPageEntry {
    Pending,
    Ready(PdfPageReady),
    Error(PdfErrorKind),
}

pub(super) struct PdfPageCache {
    entries: HashMap<PdfPageKey, PdfPageEntry>,
    completed_order: VecDeque<PdfPageKey>,
    claims: HashMap<PdfPageKey, usize>,
    completed_bytes: usize,
}

impl PdfPageCache {
    pub(super) fn new() -> Self {
        Self {
            entries: HashMap::new(),
            completed_order: VecDeque::new(),
            claims: HashMap::new(),
            completed_bytes: 0,
        }
    }

    pub(super) fn get(&mut self, key: &PdfPageKey) -> Option<PdfPageEntry> {
        let entry = self.entries.get(key).cloned();
        if matches!(entry, Some(PdfPageEntry::Ready(_) | PdfPageEntry::Error(_))) {
            self.touch(key);
        }
        entry
    }

    pub(super) fn reserve_pending(&mut self, key: PdfPageKey) -> bool {
        if self.entries.contains_key(&key) {
            return false;
        }
        if self
            .entries
            .values()
            .filter(|entry| matches!(entry, PdfPageEntry::Pending))
            .count()
            >= PDF_CACHE_MAX_ENTRIES
        {
            return false;
        }
        self.entries.insert(key, PdfPageEntry::Pending);
        true
    }

    pub(super) fn claim(&mut self, key: PdfPageKey) {
        *self.claims.entry(key).or_insert(0) += 1;
    }

    pub(super) fn release(&mut self, key: &PdfPageKey) -> Vec<Arc<RenderImage>> {
        if let Some(count) = self.claims.get_mut(key) {
            *count = count.saturating_sub(1);
            if *count == 0 {
                self.claims.remove(key);
            }
        }
        let mut dropped = Vec::new();
        if self.claims.get(key).copied().unwrap_or(0) == 0
            && matches!(self.entries.get(key), Some(PdfPageEntry::Pending))
        {
            self.entries.remove(key);
        }
        self.enforce_budget(&mut dropped);
        dropped
    }

    pub(super) fn complete(
        &mut self,
        key: PdfPageKey,
        ready: PdfPageReady,
    ) -> Vec<Arc<RenderImage>> {
        let mut dropped = Vec::new();
        if ready.byte_len > MAX_RASTER_BYTES || self.claims.get(&key).copied().unwrap_or(0) == 0 {
            self.entries.remove(&key);
            dropped.push(ready.image);
            return dropped;
        }
        self.remove_completed(&key, &mut dropped);
        self.completed_bytes = self.completed_bytes.saturating_add(ready.byte_len);
        self.entries.insert(key, PdfPageEntry::Ready(ready));
        self.completed_order.push_back(key);
        self.enforce_budget(&mut dropped);
        if self.completed_bytes > PDF_CACHE_MAX_BYTES + MAX_RASTER_BYTES {
            self.remove_entry(&key, &mut dropped);
            self.entries
                .insert(key, PdfPageEntry::Error(PdfErrorKind::RasterTooLarge));
            self.completed_order.push_back(key);
        }
        dropped
    }

    pub(super) fn fail(&mut self, key: PdfPageKey, error: PdfErrorKind) -> Vec<Arc<RenderImage>> {
        let mut dropped = Vec::new();
        self.remove_completed(&key, &mut dropped);
        self.entries.insert(key, PdfPageEntry::Error(error));
        self.completed_order.retain(|candidate| candidate != &key);
        self.completed_order.push_back(key);
        while self.completed_order.len() > PDF_CACHE_MAX_ENTRIES {
            let Some(oldest) = self.completed_order.pop_front() else {
                break;
            };
            if self.claims.get(&oldest).copied().unwrap_or(0) == 0 {
                self.entries.remove(&oldest);
            }
        }
        dropped
    }

    pub(super) fn remove_document(&mut self, document_id: DocumentId) -> Vec<Arc<RenderImage>> {
        let keys = self
            .entries
            .keys()
            .copied()
            .filter(|key| key.document_id == document_id)
            .collect::<Vec<_>>();
        let mut dropped = Vec::new();
        for key in keys {
            self.claims.remove(&key);
            self.remove_entry(&key, &mut dropped);
        }
        dropped
    }

    pub(super) fn pending_count(&self) -> usize {
        self.entries
            .values()
            .filter(|entry| matches!(entry, PdfPageEntry::Pending))
            .count()
    }

    pub(super) fn completed_bytes(&self) -> usize {
        self.completed_bytes
    }

    pub(super) fn accounting_counts(&self) -> (usize, usize, usize, usize, usize) {
        let pending = self
            .entries
            .values()
            .filter(|entry| matches!(entry, PdfPageEntry::Pending))
            .count();
        let ready = self
            .entries
            .values()
            .filter(|entry| matches!(entry, PdfPageEntry::Ready(_)))
            .count();
        let errors = self
            .entries
            .values()
            .filter(|entry| matches!(entry, PdfPageEntry::Error(_)))
            .count();
        (
            self.entries.len(),
            pending,
            ready,
            errors,
            self.claims.len(),
        )
    }

    fn touch(&mut self, key: &PdfPageKey) {
        self.completed_order.retain(|candidate| candidate != key);
        self.completed_order.push_back(*key);
    }

    fn remove_completed(&mut self, key: &PdfPageKey, dropped: &mut Vec<Arc<RenderImage>>) {
        if matches!(self.entries.get(key), Some(PdfPageEntry::Ready(_))) {
            self.remove_entry(key, dropped);
        }
    }

    fn remove_entry(&mut self, key: &PdfPageKey, dropped: &mut Vec<Arc<RenderImage>>) {
        if let Some(PdfPageEntry::Ready(ready)) = self.entries.remove(key) {
            self.completed_bytes = self.completed_bytes.saturating_sub(ready.byte_len);
            dropped.push(ready.image);
        } else {
            self.entries.remove(key);
        }
        self.completed_order.retain(|candidate| candidate != key);
    }

    fn enforce_budget(&mut self, dropped: &mut Vec<Arc<RenderImage>>) {
        while self.completed_bytes > PDF_CACHE_MAX_BYTES {
            let Some(key) = self
                .completed_order
                .iter()
                .find(|key| self.claims.get(key).copied().unwrap_or(0) == 0)
                .copied()
            else {
                break;
            };
            self.remove_entry(&key, dropped);
        }
    }
}

pub(super) fn pdf_page_layout(
    geometry: PageGeometry,
    zoom: PdfZoomMode,
    viewport_width: f32,
    display_scale: f32,
) -> (f32, f32, u32) {
    let available_width = (viewport_width - 48.0).max(1.0);
    let logical_width = match zoom {
        PdfZoomMode::FitWidth => available_width,
        PdfZoomMode::Percent(percent) => geometry.width_points * percent / 100.0,
    }
    .max(1.0);
    let logical_height = logical_width * geometry.height_points / geometry.width_points;
    let target_width = (logical_width * display_scale.max(1.0)).ceil().max(1.0) as u32;
    (logical_width, logical_height.max(1.0), target_width)
}

pub(super) fn pdf_page_placements(
    pages: &[PageGeometry],
    zoom: PdfZoomMode,
    viewport_width: f32,
    display_scale: f32,
) -> (Vec<PdfPagePlacement>, f32) {
    let mut top = 0.0;
    let placements = pages
        .iter()
        .copied()
        .map(|geometry| {
            let (logical_width, logical_height, _) =
                pdf_page_layout(geometry, zoom, viewport_width, display_scale);
            let row_height =
                logical_height + PDF_PAGE_ROW_PADDING_Y * 2.0 + PDF_PAGE_BORDER_WIDTH * 2.0;
            let placement = PdfPagePlacement {
                top,
                logical_width,
                logical_height,
                row_height,
            };
            top += row_height;
            placement
        })
        .collect();
    (placements, top)
}

pub(super) fn pdf_visible_range(
    placements: &[PdfPagePlacement],
    scroll_top: f32,
    viewport_height: f32,
) -> Range<usize> {
    if placements.is_empty() {
        return 0..0;
    }
    let scroll_top = scroll_top.max(0.0);
    let scroll_bottom = scroll_top + viewport_height.max(1.0);
    let start = placements
        .partition_point(|page| page.top + page.row_height <= scroll_top)
        .min(placements.len() - 1);
    let end = placements
        .partition_point(|page| page.top < scroll_bottom)
        .max(start + 1)
        .min(placements.len());
    start..end
}

pub(super) fn pdf_prefetch_range(page_count: usize, visible: Range<usize>) -> Range<usize> {
    if page_count == 0 {
        return 0..0;
    }
    let visible_start = visible.start.min(page_count - 1);
    let visible_end = visible.end.max(visible_start + 1).min(page_count);
    visible_start.saturating_sub(1)..visible_end.saturating_add(1).min(page_count)
}

fn next_pdf_request_id() -> RequestId {
    static NEXT: AtomicU64 = AtomicU64::new(1);
    RequestId(NEXT.fetch_add(1, Ordering::Relaxed))
}

fn pdf_resource_root() -> PathBuf {
    if let Some(logo) = bundled_resource_path("assets/markion.png")
        && let Some(root) = logo.parent().and_then(Path::parent)
    {
        return root.to_path_buf();
    }
    env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(Path::to_path_buf))
        .unwrap_or_else(|| PathBuf::from("."))
}

fn map_runtime_error(error: RuntimeDiscoveryError) -> PdfErrorKind {
    match error {
        RuntimeDiscoveryError::Missing(_) | RuntimeDiscoveryError::NotAFile(_) => {
            PdfErrorKind::RuntimeMissing
        }
        RuntimeDiscoveryError::DeveloperOverrideForbidden | RuntimeDiscoveryError::LoadFailed => {
            PdfErrorKind::RuntimeUnavailable
        }
    }
}

pub(super) fn pdf_error_message(error: PdfErrorKind) -> Msg {
    match error {
        PdfErrorKind::RuntimeMissing => Msg::PdfErrorRuntimeMissing,
        PdfErrorKind::RuntimeUnavailable | PdfErrorKind::ServiceStopped => {
            Msg::PdfErrorRuntimeUnavailable
        }
        PdfErrorKind::FileUnavailable => Msg::PdfErrorFileUnavailable,
        PdfErrorKind::SourceTooLarge => Msg::PdfErrorSourceTooLarge,
        PdfErrorKind::PageLimitExceeded => Msg::PdfErrorPageLimit,
        PdfErrorKind::Encrypted => Msg::PdfErrorEncrypted,
        PdfErrorKind::CorruptOrUnsupported
        | PdfErrorKind::InvalidPageGeometry
        | PdfErrorKind::InvalidPageNumber => Msg::PdfErrorCorrupt,
        PdfErrorKind::PageUnavailable
        | PdfErrorKind::RasterTooLarge
        | PdfErrorKind::InvalidRaster => Msg::PdfErrorPageUnavailable,
        PdfErrorKind::QueueFull | PdfErrorKind::StaleRequest | PdfErrorKind::Internal => {
            Msg::PdfErrorGeneric
        }
    }
}

fn raster_to_ready(raster: markion_pdf_viewer::PageRaster) -> Result<PdfPageReady, PdfErrorKind> {
    let byte_len = raster.byte_len();
    let buffer = RgbaImage::from_raw(raster.width_px, raster.height_px, raster.rgba)
        .ok_or(PdfErrorKind::InvalidRaster)?;
    Ok(PdfPageReady {
        image: Arc::new(RenderImage::new(vec![Frame::new(buffer)])),
        byte_len,
        width_px: raster.width_px,
        height_px: raster.height_px,
    })
}

impl MarkionApp {
    pub(super) fn editor_tab_for_pdf(&self, path: PathBuf) -> EditorTab {
        EditorTab::new_pdf(comparable_document_path(&path), next_pdf_request_id())
    }

    pub(super) fn open_pdf_in_new_tab(&mut self, path: PathBuf, cx: &mut Context<Self>) {
        let tab = self.editor_tab_for_pdf(path);
        self.open_tab_in_new_tab(tab, cx);
        self.begin_pdf_open(self.active_tab, cx);
    }

    pub(super) fn replace_active_tab_with_pdf(&mut self, path: PathBuf, cx: &mut Context<Self>) {
        let tab = self.editor_tab_for_pdf(path);
        self.replace_active_with_tab(tab, cx);
        self.begin_pdf_open(self.active_tab, cx);
    }

    fn ensure_pdf_service(&mut self) -> Result<(), PdfErrorKind> {
        if self.pdf_service.is_some() {
            return Ok(());
        }
        if let Some(error) = self.pdf_service_error {
            return Err(error);
        }
        let runtime = discover_runtime(pdf_resource_root()).map_err(map_runtime_error)?;
        let service = PdfService::start_native(runtime)?;
        self.pdf_service = Some(service);
        Ok(())
    }

    pub(super) fn begin_pdf_open(&mut self, tab_index: usize, cx: &mut Context<Self>) {
        let Some(request) = self
            .tabs
            .get(tab_index)
            .and_then(|tab| tab.pdf())
            .map(|pdf| OpenRequest {
                request_id: pdf.request_id,
                path: pdf.path.clone(),
            })
        else {
            return;
        };
        let result = self.ensure_pdf_service().and_then(|()| {
            self.pdf_service
                .as_ref()
                .unwrap()
                .open(request.clone())
                .map(|_| ())
        });
        if let Err(error) = result {
            if let Some(pdf) = self.tabs.get_mut(tab_index).and_then(|tab| tab.pdf_mut()) {
                pdf.load_state = PdfLoadState::Error(error);
            }
            self.pdf_service_error = Some(error);
            cx.notify();
            return;
        }
        self.schedule_pdf_event_poll(cx);
    }

    pub(super) fn schedule_pdf_event_poll(&mut self, cx: &mut Context<Self>) {
        if self.pdf_event_poll_scheduled || self.pdf_service.is_none() {
            return;
        }
        self.pdf_event_poll_scheduled = true;
        cx.spawn(async move |this, cx| {
            Timer::after(PDF_EVENT_POLL_INTERVAL).await;
            let _ = this.update(cx, |app, cx| {
                app.pdf_event_poll_scheduled = false;
                app.drain_pdf_events(cx);
                if app.pdf_has_pending_work() {
                    app.schedule_pdf_event_poll(cx);
                }
            });
        })
        .detach();
    }

    fn pdf_has_pending_work(&self) -> bool {
        self.tabs.iter().any(|tab| {
            tab.pdf()
                .is_some_and(|pdf| matches!(pdf.load_state, PdfLoadState::Loading))
        }) || self.pdf_page_cache.pending_count() > 0
    }

    fn drain_pdf_events(&mut self, cx: &mut Context<Self>) {
        loop {
            let event = self
                .pdf_service
                .as_ref()
                .and_then(|service| service.try_recv().ok());
            let Some(event) = event else {
                break;
            };
            self.apply_pdf_event(event, cx);
        }
    }

    pub(super) fn apply_pdf_event(&mut self, event: PdfEvent, cx: &mut Context<Self>) {
        match event {
            PdfEvent::ServiceReady => {}
            PdfEvent::ServiceFailed(error) => {
                self.pdf_service_error = Some(error);
                for tab in &mut self.tabs {
                    if let Some(pdf) = tab.pdf_mut()
                        && matches!(pdf.load_state, PdfLoadState::Loading)
                    {
                        pdf.load_state = PdfLoadState::Error(error);
                    }
                }
            }
            PdfEvent::Opened(opened) => {
                let opened_path = comparable_document_path(&opened.path);
                let Some(index) = self.tabs.iter().position(|tab| {
                    tab.pdf().is_some_and(|pdf| {
                        pdf.path == opened_path && matches!(pdf.load_state, PdfLoadState::Loading)
                    })
                }) else {
                    if let Some(service) = &self.pdf_service {
                        let _ = service.close(opened.document_id);
                    }
                    return;
                };
                if let Some(pdf) = self.tabs[index].pdf_mut() {
                    pdf.document_id = Some(opened.document_id);
                    pdf.pages = Arc::from(opened.pages);
                    pdf.page_layout = Arc::from([]);
                    pdf.page_content_height = px(0.);
                    pdf.visible_range = 0..0;
                    pdf.page_scroll.set_offset(point(px(0.), px(0.)));
                    pdf.load_state = PdfLoadState::Ready;
                    pdf.current_page = 0;
                }
                self.schedule_pdf_visible_range(index, 0..1, cx);
            }
            PdfEvent::OpenFailed { request, error } => {
                if let Some(pdf) = self.tabs.iter_mut().find_map(|tab| {
                    tab.pdf_mut()
                        .filter(|pdf| pdf.request_id == request.request_id)
                }) {
                    pdf.load_state = PdfLoadState::Error(error);
                }
            }
            PdfEvent::PageReady { request, raster } => {
                let key = PdfPageKey {
                    document_id: request.document_id,
                    generation: request.generation,
                    page_index: request.page_index,
                    width_bucket_px: request.target_width_px,
                };
                let current = self.tabs.iter().any(|tab| {
                    tab.pdf().is_some_and(|pdf| {
                        pdf.document_id == Some(request.document_id)
                            && pdf.generation == request.generation
                            && pdf.claimed_pages.contains(&key)
                    })
                });
                if current {
                    match raster_to_ready(raster) {
                        Ok(ready) => {
                            for image in self.pdf_page_cache.complete(key, ready) {
                                cx.drop_image(image, None);
                            }
                        }
                        Err(error) => {
                            for image in self.pdf_page_cache.fail(key, error) {
                                cx.drop_image(image, None);
                            }
                        }
                    }
                }
            }
            PdfEvent::PageFailed { request, error } => {
                let key = PdfPageKey {
                    document_id: request.document_id,
                    generation: request.generation,
                    page_index: request.page_index,
                    width_bucket_px: request.target_width_px,
                };
                if self.tabs.iter().any(|tab| {
                    tab.pdf().is_some_and(|pdf| {
                        pdf.document_id == Some(request.document_id)
                            && pdf.generation == request.generation
                            && pdf.claimed_pages.contains(&key)
                    })
                }) {
                    for image in self.pdf_page_cache.fail(key, error) {
                        cx.drop_image(image, None);
                    }
                }
            }
            PdfEvent::Closed(_) => {}
        }
        cx.notify();
    }

    pub(super) fn prepare_pdf_surface(
        &mut self,
        viewport_width: Pixels,
        display_scale: f32,
        cx: &mut Context<Self>,
    ) {
        let index = self.active_tab;
        let changed = self
            .tabs
            .get(index)
            .and_then(|tab| tab.pdf())
            .is_some_and(|pdf| {
                matches!(pdf.zoom, PdfZoomMode::FitWidth)
                    && (f32::from(pdf.viewport_width) - f32::from(viewport_width)).abs() >= 1.0
            });
        if changed {
            self.invalidate_pdf_generation(index, cx);
        }
        let Some(pdf) = self.tabs.get_mut(index).and_then(|tab| tab.pdf_mut()) else {
            return;
        };
        pdf.viewport_width = viewport_width;
        pdf.display_scale = display_scale.max(1.0);
        let layout_changed = pdf.page_layout.len() != pdf.pages.len()
            || pdf.layout_zoom != Some(pdf.zoom)
            || (matches!(pdf.zoom, PdfZoomMode::FitWidth)
                && (f32::from(pdf.layout_viewport_width) - f32::from(viewport_width)).abs() >= 1.0);
        if layout_changed {
            let anchor = pdf.current_page.min(pdf.pages.len().saturating_sub(1));
            let (placements, content_height) = pdf_page_placements(
                &pdf.pages,
                pdf.zoom,
                f32::from(viewport_width),
                pdf.display_scale,
            );
            pdf.page_layout = Arc::from(placements);
            pdf.page_content_height = px(content_height);
            pdf.layout_zoom = Some(pdf.zoom);
            pdf.layout_viewport_width = viewport_width;
            let anchor_top = pdf
                .page_layout
                .get(anchor)
                .map(|page| page.top)
                .unwrap_or(0.0);
            pdf.page_scroll.set_offset(point(px(0.), px(-anchor_top)));
        }
        let viewport_height = f32::from(pdf.page_scroll.bounds().size.height).max(1.0);
        let scroll_top = (-f32::from(pdf.page_scroll.offset().y)).max(0.0);
        let visible = pdf_visible_range(&pdf.page_layout, scroll_top, viewport_height);
        pdf.visible_range = visible.clone();
        self.schedule_pdf_visible_range(index, visible, cx);
    }

    pub(super) fn schedule_pdf_visible_range(
        &mut self,
        tab_index: usize,
        visible: Range<usize>,
        cx: &mut Context<Self>,
    ) {
        let Some(pdf) = self.tabs.get(tab_index).and_then(|tab| tab.pdf()) else {
            return;
        };
        if !matches!(pdf.load_state, PdfLoadState::Ready) || pdf.pages.is_empty() {
            return;
        }
        let page_count = pdf.pages.len();
        let visible_start = visible.start.min(page_count - 1);
        let visible_end = visible.end.max(visible_start + 1).min(page_count);
        let wanted_range = pdf_prefetch_range(page_count, visible_start..visible_end);
        let document_id = pdf.document_id.expect("ready PDF has a document id");
        let generation = pdf.generation;
        let viewport_width = if pdf.viewport_width > px(0.) {
            f32::from(pdf.viewport_width)
        } else {
            PDF_DEFAULT_RENDER_WIDTH_PX as f32
        };
        let display_scale = pdf.display_scale;
        let zoom = pdf.zoom;
        let pages = Arc::clone(&pdf.pages);

        let mut wanted = Vec::new();
        for page_index in wanted_range {
            let (_, _, target_width) =
                pdf_page_layout(pages[page_index], zoom, viewport_width, display_scale);
            let Ok(width_bucket_px) = target_width_bucket(target_width) else {
                continue;
            };
            wanted.push((
                PdfPageKey {
                    document_id,
                    generation,
                    page_index: page_index as u32,
                    width_bucket_px,
                },
                if (visible_start..visible_end).contains(&page_index) {
                    RenderPriority::Visible
                } else {
                    RenderPriority::Adjacent
                },
            ));
        }
        let wanted_keys = wanted.iter().map(|(key, _)| *key).collect::<HashSet<_>>();
        let old_keys = self.tabs[tab_index]
            .pdf_mut()
            .map(|pdf| std::mem::take(&mut pdf.claimed_pages))
            .unwrap_or_default();
        for key in old_keys.difference(&wanted_keys) {
            for image in self.pdf_page_cache.release(key) {
                cx.drop_image(image, None);
            }
        }
        for key in wanted_keys.difference(&old_keys) {
            self.pdf_page_cache.claim(*key);
        }
        if let Some(pdf) = self.tabs[tab_index].pdf_mut() {
            pdf.claimed_pages = wanted_keys;
            pdf.current_page = visible_start;
        }
        for (key, priority) in wanted {
            if self.pdf_page_cache.reserve_pending(key)
                && let Some(service) = &self.pdf_service
            {
                if let Err(error) = service.render(RenderRequest {
                    document_id: key.document_id,
                    generation: key.generation,
                    page_index: key.page_index,
                    target_width_px: key.width_bucket_px,
                    priority,
                }) {
                    for image in self.pdf_page_cache.fail(key, error) {
                        cx.drop_image(image, None);
                    }
                }
            }
        }
        self.schedule_pdf_event_poll(cx);
    }

    pub(super) fn pdf_page_entry(&mut self, page_index: usize) -> Option<PdfPageEntry> {
        let pdf = self.active_tab().pdf()?;
        let document_id = pdf.document_id?;
        let geometry = *pdf.pages.get(page_index)?;
        let (_, _, target_width) = pdf_page_layout(
            geometry,
            pdf.zoom,
            if pdf.viewport_width > px(0.) {
                f32::from(pdf.viewport_width)
            } else {
                PDF_DEFAULT_RENDER_WIDTH_PX as f32
            },
            pdf.display_scale,
        );
        let width_bucket_px = target_width_bucket(target_width).ok()?;
        self.pdf_page_cache.get(&PdfPageKey {
            document_id,
            generation: pdf.generation,
            page_index: page_index as u32,
            width_bucket_px,
        })
    }

    pub(super) fn invalidate_pdf_generation(&mut self, tab_index: usize, cx: &mut Context<Self>) {
        let old_keys = self
            .tabs
            .get_mut(tab_index)
            .and_then(|tab| tab.pdf_mut())
            .map(|pdf| {
                pdf.generation = Generation(pdf.generation.0.wrapping_add(1).max(1));
                std::mem::take(&mut pdf.claimed_pages)
            });
        for key in old_keys.unwrap_or_default() {
            for image in self.pdf_page_cache.release(&key) {
                cx.drop_image(image, None);
            }
        }
    }

    pub(super) fn release_tab_pdf_claims(&mut self, tab_index: usize, cx: &mut Context<Self>) {
        let keys = self
            .tabs
            .get_mut(tab_index)
            .and_then(|tab| tab.pdf_mut())
            .map(|pdf| std::mem::take(&mut pdf.claimed_pages));
        for key in keys.unwrap_or_default() {
            for image in self.pdf_page_cache.release(&key) {
                cx.drop_image(image, None);
            }
        }
    }

    pub(super) fn close_tab_pdf_resources(&mut self, tab_index: usize, cx: &mut Context<Self>) {
        self.release_tab_pdf_claims(tab_index, cx);
        let document_id = self
            .tabs
            .get(tab_index)
            .and_then(|tab| tab.pdf())
            .and_then(|pdf| pdf.document_id);
        if let Some(document_id) = document_id {
            if let Some(service) = &self.pdf_service {
                let _ = service.close(document_id);
            }
            for image in self.pdf_page_cache.remove_document(document_id) {
                cx.drop_image(image, None);
            }
        }
    }

    pub(super) fn set_pdf_zoom(&mut self, zoom: PdfZoomMode, cx: &mut Context<Self>) {
        let index = self.active_tab;
        let Some(pdf) = self.tabs.get_mut(index).and_then(|tab| tab.pdf_mut()) else {
            return;
        };
        if pdf.zoom == zoom {
            return;
        }
        pdf.zoom = zoom;
        self.invalidate_pdf_generation(index, cx);
        let current = self.tabs[index]
            .pdf()
            .map(|pdf| pdf.current_page)
            .unwrap_or(0);
        self.schedule_pdf_visible_range(index, current..current.saturating_add(1), cx);
        cx.notify();
    }

    pub(super) fn step_pdf_zoom(&mut self, delta: f32, cx: &mut Context<Self>) {
        let current = self
            .active_tab()
            .pdf()
            .and_then(|pdf| pdf.zoom.percent())
            .unwrap_or(100.0);
        self.set_pdf_zoom(PdfZoomMode::numeric(current + delta), cx);
    }

    pub(super) fn jump_to_pdf_page(&mut self, page_index: usize, cx: &mut Context<Self>) {
        let index = self.active_tab;
        self.pdf_page_input = None;
        let Some(pdf) = self.tabs.get_mut(index).and_then(|tab| tab.pdf_mut()) else {
            return;
        };
        if page_index >= pdf.pages.len() {
            return;
        }
        pdf.current_page = page_index;
        if let Some(page) = pdf.page_layout.get(page_index) {
            pdf.page_scroll.set_offset(point(px(0.), px(-page.top)));
        }
        self.schedule_pdf_visible_range(index, page_index..page_index + 1, cx);
        cx.notify();
    }

    pub(super) fn begin_pdf_page_input(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let ready = self.active_tab().pdf().is_some_and(|pdf| {
            matches!(pdf.load_state, PdfLoadState::Ready) && !pdf.pages.is_empty()
        });
        if !ready {
            return;
        }
        self.pdf_page_input = Some(String::new());
        self.input_marked_len = 0;
        window.focus(&self.focus_handle);
        cx.notify();
    }

    pub(super) fn confirm_pdf_page_input(&mut self, cx: &mut Context<Self>) -> bool {
        let Some(input) = self.pdf_page_input.take() else {
            return false;
        };
        self.input_marked_len = 0;
        let page_count = self
            .active_tab()
            .pdf()
            .map(|pdf| pdf.pages.len())
            .unwrap_or(0);
        let page = input.parse::<usize>().ok();
        if let Some(page) = page.filter(|page| (1..=page_count).contains(page)) {
            self.jump_to_pdf_page(page - 1, cx);
        } else {
            self.pdf_page_input = Some(input);
            self.status = t(self.language, Msg::PdfPageInvalid).into();
            cx.notify();
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ready(byte_len: usize) -> PdfPageReady {
        let buffer = RgbaImage::from_raw(1, 1, vec![0; 4]).unwrap();
        PdfPageReady {
            image: Arc::new(RenderImage::new(vec![Frame::new(buffer)])),
            byte_len,
            width_px: 1,
            height_px: 1,
        }
    }

    fn key(page_index: u32) -> PdfPageKey {
        PdfPageKey {
            document_id: DocumentId(1),
            generation: Generation(1),
            page_index,
            width_bucket_px: 64,
        }
    }

    #[test]
    fn fit_width_and_numeric_layout_preserve_aspect_and_bounds() {
        let geometry = PageGeometry::new(600.0, 800.0).unwrap();
        let (width, height, target) = pdf_page_layout(geometry, PdfZoomMode::FitWidth, 448.0, 1.0);
        assert_eq!((width, target), (400.0, 400));
        assert!((height - 533.3333).abs() < 0.001);
        let (wide_width, wide_height, wide_target) =
            pdf_page_layout(geometry, PdfZoomMode::FitWidth, 848.0, 1.0);
        assert_eq!((wide_width, wide_target), (800.0, 800));
        assert!((wide_height - 1066.6666).abs() < 0.001);
        assert_eq!(
            pdf_page_layout(geometry, PdfZoomMode::numeric(10.0), 448.0, 2.0),
            (150.0, 200.0, 300)
        );
        assert_eq!(PdfZoomMode::numeric(500.0).percent(), Some(400.0));
    }

    #[test]
    fn page_placements_expose_full_scroll_height_without_mounting_every_page() {
        let pages = [
            PageGeometry::new(600.0, 800.0).unwrap(),
            PageGeometry::new(800.0, 400.0).unwrap(),
            PageGeometry::new(400.0, 600.0).unwrap(),
        ];
        let (placements, content_height) =
            pdf_page_placements(&pages, PdfZoomMode::FitWidth, 448.0, 1.0);
        assert_eq!(placements.len(), 3);
        assert!((placements[1].top - placements[0].row_height).abs() < 0.001);
        assert!(
            (placements[2].top - (placements[0].row_height + placements[1].row_height)).abs()
                < 0.001
        );
        assert!(
            (content_height - placements.iter().map(|page| page.row_height).sum::<f32>()).abs()
                < 0.001
        );
        assert_eq!(pdf_visible_range(&placements, 0.0, 500.0), 0..1);
        assert_eq!(
            pdf_visible_range(&placements, placements[1].top + 1.0, 100.0),
            1..2
        );
    }

    #[test]
    fn cache_pays_down_one_page_overshoot_after_release() {
        let mut cache = PdfPageCache::new();
        let page_bytes = 24 * 1_048_576;
        for page in 0..3 {
            let key = key(page);
            cache.claim(key);
            assert!(cache.reserve_pending(key));
            let dropped = cache.complete(key, ready(page_bytes));
            assert!(dropped.is_empty());
        }
        assert_eq!(cache.completed_bytes(), 72 * 1_048_576);
        let dropped = cache.release(&key(0));
        assert_eq!(dropped.len(), 1);
        assert!(cache.completed_bytes() <= PDF_CACHE_MAX_BYTES);
    }

    #[test]
    fn unclaimed_late_page_is_not_admitted() {
        let mut cache = PdfPageCache::new();
        assert!(cache.reserve_pending(key(0)));
        assert_eq!(cache.complete(key(0), ready(4096)).len(), 1);
        assert_eq!(cache.completed_bytes(), 0);
        assert!(cache.get(&key(0)).is_none());
    }

    #[test]
    fn exact_cache_boundary_is_retained_without_overshoot() {
        let mut cache = PdfPageCache::new();
        for page in 0..2 {
            let key = key(page);
            cache.claim(key);
            assert!(cache.reserve_pending(key));
            assert!(cache.complete(key, ready(MAX_RASTER_BYTES)).is_empty());
        }
        assert_eq!(cache.completed_bytes(), PDF_CACHE_MAX_BYTES);
        assert_eq!(cache.accounting_counts().2, 2);
    }

    #[test]
    fn releasing_an_unfinished_claim_removes_its_pending_entry() {
        let mut cache = PdfPageCache::new();
        let key = key(0);
        cache.claim(key);
        assert!(cache.reserve_pending(key));
        assert_eq!(cache.pending_count(), 1);
        assert!(cache.release(&key).is_empty());
        assert_eq!(cache.pending_count(), 0);
        assert!(cache.get(&key).is_none());
    }

    #[test]
    fn safe_error_messages_cover_every_native_category_without_diagnostics() {
        for error in [
            PdfErrorKind::RuntimeMissing,
            PdfErrorKind::RuntimeUnavailable,
            PdfErrorKind::FileUnavailable,
            PdfErrorKind::SourceTooLarge,
            PdfErrorKind::PageLimitExceeded,
            PdfErrorKind::Encrypted,
            PdfErrorKind::CorruptOrUnsupported,
            PdfErrorKind::InvalidPageGeometry,
            PdfErrorKind::InvalidPageNumber,
            PdfErrorKind::PageUnavailable,
            PdfErrorKind::RasterTooLarge,
            PdfErrorKind::InvalidRaster,
            PdfErrorKind::QueueFull,
            PdfErrorKind::StaleRequest,
            PdfErrorKind::ServiceStopped,
            PdfErrorKind::Internal,
        ] {
            assert!(!t(Language::En, pdf_error_message(error)).is_empty());
        }
    }

    #[test]
    fn prefetch_range_contains_only_visible_pages_and_one_neighbor_per_side() {
        assert_eq!(pdf_prefetch_range(0, 0..1), 0..0);
        assert_eq!(pdf_prefetch_range(10, 0..2), 0..3);
        assert_eq!(pdf_prefetch_range(10, 3..6), 2..7);
        assert_eq!(pdf_prefetch_range(10, 8..10), 7..10);
        assert_eq!(pdf_prefetch_range(10, 99..100), 8..10);
    }
}
