# Implementation Plan: Markdown-only file tree

## Overview

Two coordinated edits make the Files sidebar a Markdown-only view that starts empty: (1) the scan in `src/storage/file_tree.rs` collects only Markdown files and the folders that contain them; (2) `src/main.rs` stops scanning on startup, shows an empty-state placeholder while no file is open, and drops the dead non-Markdown row rendering. The scan-root logic (open file's parent) and the bounded-row invariant are unchanged. Each task is one testable commit. No cached-per-version Markdown invariants are on this path.

## Tasks

- [x] 1. Markdown-only collection (`src/storage/file_tree.rs`)
  - [x] 1.1 Change `collect_file_tree_entries` so regular (non-directory) entries are only pushed when their extension is `md`/`markdown`/`mdown`. Non-Markdown files are skipped entirely (no longer collected as inert `--` rows).
  - [x] 1.2 Make directory inclusion subtree-aware: a directory is pushed (and recursed into) only if its subtree contains at least one Markdown file. Use the recursive return to signal "had a markdown descendant" so empty folders don't pollute the tree. The root call must still succeed even if the root itself has no markdown files (returns an empty entry list).
  - [x] 1.3 Preserve the existing sort order (directories first, then alphabetical by name) and depth accounting for the directories that remain.
  - [x] 1.4 Keep the `should_skip_file_tree_path` ignore list (`target`, `node_modules`, hidden dirs, …) exactly as-is — it still runs before the markdown filter.
  - [x] 1.5 Add/extend a unit test asserting: a tree with mixed files lists only markdown files, folders with no markdown anywhere are absent, and a root with no markdown yields an empty (but `Ok`) tree.
  - [x] _Requirements: workspace (markdown-only hierarchy)_

- [x] 2. Stop the startup scan (`src/main.rs`)
  - [x] 2.1 Remove the `app.schedule_file_tree_scan(cx)` call at the end of `main()` so the tree stays `None` on the welcome document. Leave `schedule_file_tree_scan` itself in place — it is still used by `update_workspace_root_from_document` and the Refresh action.
  - [x] 2.2 Remove the lazy-scan fallback in `toggle_sidebar` (the `if sidebar_visible && sidebar_tab == Files && file_tree.is_none()` → `refresh_file_tree` arm) — it would re-introduce a CWD scan on the welcome document when the sidebar is revealed.
  - [x] 2.3 Remove the matching lazy-scan fallback in `set_sidebar_tab` (the `if tab == Files && file_tree.is_none()` → `refresh_file_tree` arm) for the same reason.
  - [x] 2.4 Confirm the tree still populates on real opens: `update_workspace_root_from_document` → `refresh_file_tree` → `schedule_file_tree_scan` is unchanged and covers File→Open dialog, sidebar click, and Save As.
  - [x] _Requirements: workspace (no scan until a file is opened)_

- [x] 3. Empty-state placeholder in the panel body (`src/main.rs`, `src/i18n.rs`)
  - [x] 3.1 Add `Msg::FileTreeEmptyState` to the `Msg` enum in `src/i18n.rs` with English text (e.g. "Open a Markdown file to see it listed here.") and Chinese text. Add it to the translation-presence / exhaustiveness test groups so the build stays exhaustive.
  - [x] 3.2 In `file_tree_panel_body`, when `app.file_tree` is `None`, render only the placeholder string (muted, centered) and return early — skip the root subheading, filter input, toolbar, and list. When `file_tree` is `Some`, render the existing panel (now markdown-only by virtue of task 1).
  - [x] 3.3 Verify the active-row highlight (`active_path`) still works after a file is opened (the tree now exists and contains the opened file).
  - [x] _Requirements: workspace (empty state until a file is opened)_

- [x] 4. Drop the dead non-Markdown rendering arm (`src/main.rs`)
  - [x] 4.1 Remove the `FileTreeEntryKind::File => "--"` marker arm; the `marker` match now has only `Directory => "dir"` and `File => "md"` arms. `clickable = kind == File && is_markdown` is kept (Directory rows stay non-clickable, and `is_markdown` is read defensively).
  - [x] 4.2 Confirm no other code paths still render non-markdown tree entries (status-bar `file_tree_summary` counts the same `filtered_entries_limited`, which now returns only markdown).
  - [x] _Requirements: workspace (markdown-only listing)_

- [x] 5. Verify and finalize
  - [x] 5.1 `cargo build` and `cargo test` pass (108 tests pass, including the 3 new `file_tree.rs` tests and the i18n exhaustiveness guard).
  - [x] 5.2 `openspec validate markdown-only-file-tree` passes.
  - [ ] 5.3 Manual check: on a fresh launch the Files view shows the placeholder and does no directory I/O; after File→Open of a `.md` file the tree lists only Markdown files under that file's directory, nested by folder. (Deferred to the user — cannot launch a GUI window in this environment.)
