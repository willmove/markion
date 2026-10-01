## 1. Shared drag plumbing

- [x] 1.1 Add `FileTree` and `Outline` variants to `PaneScrollTarget` in `src/app/mod.rs`. Update exhaustive matches so `mark_sync_scroll_driver` stays a no-op for them and `reconcile_sync_scroll` ignores them next to Visual/Preferences.
- [x] 1.2 Extend `pane_scrollbar_view`'s id mapping in `src/app/root_view.rs` with `file-tree-scrollbar` and `outline-scrollbar`.

## 2. Sidebar overlay layout

- [x] 2.1 Files panel: wrap `#file-tree-scroll` in a `.relative().flex_1().min_h_0()` container (keep the workspace heading and name-editor fallback outside it), set `.scrollbar_width(px(PANE_SCROLLBAR_RESERVED_WIDTH))`, keep `overflow_x_scroll()` and `.track_scroll(&file_tree_scroll)`, and overlay `pane_scrollbar_view(FileTree, …)` as a sibling.
- [x] 2.2 Include `PANE_SCROLLBAR_RESERVED_WIDTH` in `file_tree_content_width` so long names remain reachable by horizontal scroll instead of sitting under the thumb.
- [x] 2.3 Outline panel: wrap `#outline-scroll` the same way, overlay `pane_scrollbar_view(Outline, …)`, and leave the image-tab placeholder without a scrollbar.

## 3. Tests and verification

- [x] 3.1 Extend `src/app/tests.rs` so `FileTree` and `Outline` are covered by the sync-driver no-op assertions (mirror `preferences_scrollbar_targets_are_no_op_for_sync_scroll` / `list_scrollbar_marks_sync_driver_only_for_preview`).
- [x] 3.2 Add a window-context test that shows overflowing Files and Outline lists, asserts a visible thumb, simulates left-button drag on each thumb, and checks proportional scroll plus independent `pane_scrollbar_drag` identity (follow `preferences_scrollbar_thumbs_drag_their_own_region`). Assert fitting/empty lists hide the thumb. Do not recompute derived Markdown state in these tests beyond existing outline cache hits.
- [x] 3.3 Run `cargo test` (root package) and `cargo build`. Manual checklist: overflowing Files thumb drags the tree; overflowing Outline thumb drags headings; both hide when content fits; wheel/trackpad still work; sidebar resize handle still works; long file names remain horizontally reachable; Sync scroll is unaffected. *(Verified by `sidebar_scrollbar_thumbs_drag_their_own_region` for Files/Outline drag identity, proportional scrolling, hide-when-fits, and Sync-scroll no-op; `long_partially_folded_outline_remains_scrollable` still passes for wheel input. A quick in-app check of the sidebar resize handle and long-name horizontal overflow remains worthwhile.)*
