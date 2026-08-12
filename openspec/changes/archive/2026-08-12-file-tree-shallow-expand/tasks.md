## 1. Pure toggle helper + unit tests

- [x] 1.1 Add `pub(super) fn toggle_tree_folder(folder: &Path, tree: &FileTree, collapsed_paths: &mut HashSet<PathBuf>)` to `src/app/state.rs`, next to `update_file_tree_collapse_state_from_scan`. Semantics: if `folder` is in the set (collapsed) → expand by removing it and then inserting every descendant **directory** from `tree.entries` (`entry.kind == FileTreeEntryKind::Directory && entry.path != folder && entry.path.starts_with(folder)`); otherwise (expanded) → collapse by inserting `folder`. Import `FileTree` / `FileTreeEntryKind` as already done in that module.
- [x] 1.2 Re-export `toggle_tree_folder` through `src/app/mod.rs` (or the existing `state` re-export path) so `root_view.rs` can call it, matching how `update_file_tree_collapse_state_from_scan` is surfaced.
- [x] 1.3 Add unit tests in `src/app/tests.rs` reusing `nested_file_tree_fixture` and `visible_tree_entry_names`, covering: (a) expanding a collapsed folder reveals only its immediate children while deeper subfolders stay collapsed; (b) clicking the now-visible collapsed subfolder reveals only its own immediate children (progressive drill-down); (c) collapsing an expanded folder hides its entire subtree; (d) expanding a folder that contains only direct Markdown files shows those files and no deeper structure.

## 2. Wire the click handler to the helper

- [x] 2.1 In `src/app/root_view.rs`, replace the inline toggle in the `FileTreeEntryKind::Directory` arm of the `on_mouse_up(MouseButton::Left)` closure (around lines 1656-1665) with a call to `toggle_tree_folder(&path, &app.file_tree, &mut app.collapsed_tree_paths)`. Leave `app.selected_tree_path`, the status message, and `cx.notify()` unchanged.
- [x] 2.2 Confirm the call compiles: `&app.file_tree` and `&mut app.collapsed_tree_paths` are distinct fields of `MarkionApp`, so Rust's split-borrow rule permits borrowing both simultaneously. Adjust only if the borrow checker rejects it (e.g. clone the folder path first, which the closure already does at `let path = path.clone();`).

## 3. Verification

- [x] 3.1 Run `cargo test` for the root package: the new toggle tests pass and the existing file-tree tests still pass — in particular `initial_file_tree_collapse_shows_root_children_and_expands_one_branch` remains valid because it exercises the unchanged visibility filter directly, not the new toggle helper.
- [x] 3.2 Run `cargo build` to confirm no regressions, then manually verify in the app that clicking a folder expands exactly one level, deeper folders stay collapsed until clicked, and collapsing a folder hides its whole subtree.
- [x] 3.3 Run `openspec validate file-tree-shallow-expand` and confirm it passes.
