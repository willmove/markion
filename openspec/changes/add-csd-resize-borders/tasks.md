## 1. Margin decision and helpers

- [x] 1.1 Add `window_controls::csd_margin(decorations, maximized, fullscreen) -> Edges<Pixels>` (10 px per non-tiled CSD edge, zero otherwise) and `content_viewport_size(window)`
- [x] 1.2 Unit-test the margin: server-decorated, floating CSD, maximized, fullscreen, and per-edge tiled cases

## 2. Resize grips and margin layout

- [x] 2.1 Split the root view into an unpainted outer container and an inner content container inset by the margin; move background, radii, and all chrome children into the inner container; call `window.set_client_inset` with the uniform margin each render
- [x] 2.2 Add the four edge bands and four corner squares as the outer container's first children, each with its resize cursor and `start_window_resize` on left press, present only for reserved edges
- [x] 2.3 Add the drop shadow to the inner container so the margin reads as native window shadow
- [x] 2.4 Switch production `viewport_size()` consumers (root view overlay math, AI panel width clamps and pointer-distance math) to the margin-aware helpers

## 3. Verification

- [x] 3.1 `cargo fmt --check` on touched files (no diffs), `cargo build`, `cargo test --bin markion` pass: 774 passed / 0 failed (lib target's pre-existing `ai_actions` failure unchanged)
- [x] 3.2 Extend the CSD source-contract test to pin the inset call, the grip wiring, and the margin-aware content size
- [x] 3.3 Verify live on GNOME Wayland (Ubuntu 26.04.1): screenshot shows the transparent margin with the drop shadow around the rounded window, content and scrollbar inset inside it, and the underlying desktop visible through the margin
- [ ] 3.4 User exercises the resize cursors and drag-resize on the live session (no input-injection tooling in this environment, so hover/drag behavior is verified by the user)