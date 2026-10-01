## Context

`search_panel_view` (`src/app/root_view.rs`) renders the overlay as an `absolute` full-width strip at `top = 36 + tab band`, `left/right = 16 px`, with `justify_end` pushing a `max_w(680)` panel to the right edge. The panel is a child of the root view, so its position is in window coordinates. Single-line fields inside it already stop mouse-down propagation (and now drag-select text through window-level listeners); toolbar buttons act on mouse-up.

The workspace already uses GPUI drags for resize-style interactions (`DraggedSidebarHandle`, `DraggedEditorSplitHandle`, `DraggedTableColumnHandle`) with `on_drag` on the handle and `on_drag_move` on an ancestor.

## Goals / Non-Goals

**Goals:** one explicit grip; window-coordinate position that survives close/reopen; clamping that is a pure function of overlay size and viewport so it is unit-testable.

**Non-Goals:** dragging from arbitrary overlay background (too easy to trigger while aiming at small buttons); remembering the position per tab or per view mode.

## Decisions

### 1. Session state: `Option<Point<Pixels>>` on `MarkionApp`
`search_overlay_origin: Option<Point<Pixels>>` holds the panel's top-left in window coordinates. `None` means the default layout, which is rendered exactly as today, so an unmoved overlay is pixel-identical to the current one. It is not part of the preferences model, so nothing reaches `config.toml`. It is app-level rather than per-tab state, because the overlay itself is app-level (`search_visible`).

*Alternative:* store an offset relative to the default corner. Rejected: the default corner moves with the tab band height and window width, so an offset would make the overlay drift on resize and on tab count changes.

### 2. Drag via a grip with GPUI `on_drag` / `on_drag_move`
A small grip at the panel's leading edge calls `on_drag(DraggedSearchOverlay { grab_offset })`, where `grab_offset` is the pointer position minus the panel origin at drag start. The root view (already the ancestor of every overlay) handles `on_drag_move::<DraggedSearchOverlay>` and sets `origin = clamp(pointer − grab_offset)`. On the first move the panel's measured size is needed for clamping; it is captured from a paint-time `canvas` (the same bounds-capture pattern the fields use) into app state.

Double-click on the grip (`click_count == 2`) sets the origin back to `None`.

*Alternative:* hand-rolled mouse-down/move/up tracking like the field selection. Rejected: GPUI drags already deliver moves outside the element and end cleanly on release, and match the existing resize handles.

### 3. Clamping at render time as well as during the drag
`clamp_search_overlay_origin(origin, panel_size, viewport) -> Point<Pixels>` keeps the whole panel inside the viewport, with the same 16 px margin the default layout uses. The drag handler applies it, and `search_panel_view` applies it again every render against the current `window.viewport_size()`. A window shrink therefore pulls the overlay back without a resize listener. The stored origin is left untouched by render-time clamping, so a temporary shrink does not permanently move the overlay.

When moved, the panel renders at a fixed width, `min(680, viewport − 32)`. This is the default panel's maximum width, so moving does not reflow the controls.

### 4. Data flow and caching
Grip drag → `on_drag_move` → `search_overlay_origin` updated → `cx.notify()` → next frame repositions one absolutely positioned overlay. No document mutation, no `after_document_changed`, no version bump. Derived per-version caches (preview blocks, outline, stats, highlighting) are untouched, and workspace layout is unaffected because the overlay is absolute.

## Risks / Trade-offs

- [The grip adds width to an already dense toolbar] → A narrow (about 10 px) dotted grip replaces part of the leading padding, so the controls keep their size.
- [The panel size is unknown on the very first drag frame] → Fall back to the default panel size (680 × last known height). The render-time clamp corrects it on the next frame.
- [The overlay is moved over the tab bar or the status bar] → Allowed. Only the window bounds clamp it; the user chose the spot, and the close control stays reachable because the whole panel is inside the window.
