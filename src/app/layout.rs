use super::*;

const LAYOUT_PERSIST_DEBOUNCE: Duration = Duration::from_millis(300);

/// Map a session `[layout]` table onto GPUI window bounds for `open_window`.
///
/// A recorded origin that no longer intersects any display is discarded and
/// the (clamped) size is centered on the primary display as a windowed window.
pub(super) fn startup_window_bounds(layout: &SessionLayout, cx: &App) -> WindowBounds {
    let (width, height) = layout.normalized_window_size();
    let size = size(px(width), px(height));
    let displays = current_display_rects(cx);
    let visible_origin = match (layout.x, layout.y) {
        (Some(x), Some(y)) if layout_rect_is_visible((x, y, width, height), &displays) => {
            Some(point(px(x), px(y)))
        }
        _ => None,
    };

    match visible_origin {
        Some(origin) => {
            let bounds = Bounds { origin, size };
            if layout.maximized {
                WindowBounds::Maximized(bounds)
            } else {
                WindowBounds::Windowed(bounds)
            }
        }
        None => WindowBounds::Windowed(Bounds::centered(None, size, cx)),
    }
}

fn current_display_rects(cx: &App) -> Vec<(f32, f32, f32, f32)> {
    cx.displays()
        .into_iter()
        .map(|display| {
            let bounds = display.bounds();
            (
                f32::from(bounds.origin.x),
                f32::from(bounds.origin.y),
                f32::from(bounds.size.width),
                f32::from(bounds.size.height),
            )
        })
        .collect()
}

impl MarkionApp {
    pub(super) fn install_layout_persistence(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.capture_window_layout(window);
        let subscription = cx.observe_window_bounds(window, |app, window, cx| {
            app.capture_window_layout(window);
            app.schedule_persist_layout(cx);
        });
        self.layout_bounds_subscription = Some(subscription);
    }

    pub(super) fn capture_window_layout(&mut self, window: &Window) {
        self.apply_window_bounds(window.window_bounds());
    }

    pub(super) fn apply_window_bounds(&mut self, bounds: WindowBounds) {
        match bounds {
            WindowBounds::Fullscreen(_) => {}
            WindowBounds::Maximized(bounds) => {
                self.session.layout.maximized = true;
                write_layout_bounds(&mut self.session.layout, bounds);
            }
            WindowBounds::Windowed(bounds) => {
                self.session.layout.maximized = false;
                write_layout_bounds(&mut self.session.layout, bounds);
            }
        }
    }

    pub(super) fn sync_layout_panes(&mut self) {
        self.session.layout.sidebar_width = Some(self.sidebar_width);
        self.session.layout.editor_split_ratio = Some(self.editor_split_ratio);
    }

    pub(super) fn schedule_persist_layout(&mut self, cx: &mut Context<Self>) {
        self.layout_persist_generation = self.layout_persist_generation.wrapping_add(1);
        let generation = self.layout_persist_generation;
        cx.spawn(async move |this, cx| {
            Timer::after(LAYOUT_PERSIST_DEBOUNCE).await;
            let _ = this.update(cx, |app, _| {
                if app.layout_persist_generation != generation {
                    return;
                }
                app.flush_layout();
            });
        })
        .detach();
    }

    pub(super) fn flush_layout(&mut self) {
        self.sync_layout_panes();
        self.persist_session();
    }
}

fn write_layout_bounds(layout: &mut SessionLayout, bounds: Bounds<Pixels>) {
    layout.x = Some(f32::from(bounds.origin.x));
    layout.y = Some(f32::from(bounds.origin.y));
    layout.width = Some(f32::from(bounds.size.width));
    layout.height = Some(f32::from(bounds.size.height));
}
