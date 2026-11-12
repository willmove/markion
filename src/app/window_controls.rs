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

/// GNOME/Adwaita window corner radius at 1x; GPUI's logical pixels scale it
/// with the display scale factor like GTK does.
const CSD_WINDOW_RADIUS: f32 = 12.0;

/// Transparent margin reserved around the visible window on CSD sessions.
/// Resize grips and the drop shadow live here, outside all content, the same
/// way GTK's CSD shadow margin doubles as the resize area.
const CSD_RESIZE_MARGIN: f32 = 10.0;

/// Edge length of the diagonal corner grips. Larger than the margin so corner
/// grabs are forgiving; they sit below the content layer in hit order.
const CSD_CORNER_GRIP: f32 = 18.0;

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

/// Corner radii of the window silhouette. Zero everywhere unless the platform
/// requires client-side decorations; maximized/fullscreen windows are square,
/// and each edge the compositor reports tiled keeps its two corners square,
/// matching GNOME. Every painted surface that touches an outer corner reads
/// this one decision so the silhouette cannot disagree with itself.
pub(super) fn csd_corner_radii(
    decorations: gpui::Decorations,
    maximized: bool,
    fullscreen: bool,
) -> gpui::Corners<Pixels> {
    let gpui::Decorations::Client { tiling } = decorations else {
        return gpui::Corners::all(px(0.));
    };
    let radius = px(if maximized || fullscreen {
        0.
    } else {
        CSD_WINDOW_RADIUS
    });
    let mut corners = gpui::Corners::all(radius);
    if tiling.top || tiling.left {
        corners.top_left = px(0.);
    }
    if tiling.top || tiling.right {
        corners.top_right = px(0.);
    }
    if tiling.bottom || tiling.left {
        corners.bottom_left = px(0.);
    }
    if tiling.bottom || tiling.right {
        corners.bottom_right = px(0.);
    }
    corners
}

/// Per-edge transparent margin around the visible window. Zero unless the
/// platform requires client-side decorations; maximized/fullscreen windows
/// fill their geometry, and tiled edges have no margin (GPUI's Wayland
/// geometry insets per non-tiled edge to match).
pub(super) fn csd_margin(
    decorations: gpui::Decorations,
    maximized: bool,
    fullscreen: bool,
) -> gpui::Edges<Pixels> {
    let gpui::Decorations::Client { tiling } = decorations else {
        return gpui::Edges::all(px(0.));
    };
    let margin = px(if maximized || fullscreen {
        0.
    } else {
        CSD_RESIZE_MARGIN
    });
    gpui::Edges {
        top: if tiling.top { px(0.) } else { margin },
        right: if tiling.right { px(0.) } else { margin },
        bottom: if tiling.bottom { px(0.) } else { margin },
        left: if tiling.left { px(0.) } else { margin },
    }
}

/// The uniform value for `Window::set_client_inset`; the Wayland backend
/// applies it per non-tiled edge itself.
pub(super) fn csd_client_inset(
    decorations: gpui::Decorations,
    maximized: bool,
    fullscreen: bool,
) -> Pixels {
    px(
        if matches!(decorations, gpui::Decorations::Client { .. }) && !maximized && !fullscreen {
            CSD_RESIZE_MARGIN
        } else {
            0.
        },
    )
}

/// Viewport size minus the CSD margin: the space the visible chrome occupies.
pub(super) fn content_viewport_size(window: &Window) -> gpui::Size<Pixels> {
    let margin = csd_margin(
        window.window_decorations(),
        window.is_maximized(),
        window.is_fullscreen(),
    );
    let viewport = window.viewport_size();
    gpui::size(
        viewport.width - margin.left - margin.right,
        viewport.height - margin.top - margin.bottom,
    )
}

/// Resize grips in the transparent margin: four edge bands plus four corner
/// squares, present only for edges the margin reserves. Rendered below the
/// content layer so corner overlap can never steal chrome-button clicks; the
/// edge bands lie wholly outside the content rectangle.
pub(super) fn csd_resize_grips(
    cx: &mut Context<MarkionApp>,
    margin: gpui::Edges<Pixels>,
) -> Vec<Stateful<Div>> {
    let zero = px(0.);
    let corner = px(CSD_CORNER_GRIP);
    let mut grips = Vec::new();
    if margin.top > zero {
        grips.push(resize_grip(
            "csd-resize-top",
            cx,
            gpui::ResizeEdge::Top,
            |g| {
                g.cursor_ns_resize()
                    .top_0()
                    .left_0()
                    .right_0()
                    .h(margin.top)
            },
        ));
    }
    if margin.bottom > zero {
        grips.push(resize_grip(
            "csd-resize-bottom",
            cx,
            gpui::ResizeEdge::Bottom,
            |g| {
                g.cursor_ns_resize()
                    .bottom_0()
                    .left_0()
                    .right_0()
                    .h(margin.bottom)
            },
        ));
    }
    if margin.left > zero {
        grips.push(resize_grip(
            "csd-resize-left",
            cx,
            gpui::ResizeEdge::Left,
            |g| {
                g.cursor_ew_resize()
                    .left_0()
                    .top_0()
                    .bottom_0()
                    .w(margin.left)
            },
        ));
    }
    if margin.right > zero {
        grips.push(resize_grip(
            "csd-resize-right",
            cx,
            gpui::ResizeEdge::Right,
            |g| {
                g.cursor_ew_resize()
                    .right_0()
                    .top_0()
                    .bottom_0()
                    .w(margin.right)
            },
        ));
    }
    if margin.top > zero && margin.left > zero {
        grips.push(resize_grip(
            "csd-resize-top-left",
            cx,
            gpui::ResizeEdge::TopLeft,
            |g| g.cursor_nwse_resize().top_0().left_0().w(corner).h(corner),
        ));
    }
    if margin.top > zero && margin.right > zero {
        grips.push(resize_grip(
            "csd-resize-top-right",
            cx,
            gpui::ResizeEdge::TopRight,
            |g| g.cursor_nesw_resize().top_0().right_0().w(corner).h(corner),
        ));
    }
    if margin.bottom > zero && margin.left > zero {
        grips.push(resize_grip(
            "csd-resize-bottom-left",
            cx,
            gpui::ResizeEdge::BottomLeft,
            |g| {
                g.cursor_nesw_resize()
                    .bottom_0()
                    .left_0()
                    .w(corner)
                    .h(corner)
            },
        ));
    }
    if margin.bottom > zero && margin.right > zero {
        grips.push(resize_grip(
            "csd-resize-bottom-right",
            cx,
            gpui::ResizeEdge::BottomRight,
            |g| {
                g.cursor_nwse_resize()
                    .bottom_0()
                    .right_0()
                    .w(corner)
                    .h(corner)
            },
        ));
    }
    grips
}

fn resize_grip(
    id: &'static str,
    cx: &mut Context<MarkionApp>,
    edge: gpui::ResizeEdge,
    place: impl FnOnce(Stateful<Div>) -> Stateful<Div>,
) -> Stateful<Div> {
    // The compositor resize grab needs the current mouse-press serial, so the
    // request must happen here, on press, not deferred to a move.
    place(div().id(id).absolute().on_mouse_down(
        MouseButton::Left,
        cx.listener(move |_, _, window, _| {
            window.start_window_resize(edge);
        }),
    ))
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
    top_right_radius: Pixels,
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
                px(0.),
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
                px(0.),
                cx.listener(|_, _, window, _| window.zoom_window()),
            ))
        })
        .child(window_control_button(
            Icon::Close,
            Msg::TooltipCloseWindow,
            WindowControlArea::Close,
            language,
            palette,
            top_right_radius,
            cx.listener(|app, _, window, cx| app.request_window_close(window, cx)),
        ))
}

fn window_control_button(
    glyph: Icon,
    label: Msg,
    area: WindowControlArea,
    language: Language,
    palette: ThemePalette,
    corner_radius: Pixels,
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
        // Flush against the window's top-right corner: the hover background
        // must follow the window silhouette, not spill outside it.
        .when(close, |button| button.rounded_tr(corner_radius))
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

    #[test]
    fn corner_radii_follow_window_state() {
        let client = gpui::Decorations::Client {
            tiling: gpui::Tiling::default(),
        };
        let round = gpui::Corners::all(px(CSD_WINDOW_RADIUS));
        let square = gpui::Corners::all(px(0.));
        assert_eq!(csd_corner_radii(client, false, false), round);
        assert_eq!(csd_corner_radii(client, true, false), square);
        assert_eq!(csd_corner_radii(client, false, true), square);
        assert_eq!(
            csd_corner_radii(gpui::Decorations::Server, false, false),
            square
        );

        let left_tiled = gpui::Decorations::Client {
            tiling: gpui::Tiling {
                left: true,
                ..Default::default()
            },
        };
        let corners = csd_corner_radii(left_tiled, false, false);
        assert_eq!(corners.top_left, px(0.));
        assert_eq!(corners.bottom_left, px(0.));
        assert_eq!(corners.top_right, px(CSD_WINDOW_RADIUS));
        assert_eq!(corners.bottom_right, px(CSD_WINDOW_RADIUS));
    }

    #[test]
    fn resize_margin_follows_window_state() {
        let client = gpui::Decorations::Client {
            tiling: gpui::Tiling::default(),
        };
        let full = gpui::Edges::all(px(CSD_RESIZE_MARGIN));
        let zero = gpui::Edges::all(px(0.));
        assert_eq!(csd_margin(client, false, false), full);
        assert_eq!(
            csd_client_inset(client, false, false),
            px(CSD_RESIZE_MARGIN)
        );
        assert_eq!(csd_margin(client, true, false), zero);
        assert_eq!(csd_margin(client, false, true), zero);
        assert_eq!(csd_client_inset(client, true, false), px(0.));
        assert_eq!(csd_margin(gpui::Decorations::Server, false, false), zero);
        assert_eq!(
            csd_client_inset(gpui::Decorations::Server, false, false),
            px(0.)
        );

        let tiled = gpui::Decorations::Client {
            tiling: gpui::Tiling {
                top: true,
                left: true,
                ..Default::default()
            },
        };
        let margin = csd_margin(tiled, false, false);
        assert_eq!(margin.top, px(0.));
        assert_eq!(margin.left, px(0.));
        assert_eq!(margin.right, px(CSD_RESIZE_MARGIN));
        assert_eq!(margin.bottom, px(CSD_RESIZE_MARGIN));
        // The inset stays uniform; the backend applies it per non-tiled edge.
        assert_eq!(csd_client_inset(tiled, false, false), px(CSD_RESIZE_MARGIN));
    }
}
