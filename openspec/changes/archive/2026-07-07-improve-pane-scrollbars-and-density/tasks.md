## 1. Scrollbar Behavior

- [x] 1.1 Inspect current GPUI scroll container behavior for `editor-scroll` and `preview-scroll` and identify the minimal style changes needed for visible, draggable right-side vertical scrollbars.
- [x] 1.2 Update the editor pane scroll container so large source documents expose a draggable vertical scrollbar while preserving the active tab's `editor_scroll` handle.
- [x] 1.3 Update the preview pane scroll container so large rendered documents expose a draggable vertical scrollbar while preserving the active tab's `preview_scroll` handle.
- [x] 1.4 Verify wheel/trackpad scrolling and scrollbar dragging update the same scroll state without mutating document text or derived Markdown caches.

## 2. Pane Density

- [x] 2.1 Reduce editor and preview pane outer padding/gaps to roughly 15% of the current spacing while retaining minimal readable inner padding.
- [x] 2.2 Reduce sidebar/main-content visible gaps consistently with the editor/preview pane density.
- [x] 2.3 Keep split and sidebar visible separators compact while preserving their existing draggable hit targets.
- [x] 2.4 Confirm Edit and Read modes still fill the remaining workspace after density changes.

## 3. Verification

- [x] 3.1 Add or update focused tests where practical for layout constants or scroll-state preservation helpers without introducing brittle pixel snapshots.
- [x] 3.2 Run `cargo test`.
- [x] 3.3 Run `openspec validate improve-pane-scrollbars-and-density`.
- [x] 3.4 Build and launch the app for manual verification with a large Markdown document.
