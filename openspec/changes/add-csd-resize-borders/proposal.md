## Why

On GNOME Wayland the compositor provides no server-side decorations, so it also provides no edge-resize affordances: with client-side decorations the client owns resizing. Markion's CSD window (see `fix-linux-wayland-window-controls` and `round-csd-window-corners`) therefore cannot be resized with the mouse at all — hovering the right or bottom edge shows no resize cursor and dragging does nothing. Native GTK4/libadwaita windows solve this with a transparent margin around the visible window that doubles as shadow space and resize grip area.

## What Changes

- Reserve a transparent client margin around the visible window on CSD sessions via `Window::set_client_inset` (10 logical px per non-tiled edge; zero when maximized/fullscreen, and zero on tiled edges, which GPUI's Wayland geometry already honors). The visible chrome is laid out inside that margin.
- Paint resize grips in the margin: four edge bands (10 px) and four 18 px corner squares, each setting the matching resize cursor (`ew-resize`, `ns-resize`, `nesw-resize`, `nwse-resize`) and calling `Window::start_window_resize(edge)` on left press. Grips are painted below the content layer so any overlap at the corners can never steal clicks from chrome buttons; the edge bands lie wholly outside the content, so scrollbars and row hit areas are untouched.
- Use the freed margin for a soft drop shadow under the rounded silhouette, completing the native GNOME window look that `round-csd-window-corners` deferred.
- Content-size consumers (`viewport_size()` users in the root view and AI panel) read the margin-aware content size so overlays and pane widths stay correct inside the margin.
- Non-goals: touch/pinch resize, keyboard resize shortcuts, resize on server-side-decorated platforms (the gate stays false there and the margin is zero), and changing the saved window bounds semantics (GPUI reports geometry, which excludes the margin).

## Capabilities

### New Capabilities

- None.

### Modified Capabilities

- `chrome-platform`: the client-side-decorated window gains mouse edge/corner resizing with native resize cursors, a transparent resize/shadow margin, and a drop shadow.

## Impact

- Affected code: `src/app/window_controls.rs` (margin decision, grips, content-size helper), `src/app/root_view.rs` (outer/inner split, inset lifecycle, shadow, content-size consumers), `src/app/ai_panel.rs` (content-size consumers), `src/app/tests.rs` (margin/grip unit tests + source-contract assertions).
- Configuration: none.
- Dependencies and APIs: no dependency or vendored-GPUI changes; uses already-public GPUI 0.2.2 APIs (`set_client_inset`, `start_window_resize`, `ResizeEdge`, cursor style helpers, `shadow`).
- Architecture invariants: chrome-only change; derived-state caches, highlight memoization, and the cached text handle are untouched. Test windows report `Decorations::Server`, so the margin is zero and fixtures render unchanged.