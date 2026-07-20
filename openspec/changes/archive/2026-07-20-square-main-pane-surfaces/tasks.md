## 1. Main Pane Styling

- [x] 1.1 Remove the corner-radius styling from the source editor and visual editor surfaces in `src/app/root_view.rs` while preserving their background fill, border, padding, scrolling, and input handlers.
- [x] 1.2 Remove the corner-radius styling from the rendered preview surface in `src/app/root_view.rs` while preserving its background fill, border, padding, scrolling, context menu, and drag-and-drop behavior.

## 2. Verification

- [x] 2.1 Run `cargo fmt --check` and `cargo test` to confirm the styling-only change builds cleanly and preserves existing behavior.
- [x] 2.2 Build and launch Markion, then verify square corners in Edit, Visual Edit, Split Preview, and Read modes under representative light and dark themes.
- [x] 2.3 Run `openspec validate square-main-pane-surfaces` after implementation and resolve any reported issues.

Verification note: the current worktree's full test build is blocked by an unrelated, incomplete Mermaid integration that references a missing `src/app/diagram.rs`. An isolated `HEAD` plus this change successfully compiled the Markion binary test target and executable; its library suite passed 140 of 142 tests, with the two failures confined to pre-existing HTML-preview expectations. `cargo fmt --check` passes in the current worktree, and `src/app/root_view.rs` passes standalone `rustfmt --check` in the isolated tree. Manual light/dark checks confirmed square primary surfaces in all four view modes.
