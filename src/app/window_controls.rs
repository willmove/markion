//! Client-side window controls for platforms whose compositor provides no
//! server-side decorations (GPUI reports `Decorations::Client` — GNOME
//! Wayland, as on Ubuntu 26.04). The vendored GPUI paints nothing in that
//! mode, so the in-window menu bar hosts the minimize/maximize/close cluster
//! and the drag region instead. Platforms reporting server-side decorations
//! (macOS, Windows, X11, or a Wayland compositor that grants them) and test
//! windows keep the old menu bar because the gate in `root_view.rs` stays
//! false there.

use super::root_view::SearchTooltip;
use super::*;
use crate::ui::icon::{Icon, icon};
use gpui::WindowControlArea;

/// Pointer travel in px before a press in the drag region is treated as a
/// window move. Below it the press still completes as a click, so
/// double-click maximize keeps working.
const CSD_DRAG_THRESHOLD: f32 = 3.0;

/// Edge length of a window-control glyph.
const CSD_CONTROL_ICON_SIZE: f32 = 12.0;

// The embedded icon set already carries the square (maximize), copy
// (restore: two overlapping squares) and x (close) glyphs; only minimize
// needed a new asset (`minus`).
const MAXIMIZE_GLYPH: Icon = Icon::Stop;
const RESTORE_GLYPH: Icon = Icon::Copy;

/// GNOME-style close affordance: neutral until hovered, then the danger red
/// the platform's own title bars use (`#c01c28`).
fn csd_close_hover_bg() -> Rgba {
    rgb(0xc01c28)
}

/// Whether a pointer that went down at `origin` has travelled far enough to
/// start a compositor move grab.
fn csd_drag_past_threshold(origin: Point<Pixels>, position: Point<Pixels>) -> bool {
    let dx = f32::from(position.x - origin.x);
    let dy = f32::from(position.y - origin.y);
    dx * dx + dy * dy >= CSD_DRAG_THRESHOLD * CSD_DRAG_THRESHOLD
}

/// Decorations to request when opening the main window.
///
/// GPUI defaults to `Server`, and its Wayland backend records that request as
/// the window's state even when the compositor never advertises
/// `zxdg_decoration_manager_v1` (GNOME/Mutter). The window then reports
/// `Decorations::Server` while nothing at all draws a title bar, which hides
/// the cluster below. Requesting `Client` makes the report match reality;
/// compositors that do implement the protocol answer with a configure event
/// that overrides the request, so they keep their native title bar.
pub(super) fn requested_decorations() -> Option<gpui::WindowDecorations> {
    #[cfg(any(target_os = "linux", target_os = "freebsd"))]
    if gpui::guess_compositor() == "Wayland" {
        return Some(gpui::WindowDecorations::Client);
    }
    None
}

impl MarkionApp {
    /// Drag region: remember where the press started. The move request is
    /// deferred to [`Self::on_csd_drag_move`] because a compositor move grab
    /// ends pointer delivery, which would swallow the release (and with it
    /// every click).
    pub(super) fn on_csd_drag_down(
        &mut self,
        event: &MouseDownEvent,
        _: &mut Window,
        _: &mut Context<Self>,
    ) {
        self.csd_drag = Some(event.position);
    }

    pub(super) fn on_csd_drag_move(
        &mut self,
        event: &MouseMoveEvent,
        window: &mut Window,
        _: &mut Context<Self>,
    ) {
        if self
            .csd_drag
            .is_some_and(|origin| csd_drag_past_threshold(origin, event.position))
        {
            self.csd_drag = None;
            window.start_window_move();
        }
    }

    pub(super) fn on_csd_drag_click(
        &mut self,
        event: &ClickEvent,
        window: &mut Window,
        _: &mut Context<Self>,
    ) {
        self.csd_drag = None;
        if event.click_count() >= 2 {
            window.zoom_window();
        }
    }
}

/// The menu bar's free space right of the menus: drags move the window,
/// double-clicks toggle maximize.
pub(super) fn csd_drag_region(cx: &mut Context<MarkionApp>) -> Stateful<Div> {
    div()
        .id("csd-drag-region")
        .flex_1()
        .min_w(px(16.))
        .h_full()
        .window_control_area(WindowControlArea::Drag)
        .on_mouse_down(MouseButton::Left, cx.listener(MarkionApp::on_csd_drag_down))
        .on_mouse_move(cx.listener(MarkionApp::on_csd_drag_move))
        .on_click(cx.listener(MarkionApp::on_csd_drag_click))
}

/// Minimize / maximize-restore / close cluster. Rendered only when the
/// platform reports the corresponding control as supported.
pub(super) fn csd_window_controls(
    app: &MarkionApp,
    window: &mut Window,
    cx: &mut Context<MarkionApp>,
) -> Div {
    let palette = app.palette();
    let language = app.language;
    let controls = window.window_controls();
    let maximized = window.is_maximized();

    div()
        .flex()
        .flex_none()
        .items_center()
        .when(controls.minimize, |row| {
            row.child(window_control_button(
                Icon::Minus,
                Msg::TooltipMinimizeWindow,
                WindowControlArea::Min,
                language,
                palette,
                cx.listener(|_, _, window, _| window.minimize_window()),
            ))
        })
        .when(controls.maximize, |row| {
            let (glyph, label) = if maximized {
                (RESTORE_GLYPH, Msg::TooltipRestoreWindow)
            } else {
                (MAXIMIZE_GLYPH, Msg::TooltipMaximizeWindow)
            };
            row.child(window_control_button(
                glyph,
                label,
                WindowControlArea::Max,
                language,
                palette,
                cx.listener(|_, _, window, _| window.zoom_window()),
            ))
        })
        .child(window_control_button(
            Icon::Close,
            Msg::TooltipCloseWindow,
            WindowControlArea::Close,
            language,
            palette,
            cx.listener(|app, _, window, cx| app.request_window_close(window, cx)),
        ))
}

fn window_control_button(
    glyph: Icon,
    label: Msg,
    area: WindowControlArea,
    language: Language,
    palette: ThemePalette,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> Stateful<Div> {
    let close = area == WindowControlArea::Close;
    let tooltip_label = SharedString::from(t(language, label));
    let id: SharedString = match area {
        WindowControlArea::Close => "csd-close",
        WindowControlArea::Max => "csd-maximize",
        WindowControlArea::Min => "csd-minimize",
        WindowControlArea::Drag => "csd-drag",
    }
    .into();
    div()
        .id(id)
        .w(px(36.))
        .h(px(28.))
        .flex()
        .flex_none()
        .items_center()
        .justify_center()
        .rounded_sm()
        .cursor_pointer()
        .when(close, |button| {
            button.hover(move |style| style.bg(csd_close_hover_bg()))
        })
        .when(!close, move |button| {
            button.hover(move |style| style.bg(palette.surface_bg))
        })
        .window_control_area(area)
        .tooltip(move |_, cx| {
            cx.new(|_| SearchTooltip {
                palette,
                label: tooltip_label.clone(),
            })
            .into()
        })
        .on_click(on_click)
        .child(icon(glyph, CSD_CONTROL_ICON_SIZE, palette.text))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drag_starts_only_past_the_threshold() {
        let origin = point(px(100.), px(50.));
        assert!(!csd_drag_past_threshold(origin, origin));
        assert!(!csd_drag_past_threshold(origin, point(px(102.), px(50.))));
        assert!(csd_drag_past_threshold(origin, point(px(103.), px(50.))));
        assert!(csd_drag_past_threshold(origin, point(px(100.), px(53.))));
        // Diagonal travel counts as Euclidean distance.
        assert!(csd_drag_past_threshold(origin, point(px(102.2), px(52.2))));
    }
}
