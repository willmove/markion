## Context

Mutter grants no server-side decorations on Wayland, and with them no server-side resize: `xdg_toplevel.resize` must be requested by the client, and the client must also provide the hover affordance. GPUI 0.2.2 exposes both halves — `Window::start_window_resize(ResizeEdge)` (which uses the current mouse-press serial, so it must be called from a press handler) and `Window::set_client_inset`, whose Wayland implementation insets `xdg_surface.set_window_geometry` per non-tiled edge (`inset_by_tiling`) and adds the inset back when computing the render surface size (`compute_outer_size`). The surface therefore extends beyond the visible window by the inset, exactly like GTK's CSD shadow margin.

Two constraints from the previous CSD changes carry over: content masks are rectangular, and the corner zone of the rounded silhouette is transparent — both make the margin a natural, conflict-free home for resize grips.

## Goals / Non-Goals

**Goals:**

- Mouse resizing from all four edges and corners on CSD sessions, with the platform-native resize cursors, without stealing a single pixel of interactive content (scrollbars, tree rows, chrome buttons).
- Margin lifecycle that matches the compositor: zero when maximized/fullscreen, zero on tiled edges, uniform elsewhere.
- Keep every existing platform and test fixture byte-identical (margin zero, grips absent).

**Non-Goals:**

- Touch gestures, keyboard resize, animated margin transitions.
- Changing saved bounds semantics: GPUI's `window_bounds()` on Wayland is the geometry, which already excludes the margin, so session restore is unaffected.
- Resize grips inside the content area (the alternative considered and rejected: an inside band would cover the preview scrollbar's drag area and the left edge of file-tree rows).

## Decisions

### 1. Margin, not inside bands

`window_controls::csd_margin(decorations, maximized, fullscreen) -> Edges<Pixels>` returns 10 px per edge for `Decorations::Client`, zeroed for maximized/fullscreen and per tiled edge; all zero for `Decorations::Server`. The render loop calls `window.set_client_inset(uniform)` (GPUI applies it per non-tiled edge itself) and lays the visible chrome out inside an inner container inset by those edges. The outer container stays unpainted, so the margin is transparent.

### 2. Grips below the content, edges wholly in the margin

Grips are the outer container's first children, before the inner content container, so content hitboxes win any overlap. Edge bands are exactly the margin thickness and lie outside the content rectangle; corner squares are 18 px at the surface corners, overlapping only the margin plus the transparent rounded-corner zone — and even there the close button and menu buttons sit above them. Each grip sets its cursor (`cursor_ew_resize`, `cursor_ns_resize`, `cursor_nesw_resize`, `cursor_nwse_resize`) and starts the matching `ResizeEdge` on left mouse-down. A grip exists only for edges the margin actually reserves.

### 3. Content size is margin-aware

`window_controls::content_viewport_size(window)` subtracts the margin from `viewport_size()`. The root view's overlay height/width math and the AI panel's width clamps use it; the AI panel's pointer-distance math subtracts only the right margin, because pointer coordinates are surface-relative. Test code keeps raw `viewport_size()` (the test platform's margin is zero).

### 4. Shadow rides in the margin

The inner container gains one `BoxShadow` (black at low alpha, small blur and downward offset) that spreads into the margin, giving the rounded window the edge definition native GTK windows have. It is painted by the inner container and clipped only by the surface, so it never touches content.

### 5. Inset lifecycle in render, guarded by GPUI

`set_client_inset` is called from render with the current decision; the Wayland backend no-ops when the value is unchanged, so this cannot loop with configure events. Maximize/fullscreen/tiling changes arrive as configure events that invalidate the window, so the next render updates both the inset and the grips.

## Risks / Trade-offs

- 10 px edge bands are thinner than GTK's shadow-margin grips but match the reserved margin; 18 px corners compensate for diagonal grabs. Users coming from GTK may find edges slightly less forgiving.
- The margin makes the surface 20 px larger per axis; every `viewport_size()` consumer must be margin-aware or overlays mis-size. The change audits all production consumers (ten sites) and pins the helper with a contract test.
- Without a compositor that honors `set_client_inset` (non-Wayland backends ignore it), the margin decision is zero anyway because the gate is false, so no platform sees a margin it cannot render.