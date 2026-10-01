## Why

The file-tree watcher reacts to every filesystem event under the workspace root by rescanning the whole tree. In a large vault, events that cannot change the tree arrive constantly: Markion's own autosave writing file contents, Obsidian rewriting `.obsidian/workspace.json`, and Git or sync tools churning inside `.git`. Each one triggers a full background rescan of tens of thousands of entries for no visible change.

## What Changes

- Watched refreshes ignore events that cannot change what the file tree shows:
  - content-only and metadata-only modifications of existing files (including Markion's own saves and autosaves);
  - events whose every affected path lies inside a directory the tree always excludes (`.git`, `node_modules`, `target`, …);
  - while hidden entries are not shown, events whose every affected path is a dot-named entry or lies inside one (for example `.obsidian`).
- Creates, deletes, and renames of visible entries refresh the tree as before. Events of unknown kind, and backend "rescan needed" signals, still refresh.
- The periodic fallback refresh (used when a watch cannot start) and manual Refresh are unchanged.

Non-goals: no incremental tree patching (a relevant event still triggers the existing full background scan); no change to which entries the tree shows; no change to the watch debounce interval.

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `workspace`: adds a requirement that watched refreshes skip events that cannot change the visible tree. The existing "File tree SHALL refresh when the workspace changes on disk" requirement already scopes detection to create, delete, and rename. It is not modified.

## Impact

- Code: `src/app/application.rs` (`arm_file_tree_watch` filters `notify` events before signalling a refresh), `src/storage/file_tree.rs` (a path-relevance helper that reuses the scan's exclusion and dot-name rules), unit tests in both.
- Invariants: automatic refreshes still never reload open documents, dirty state, undo history, or derived Markdown caches. Fewer spurious rescans also means fewer `file_tree` replacements and fewer redraws while typing with autosave on.
