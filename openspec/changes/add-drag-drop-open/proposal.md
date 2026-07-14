## Why

Today the only ways to open a file are the file tree, the File -> Open dialog, and Save As. A user who already has a file in their OS file manager (Windows Explorer / Finder / a Linux file manager) must use the File -> Open dialog and re-navigate to the file they already have in front of them. Modern editors commonly let the user drop a file straight onto the window to open it; Markion does not.

GPUI already does the heavy lifting: the platform layer (Windows `IDropTarget`, macOS, Wayland, X11) translates an OS file drag into a `FileDropEvent`, and on `Entered` it publishes the dragged paths as an `active_drag` whose value is an `Arc<ExternalPaths>`. When the user releases, GPUI fires `on_drop::<ExternalPaths>` on whatever element is under the cursor. Markion never registers such a handler today (confirmed: `grep ExternalPaths src/` is empty), so drops currently do nothing. This change wires up that existing primitive.

## What Changes

- The editor SHALL accept files dragged from the operating system file manager and dropped onto the **editor pane** or the **preview pane**.
- On drop, the editor SHALL open each dropped path whose extension is `md`, `markdown`, or `mdown` as a **new tab**, with the focus following the last opened file. This matches the existing file-tree open behaviour (`open_file_in_new_tab_from_path`, `src/main.rs:2473`).
- Non-Markdown files and directories SHALL be silently ignored (they are simply not opened); opening them into a Markdown editor would either show garbage or fail `fs::read_to_string`.
- A drop that opens at least one file SHALL set the status bar to the existing `StatusOpened` message (`tf(Msg::StatusOpened, &[last_opened_path])`). The opened tab's own load path is used as the `{0}` argument, so no new i18n string is introduced.
- The drop handler SHALL be registered on both the editor pane `div` (`src/main.rs:5103`) and the preview pane `div` (`src/main.rs:5170`). The sidebar / file-tree area is **not** a drop target, so dropping onto the tree does nothing surprising (the tree keeps its click-to-open semantics; UI drag-move of tree rows is out of scope).

## Non-Goals

- No drag-and-drop **move / copy** of files within the file tree (the `workspace` spec already calls this out as a future candidate; it is a separate feature).
- No internal drag from a file-tree row onto the editor to open it (the tree already opens on click; this change only covers OS-originated drops).
- No drop-hover visual affordance (highlight overlay while the OS drag is in flight). GPUI already renders the OS-provided file icons during the drag; a custom hover highlight is polish that can come later.
- No support for non-Markdown files (images, `.txt`, etc.) and no "import as link" behaviour.
- No new i18n strings; the change reuses the existing `StatusOpened` message.
- No changes to document parsing, derived Markdown caches, syntax-highlighting memoization, cached text handles, undo snapshots, or the bounded-row tree rendering.

## Capabilities

### New Capabilities

- None.

### Modified Capabilities

- `workspace`: gains a fourth way to open a file (OS drag-and-drop onto the editor or preview pane), alongside the file tree, the File -> Open dialog, and Save As. The Markdown-only open filter and the open-in-new-tab behaviour are reused as-is.

## Impact

- Affected code:
  - `src/main.rs` - new `handle_external_drop` handler that iterates `ExternalPaths::paths()`, filters by Markdown extension, and calls the existing `open_file_in_new_tab_from_path` for each match (the last call wins focus); one `use gpui::ExternalPaths;` import; two `.on_drop::<ExternalPaths>(cx.listener(Self::handle_external_drop))` calls added to the editor pane `div` (~`src/main.rs:5103`) and the preview pane `div` (~`src/main.rs:5170`).
- Affected specs: `workspace` (one new requirement, "Open files via drag-and-drop from the OS").
- APIs/dependencies: no public API changes, no new dependencies. Uses `gpui::ExternalPaths` and `gpui::FileDropEvent`, both already re-exported by the `gpui = "0.2.2"` dependency.
- Invariants: the handler only touches filesystem reads (via `MarkdownDocument::open`, called inside `open_file_in_new_tab_from_path`) and app-level tab state. It does NOT touch per-document derived Markdown caches, the syntax-highlighting memo, cached text handles, or undo snapshots beyond what `open_in_new_tab` already does.
