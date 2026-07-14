## 1. External drop handler

- [x] 1.1 In `src/main.rs`, add `use gpui::ExternalPaths;` to the existing gpui import block (or a dedicated `use` if the crate imports gpui types individually). Confirm `ExternalPaths` and its `paths()` accessor are reachable from the `gpui = "0.2.2"` dependency.
- [x] 1.2 Add a `Markdown_EXTENSIONS`-style helper or inline check: a `fn is_markdown_path(path: &Path) -> bool` that returns true when `path.extension()` is one of `md` / `markdown` / `mdown` (case-insensitive on Windows). Reuse the same set the file tree uses (`src/storage/file_tree.rs:~287`); do not duplicate the literal set silently - if a shared constant already exists, reuse it, otherwise extract one.
- [x] 1.3 Add `fn handle_external_drop(&mut self, dragged: &ExternalPaths, _window: &mut Window, cx: &mut Context<Self>)` on `MarkionApp`. It iterates `dragged.paths()`; for each path where `is_markdown_path` is true and `path.is_file()`, it calls `self.open_file_in_new_tab_from_path(path.clone(), cx)`. After the loop, the status bar is already set by the last `open_file_in_new_tab_from_path` call (`StatusOpened` with that path); if no file was opened, leave the status untouched (no-op). Directories and non-Markdown files are skipped silently.

## 2. Register the drop target on both panes

- [x] 2.1 On the editor pane `div` (~`src/main.rs:5103`, the one sized by `editor_width` and hidden in `ViewMode::Read`), add `.on_drop::<ExternalPaths>(cx.listener(Self::handle_external_drop))`. Place it alongside the existing `.on_drop::<DraggedEditorSplitHandle>` etc. so the pane accepts both internal divider drags and external OS file drops.
- [x] 2.2 On the preview pane `div` (~`src/main.rs:5170`, the one sized by `preview_width`), add the same `.on_drop::<ExternalPaths>(cx.listener(Self::handle_external_drop))`.
- [x] 2.3 Do NOT add the handler to `main-content-row` (~`src/main.rs:5080`) or to the sidebar/file-tree views - dropping onto the tree or the chrome must not open a file.

## 3. Spec delta

- [x] 3.1 Add the delta requirement "Open files via drag-and-drop from the OS" to `openspec/changes/add-drag-drop-open/specs/workspace/spec.md`, with scenarios for: dropping one Markdown file opens it in a new tab; dropping multiple Markdown files opens each in its own tab with focus on the last; dropping only non-Markdown files (or directories) does nothing and leaves the status bar unchanged; the sidebar/file-tree area is not a drop target.

## 4. Tests and verification

- [x] 4.1 Add a unit test for `is_markdown_path` (or the shared helper from 1.2) covering `.md`, `.markdown`, `.mdown`, case variants (`MD`, `Md`), no extension, `.txt`, and a directory path (returns false for the extension check; the `is_file` guard in the handler handles directories at call time).
- [x] 4.2 Where feasible, add an app-level test that constructing an `ExternalPaths` containing two `.md` paths plus one `.txt` and invoking the drop path results in two new tabs and skips the `.txt`. If the GPUI test harness cannot synthesize an `ExternalPaths` drop event, document that this path is verified manually instead and skip the test (do not fake a brittle harness).
- [x] 4.3 Run `cargo fmt`, `cargo build`, `cargo test`, and `openspec validate add-drag-drop-open`.
- [ ] 4.4 Manually verify on Windows: drag one `.md` file from Explorer onto the editor pane -> opens in a new tab; drag several files (mix of `.md` and `.png`) onto the preview pane -> only the `.md` files open, focus on the last; drag a `.txt` file -> nothing opens; drag onto the file tree -> nothing happens.
