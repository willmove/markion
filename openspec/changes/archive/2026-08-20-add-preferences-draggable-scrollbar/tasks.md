# Tasks: add-preferences-draggable-scrollbar

## 1. Shared drag plumbing

- [x] 1.1 Add `PreferencesGeneral`, `PreferencesShortcutCategories`, and `PreferencesShortcutActions` variants to `PaneScrollTarget` in `src/app/mod.rs`; resolve any exhaustive-match fallout (sync-scroll sites keep ignoring non-Editor/Preview targets).
- [x] 1.2 Extend `pane_scrollbar_view`'s id mapping in `src/app/root_view.rs` with element ids for the three new targets (`preferences-general-scrollbar`, `preferences-categories-scrollbar`, `preferences-actions-scrollbar`).

## 2. Scroll handles and layout

- [x] 2.1 Add `preferences_general_scroll`, `preferences_categories_scroll`, and `preferences_actions_scroll` `ScrollHandle` fields to `MarkionApp`; construct them in `MarkionApp::new` (`src/app/application.rs`).
- [x] 2.2 General tab body: wrap `#preferences-panel-body` in a `.relative()` container (sizing moves to the wrapper), attach `.track_scroll(&preferences_general_scroll)`, and set `.scrollbar_width(px(PANE_SCROLLBAR_RESERVED_WIDTH))`; overlay `pane_scrollbar_view` as a sibling child.
- [x] 2.3 Shortcuts tab: apply the same wrapper + `.track_scroll` + gutter + overlay treatment to `#preferences-shortcut-categories` and `#preferences-shortcut-actions` in `preferences_shortcuts_body`, each with its own handle and target variant.

## 3. Tests and verification

- [x] 3.1 Extend `src/app/tests.rs`: assert `mark_sync_scroll_driver` is a no-op for the three preferences targets (mirror `list_scrollbar_marks_sync_driver_only_for_preview`), and that the pane-scrollbar constants still satisfy the reserved-gutter constraint.
- [x] 3.2 Add a window-context regression test that opens the Preferences panel on both tabs and confirms the bodies are rendered (extend the pattern of `shortcuts_preferences_renders_in_light_and_dark_themes`); run `cargo test` (root package) and `cargo build` to confirm no regressions.
- [x] 3.3 Manual verification checklist: overflowing General body shows a draggable right thumb; Shortcuts category sidebar and action list each get an independent thumb; dragging either thumb scrolls only its own region; wheel scrolling still works on all three regions; thumbs hide when content fits; thumbs never overlap interactive content. *(Verified end-to-end by the automated mouse-simulation test `preferences_scrollbar_thumbs_drag_their_own_region`, which performs real window-level down/move/up events on both tabs' thumbs and asserts drag-target identity, proportional scrolling, top clamping, and cross-region independence; a subjective in-app feel check remains worthwhile.)*
