## 1. File Tree Icon Rendering

- [x] 1.1 Add small GPUI helper(s) in `src/main.rs` to render folder and Markdown-file icons using existing theme/palette colors and no new dependencies.
- [x] 1.2 Replace the file-tree row `dir` / `md` text marker with a compact flex row containing the icon and entry name.
- [x] 1.3 Preserve existing indentation, active/selected styling, hover behavior, row click target, and bounded `MAX_VISIBLE_TREE_ENTRIES` rendering.

## 2. Verification

- [x] 2.1 Run `cargo fmt`.
- [x] 2.2 Run `cargo test`.
- [ ] 2.3 Verify the Files sidebar visually with at least one folder and one Markdown file, confirming rows show icons instead of `dir` / `md` and open/select behavior is unchanged.

## 3. Requested Refinements

- [x] 3.1 Extend the Files panel state to track collapsed directories and toggle folder rows on click.
- [x] 3.2 Hide descendants of collapsed folders while preserving filtering and the visible-row cap.
- [x] 3.3 Redraw folder icons to better match conventional editor folder shapes, with distinct expanded and collapsed states.
- [x] 3.4 Keep file and folder names on one line and make the file-tree list horizontally scrollable for long rows.
- [x] 3.5 Tighten file-tree row spacing.
- [x] 3.6 Run `cargo fmt`, `cargo test`, and `openspec validate use-icons-in-file-tree`.
