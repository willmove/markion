## Context

See `proposal.md` for the platform analysis. GPUI 0.2.2 (vendored) exposes a complete client-side-decoration contract: the platform window reports `Decorations::Client` when the app must paint its own title-bar affordances, `WindowControls` says which buttons the compositor honors, and `minimize_window` / `zoom_window` / `start_window_move` / `remove_window` are the actions behind them. The report is only meaningful if the app asks for the mode it wants: `Window::new` requests `WindowDecorations::Server` by default and the Wayland backend stores that request as the window's state without any compositor negotiation when no decoration manager exists. On Wayland the backend neither paints decorations nor consumes `WindowControlArea` hitboxes, so the application owns everything. Zed's own workspace titlebar takes exactly this app-level approach at this GPUI revision; newer upstream GPUI moved the painting into the backend, which is out of scope for the vendored tree.

Markion already has a natural CSD host: the in-window menu bar (28 px tall, top row of the root view). GNOME headerbars put window controls at the right edge of that row.

## Goals / Non-Goals

**Goals:**

- Make minimize/maximize/close reachable and the window movable on GNOME Wayland (Ubuntu 26.04) with zero behavior change on every platform that reports server-side decorations.
- Reuse the existing unsaved-changes close flow so the CSD close button is not a second, weaker close path.
- Keep the addition inside `src/app` + assets; no vendored GPUI edits, no new dependencies.

**Non-Goals:**

- Backend-painted Wayland decorations, shadows, translucent borders, or edge-resize handles (`set_client_inset`, `start_window_resize`). Pointer resize stays what it is today (compositor keyboard/tiling shortcuts).
- Right-click window menus (`show_window_menu`), touch gestures, per-compositor styling.
- Changing the macOS/Windows/X11 title bars.

## Decisions

### 1. Request `Client` on Wayland, then gate on `Decorations::Client`

`matches!(window.window_decorations(), Decorations::Client { .. })` decides per render whether the cluster and drag region exist. This is the API's own signal: false on macOS/Windows/X11 and on Wayland compositors that grant server-side decorations, true exactly where the app must paint. It also keeps test fixtures on the old layout, because the test platform reports `Decorations::Server`.

That signal is only correct if the window asks for client decorations first. `window_controls::requested_decorations()` returns `Some(WindowDecorations::Client)` when `gpui::guess_compositor() == "Wayland"` and `None` elsewhere, and `bootstrap.rs` passes it as `WindowOptions.window_decorations`. On GNOME the request is never negotiated, so the state stays `Client` and matches the undecorated window. On a compositor that implements the protocol the request is forwarded and its configure reply overwrites the state: `Server` keeps the native title bar and hides the cluster, `Client` shows it. The `guess_compositor` call is `#[cfg]`-gated to Linux/FreeBSD because the API only exists there; the render gate itself stays platform-agnostic. X11 must not receive the request — its `request_decorations` rewrites Motif hints and would strip the native title bar.

### 2. Buttons live at the right end of the menu bar; close gets a danger hover

Each button is a fixed 36×28 div (28 px = bar height) with a 12 px glyph: `minus` (new asset) for minimize, the existing `square` glyph for maximize, `copy` (two overlapping squares) for restore when `window.is_maximized()`, and the existing `x` for close. Min/max hover uses the same `surface_bg` the menu buttons use; close hovers to a fixed danger red (`0xc01c28`) with a white glyph, matching GNOME's close-button affordance. Buttons are tagged with `.window_control_area(Min/Max/Close)` — a no-op at this GPUI revision on every backend we ship, but the documented CSD contract, and forward-compatible if the backend starts consuming the hitboxes.

### 3. Close reuses the native-close decision, not `remove_window` directly

`request_window_close` mirrors `install_window_close_guard` in `bootstrap.rs`: when `allow_close` or nothing is dirty, capture window layout, flush persistence, and `remove_window()`; otherwise `begin_unsaved_exit(WindowClose)` runs the existing dirty-tab prompt (and the running-Git cancellation path inside `begin_unsaved_exit`). On Wayland the app cannot ask the compositor to close the toplevel, so app-level removal is the only close mechanism anyway; on X11 CSD the same code path is correct there too.

### 4. Drag is threshold-deferred so clicks survive

Calling `start_window_move()` on press hands the pointer to a compositor move grab, after which the client sees no release — a double-click-to-maximize implemented on click events would never fire. The drag region therefore records the press origin (`csd_drag: Option<Point<Pixels>>` on `MarkionApp`), starts the move only after the pointer travels ≥ 3 px, and clears the state on click. Single clicks stay no-ops, double clicks toggle `zoom_window()`, and small hand jitter during a click never moves the window.

### 5. Tooltips go through i18n like every other string

Four new `Msg` variants (minimize / maximize / restore / close) with translations for all seven languages, rendered through the same bubble component style as the search-bar tooltips.

## Risks / Trade-offs

- The cluster is inset by the menu bar's horizontal padding rather than flush against the window edge; the bar keeps its own visual language instead of faking a native headerbar.
- Window-control visibility depends on `WindowControls`, whose default is permissive (all true) — on a compositor that neither supports nor reports a control, the button renders and the request is ignored by the compositor. This matches GPUI's documented default and Zed's behavior.
- Mode flips (a compositor granting server-side decorations after startup) are picked up on the next render; no extra invalidation is added for a case GNOME never produces.
