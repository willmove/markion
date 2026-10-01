## 1. Position model

- [x] 1.1 Add `search_overlay_origin: Option<Point<Pixels>>` and the last measured panel size to `MarkionApp`, plus a pure `clamp_search_overlay_origin(origin, panel_size, viewport)` that keeps the panel inside the viewport with a 16 px margin; add unit tests for each edge, a panel larger than the viewport, and an in-bounds origin left unchanged; verify with `cargo test --bin markion search_overlay_origin`

## 2. Grip and rendering

- [x] 2.1 In `search_panel_view`, render a localized-tooltip grip at the panel's leading edge with `on_drag(DraggedSearchOverlay { grab_offset })`, handle `on_drag_move::<DraggedSearchOverlay>` on the root view to store the clamped origin, reset to `None` on grip double-click, and render the moved panel at the clamped origin with width `min(680, viewport − 32)` (unmoved rendering unchanged); add the grip tooltip to `src/i18n.rs` for every UI language; verify with `cargo build` and the i18n completeness tests
- [x] 2.2 Add app tests: drag the grip and assert the panel moves while `tab bar`/editor bounds, document text, version, dirty state, and undo history are unchanged; a drag on the find field selects text and does not move the panel; close + reopen keeps the position; shrinking the window keeps the panel inside; double-click resets to the default bounds; verify with `cargo test --bin markion search_overlay`

## 3. Integration

- [x] 3.1 Run `cargo fmt --check`, `cargo test --bin markion`, `cargo test --lib`, and confirm no new clippy lints on changed lines; validate with `openspec validate movable-find-replace-overlay --strict`
