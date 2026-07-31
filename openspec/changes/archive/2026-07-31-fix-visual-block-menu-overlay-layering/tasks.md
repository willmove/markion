## 1. Overlay State and Lifecycle

- [x] 1.1 Extend `BlockMenuState` with a transient window-space anchor, pass the invoking button position through `open_visual_block_menu`, and keep the existing immutable `BlockTarget` validation data unchanged.
- [x] 1.2 Centralize block-menu dismissal for Escape, outside pointer actions, document-pane scrolling, tab/view changes, document mutation, undo/redo, stale targets, and conflicting application overlays without changing canonical source or cached derived state.

## 2. Root Overlay Rendering

- [x] 2.1 Remove the menu panel from `visual_block_chrome` and render one block-menu view in the application root's contextual-overlay stratum after document content and before modal content, preserving all existing command callbacks and debug selectors.
- [x] 2.2 Anchor the panel with the existing GPUI `anchored()` pattern, retain opaque themed chrome and pointer occlusion, and add viewport-bounded height plus menu-local vertical scrolling so every command remains reachable.

## 3. Regression Evidence

- [x] 3.1 Add a rendered GPUI fixture whose first-row menu overlaps following headings, formatted prose, and image content; assert root-overlay composition, overlapping geometry, menu pointer precedence, exact command dispatch, and one-step undo.
- [x] 3.2 Add rendered coverage for bottom/right viewport anchoring and menu-local overflow scrolling, including reachability of the final command without moving the document viewport.
- [x] 3.3 Add lifecycle tests proving every dismissal path leaves Markdown text, document version, selection, history, dirty state, and the shared derived `Arc` identity unchanged.
- [x] 3.4 Repeat the reported multi-heading/inline-formatting/image runtime scenario and visually confirm that no document glyph or image paints over the open menu on Windows.

## 4. Verification

- [x] 4.1 Run `cargo fmt --check` and the focused Visual Edit block-menu GPUI tests.
- [x] 4.2 Run `cargo test --workspace` and resolve any regressions without weakening cache, exact-target, or one-undo invariants.
- [x] 4.3 Run `openspec validate fix-visual-block-menu-overlay-layering` and `pwsh ./scripts/check-quality.ps1`, confirming the change and complete repository quality gate pass.
