## Why

The Files view currently scans and renders the entire current-working-directory tree at startup — every file (`.rs`, `.toml`, `.png`, …) plus all folders. For a Markdown editor this is noisy: non-Markdown files are listed but inert (a `--` marker, not clickable), and on first launch the user is shown the program's own source/build tree rather than their writing workspace. The user wants the Files view to surface only Markdown content, and to start from a clean slate (no directory scan) until a real file is opened.

## What Changes

- **Markdown-only collection.** The file-tree scan SHALL collect only Markdown files (`.md`/`.markdown`/`.mdown`) plus the folders that contain them (folders are retained as nesting rows so the hierarchy is browsable, per user preference). Non-Markdown files SHALL no longer appear in the tree at all.
- **No scan on startup.** When the app launches with the in-memory welcome document (no file path), the Files view SHALL NOT scan the working directory and SHALL NOT render the CWD/folder tree. It SHALL show an empty-state placeholder instead.
- **Tree populates on file open.** The tree SHALL (re)scan only when a real file is opened (via the sidebar, the File → Open dialog, or Save As). The scan root remains the opened file's parent directory, as today (`update_workspace_root_from_document`). With a welcome document the tree stays empty.
- **Empty-state placeholder.** The Files panel body SHALL render an explanatory placeholder (a localized string) when there is no tree yet (initial state) instead of an empty list.
- **Toolbar visibility.** The New/Dir/Ren/Del/Ref toolbar and the filter input SHALL be hidden while in the empty state (they operate on a tree that does not exist yet). They reappear once a file is opened and a tree exists.
- **Removal of dead code.** The non-Markdown rendering arm in the panel body (the `--` marker / muted-inert row) is removed since non-Markdown files are never collected. The `is_markdown` flag is kept on `FileTreeEntry` (the panel still uses it to confirm clickability) but every collected file will now be markdown.
- Non-goals: no multi-document / "open documents" list; no change to the scan root logic (still the open file's parent); no new dependencies; no change to the create/rename/delete/move storage APIs; the existing `should_skip_file_tree_path` ignore list is unchanged (it still prunes `target`/`node_modules`/etc. before the markdown filter).

## Capabilities

### New Capabilities
<!-- None -->

### Modified Capabilities
- `workspace`: The file-tree panel requirement changes from "displays the file and folder hierarchy" of the whole workspace to (a) collecting only Markdown files (plus their containing folders) and (b) starting in an empty state with no scan until a file is opened.

## Impact

- **Code:**
  - `src/storage/file_tree.rs`: `collect_file_tree_entries` is changed to (1) skip non-Markdown files and (2) prune directories that contain no Markdown files (so empty folders don't appear). A directory's inclusion now depends on whether its subtree contains at least one Markdown file. The public scan/CRUD signatures are unchanged.
  - `src/main.rs`:
    - Remove the startup scan call in `main()` (`app.schedule_file_tree_scan(cx)` at the end of `main`). The tree stays `None` until a file is opened.
    - Remove the lazy-scan fallback in `toggle_sidebar` and `set_sidebar_tab` (the `file_tree.is_none()` → `refresh_file_tree` arms) — they would re-introduce a CWD scan on the welcome document.
    - `file_tree_panel_body`: render an empty-state placeholder when `file_tree` is `None` (and the document has no path); otherwise render the toolbar + list as today, but drop the non-Markdown row arm (`marker == "--"`).
  - `src/i18n.rs`: add one new `Msg` key for the empty-state placeholder text (English + Chinese), and add it to the translation-presence exhaustiveness groups.
- **Invariants touched:** The "file tree renders a bounded number of rows per frame" invariant is preserved — the existing `MAX_VISIBLE_TREE_ENTRIES = 300` cap and `filtered_entries_limited` are unchanged; collecting fewer entries only makes it cheaper. No cached-per-version Markdown state is touched.
- **Specs:** `openspec/specs/workspace/spec.md` "File tree panel" requirement's first scenario is updated via delta (markdown-only + empty-state behavior).
- **Persistence/APIs/deps:** None changed.
