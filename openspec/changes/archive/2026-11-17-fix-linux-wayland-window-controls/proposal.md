## Why

On GNOME Wayland sessions — Ubuntu's default, and the only session Ubuntu 26.04 offers — Markion's window opens with no minimize, maximize, or close buttons and cannot be moved with the mouse. GNOME's Mutter deliberately does not advertise `zxdg_decoration_manager_v1`, so no server-side title bar is drawn, and at the vendored 0.2.2 revision GPUI's Wayland backend paints no decorations itself: it expects the application to render window controls. Markion never renders them.

The backend does not, however, report that situation as client-side decorations. `Window::new` calls `request_decorations(window_decorations.unwrap_or(WindowDecorations::Server))`, and the Wayland implementation stores the requested mode as the window's state unconditionally, only forwarding it to the compositor when a decoration manager exists. With no manager present there is no negotiation, so `Window::window_decorations()` keeps reporting `Decorations::Server` on a window that has no decorations at all. Gating the controls on `Decorations::Client` alone therefore never fires on GNOME Wayland (verified live on Ubuntu 26.04.1, GNOME Shell 50.1: the protocol dump shows no `zxdg_decoration_manager_v1` global and the window reports `Server`). The window must also *request* client-side decorations so the reported state matches what the compositor actually does. X11 sessions (older Ubuntu releases) are unaffected because the window manager decorates the window server-side, which is why the gap surfaced as an "Ubuntu 26.04.1" bug.

## What Changes

- Request `WindowDecorations::Client` when opening the main window on a Wayland session, so `Window::window_decorations()` reports the mode the compositor actually applies. Compositors that implement `zxdg_decoration_manager_v1` answer with a configure event that overrides the request and keeps their native title bar; GNOME, which implements nothing, keeps the requested client mode. Other backends keep GPUI's default request.
- Render a client-side window-control cluster — minimize, maximize/restore, close — at the right end of the in-window menu bar whenever `Window::window_decorations()` reports `Decorations::Client`.
- Honor `Window::window_controls()`: a control is rendered only when the compositor reports it as supported (the API defaults to all-supported when the compositor is silent).
- Wire minimize to `Window::minimize_window()` and maximize/restore to `Window::zoom_window()`; close routes through the same dirty-document decision as the native title-bar close (clean or already-allowed windows close immediately with layout persisted; dirty ones open the existing unsaved-changes confirmation).
- Turn the menu bar's free space into a window drag region: a press that moves past a small threshold calls `Window::start_window_move()` (deferred so the compositor's move grab does not swallow click delivery), and a double-click toggles maximize.
- Add one lucide-style icon (`minus`) to the embedded icon set (maximize/restore/close reuse existing square/copy/x glyphs) and localize the three buttons' tooltips through `src/i18n.rs` for all seven languages.
- Non-goals: porting newer upstream GPUI's backend-painted Wayland decorations, shadow/resize-border negotiation (`set_client_inset`, edge resize handles), right-click window menus, and any behavior change on platforms that report server-side decorations (macOS, Windows, X11, and Wayland compositors that grant server-side decorations). No vendored GPUI source is modified.

## Capabilities

### New Capabilities

- None.

### Modified Capabilities

- `chrome-platform`: the in-window menu bar additionally hosts client-side window controls and a drag region whenever the platform reports client-side decorations.

## Impact

- Affected code: `src/app/bootstrap.rs` (main-window decoration request), `src/app/root_view.rs` (menu bar composition), new `src/app/window_controls.rs` (decoration request, controls cluster, drag region, close entry point), `src/app/documents.rs` (close routing next to `quit`), `src/app/mod.rs` (module wiring + drag-state field), `src/ui/icon.rs` + `assets/icons/ui/minus.svg` (one icon), `src/i18n.rs` (four tooltip strings across seven languages).
- Configuration: none.
- Dependencies and APIs: no dependency or vendored-GPUI changes; uses only already-public GPUI 0.2.2 APIs (`guess_compositor`, `WindowOptions.window_decorations`, `window_decorations`, `window_controls`, `minimize_window`, `zoom_window`, `start_window_move`, `remove_window`, `window_control_area`).
- Architecture invariants: a rendering-only addition to the menu bar. Document parsing, per-version derived-state caches, highlight memoization, and the cached text handle are untouched. Test windows report `Decorations::Server`, so existing GPUI test fixtures render the menu bar unchanged.
