## Context

Markion currently has path-based file opening flows in `src/main.rs`: file-tree opens create a new tab, File->Open replaces the active tab, Open in New Tab appends a tab, and the drag/drop handler from the in-progress drag/drop change appends a tab per dropped Markdown file. Once a file is open, its `EditorTab` owns isolated cursor, scroll, undo, preview-list, and document cache state.

## Goals / Non-Goals

**Goals:**

- Reuse an existing tab when the requested file path is already open.
- Preserve the existing tab's editor/preview scroll position, selection, undo/redo stacks, dirty state, and derived Markdown caches when it is focused.
- Keep unopened-file behavior unchanged for File->Open, file-tree opens, Open in New Tab, and drag/drop.
- Normalize paths enough that equivalent filesystem paths compare reliably.

**Non-Goals:**

- No deduplication for untitled documents or recovery documents without a concrete path.
- No background memory compaction or tab persistence changes.
- No UI prompt for already-open files.

## Decisions

- Compare canonical paths when possible, falling back to the provided path if canonicalization fails. This handles common relative/absolute differences for existing files while still allowing error handling to report the original open failure.
- Add a small helper on `MarkionApp` that finds an already-open tab by path and focuses it, then refreshes search matches and notifies the UI. This avoids resetting any per-tab state.
- Add path-oriented open helpers so flows can check for an existing tab before paying to construct a new `MarkdownDocument`.
- File->Open keeps the existing dirty-guard-before-picker flow, but once a path has been selected it checks for an already-open tab before replacing the active tab.

## Risks / Trade-offs

- Canonicalization can fail for missing files or inaccessible paths -> fall back to normal open error paths for unopened files.
- Case sensitivity differs by platform -> canonical paths are used when available; Windows canonicalization gives stable comparisons for ordinary existing files.
- The drag/drop change is already modifying `src/main.rs` -> keep edits narrowly scoped and reuse its `is_markdown_path` filtering without reverting those changes.
