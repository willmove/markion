## Why

The left file tree currently shows only Markdown files (`.md`/`.markdown`/`.mdown`) and the folders that contain them. Every other text-like file — plain-text notes (`.txt`), logs (`.log`), Org / RST / AsciiDoc sources, CSV — is invisible, and any folder whose subtree contains no Markdown is pruned entirely, including genuinely useful **empty** folders the user created for organization. This makes the tree unhelpful for text-centric workspaces that aren't pure Markdown, and hides on-disk structure (empty folders) the user authored. We want the tree to reflect the workspace's text content and real structure without degenerating into a noisy "show everything" file manager.

## What Changes

- Broaden the file tree's file filter beyond Markdown to a **curated set of plain-text extensions** — `.txt`, `.text`, `.log`, `.csv`, `.tsv`, `.org`, `.rst`, `.adoc`/`.asciidoc` — while keeping binary / image / source-code files (`.png`, `.rs`, `.toml`, `.json`, …) hidden. Markdown remains the primary type and stays visually distinguished.
- Plain-text (non-Markdown) files become **openable in the editor as UTF-8 text** (read/write), not inert rows — reusing the existing document-opening path, which already reads raw bytes without re-checking the extension.
- Stop pruning **empty folders**: a folder appears in the tree when it exists on disk, even when it has no text/markdown children. The hard-coded ignore-directory blacklist (`.git`, `target`, `node_modules`, `__pycache__`, …) still applies unchanged.
- Text files render with a **neutral plain-text icon** (not the Markdown icon) so users can tell types apart at a glance.
- Preserve the 300-row render cap, background scan, one-level-at-a-time expand, collapse semantics, and the existing context-menu actions. "New File" may now also produce text files.

**Non-goals** (explicitly out of scope to prevent creep):
- Showing binary, image, or source-code files — still hidden.
- Showing hidden / dotfile entries (`.gitignore`, `.env`) — covered by the separate `file-tree-show-hidden-files` change.
- A user-configurable filter mode or Preferences toggle — possible future change.
- `.gitignore`-aware exclusion — the hard-coded directory blacklist stays as-is.
- Broadening drag-and-drop — `handle_external_drop` keeps its current Markdown/image-only gate.

## Capabilities

### New Capabilities
<!-- none -->

### Modified Capabilities
- `workspace`: The "File tree panel with filename filtering" requirement changes — the tree now lists a curated set of plain-text files in addition to Markdown, opens them as UTF-8 text, and shows empty folders. Non-text files remain excluded.

## Impact

- **`src/storage/file_tree.rs`** — core change. Introduce a broader text-extension concept alongside the existing `MARKDOWN_EXTENSIONS` (Markdown stays a distinguished subset so `is_markdown_path` is preserved for the drag-drop gate); `collect_file_tree_entries` keeps skipping non-text files but admits the curated text types; remove/relax the empty-folder prune (`entries.truncate(row_index)` at ~line 296) so empty folders survive; `should_skip_file_tree_path` directory blacklist is unchanged.
- **`src/ui/icon.rs`** — resolve a neutral plain-text file icon; `file_tree_icon` routes text files to it, Markdown keeps its current icon.
- **`src/app/root_view.rs`** — the `clickable` predicate (~line 1580) broadens from `kind == File && is_markdown` to `kind == File && is_text` so text rows are clickable; row-icon lookup picks the text icon.
- **`src/app/application.rs` / `src/app/workspace.rs`** — `open_tree_file` already opens arbitrary UTF-8; confirm no Markdown-only assertion rejects text files. Drag-and-drop stays Markdown/image-only (non-goal).
- **`src/i18n.rs`** — the empty-state message ("Open a Markdown file to see it listed here.") and any "non-markdown not shown" wording need updating; possibly a status string for opening a plain-text file.
- **Specs** — `openspec/specs/workspace/spec.md` "File tree panel" requirement and its "Workspace is scanned and displayed as Markdown-only" scenario are rewritten via delta.
- **Tests** — `src/storage/file_tree.rs` (`scan_lists_only_markdown_files`, `scan_prunes_folders_without_markdown`), `src/app/tests.rs`, and the integration tests in `src/lib.rs` get new assertions: text files appear and open, empty folders appear, non-text files and blacklisted directories stay hidden.
- **Invariants preserved**: bounded rows per frame (300 cap unchanged), background scan, per-version cached derived Markdown state untouched, workspace-member gpui ban untouched.
