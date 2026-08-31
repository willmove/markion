## Why

Visual Edit currently paints a persistent table-editing header (label plus +Row / -Row / Up / Down / +Col / -Col) above every GFM table. That chrome makes tables look like editor widgets instead of document content, and there is no one-click way to remove the whole table from the same bar.

## What Changes

- Hide the Visual Edit table-editing header by default so an idle table presents as a visual grid, matching the WYSIWYG reading surface.
- Show that header only while the pointer is over the table (including the header itself) or the canonical caret belongs to a cell in that table (click/focus).
- Add a delete-entire-table control to the same header that removes the table through the existing exact block-delete path (one canonical mutation, one undo).
- Keep Split Preview and Read mode free of table-editing chrome.
- **Non-goals:** restoring preview/read table toolbars; changing row/column targeting or disabled-boundary semantics; adding a confirmation dialog; localizing the existing compact `+Row`/`-Row`/… labels; adding a new keyboard shortcut or source-mode Table menu command; changing Markdown table syntax or export.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `tables-outline`: Visual Edit table chrome is interaction-gated (hover or caret in the table) instead of always visible, and the toolbar gains a whole-table delete control that uses the existing block-delete mutation.

## Impact

- Visual Edit table chrome in `src/app/preview.rs` (`visual_table_view` and toolbar helpers). Hover/caret visibility is presentation-only tab state and MUST NOT recompute per-document-version derived Markdown caches, dirty the document, or create history entries.
- Whole-table delete reuses `delete_block` / `delete_visual_block` (`src/block_edit.rs`, `src/app/editing.rs`) rather than extending `TableEdit` row/column mutations.
- New user-visible delete-table label (and any status text) goes through `src/i18n.rs`.
- Rendered GPUI tests in `src/app/tests.rs` and any Visual Edit quality-matrix table evidence in `docs/visual-editing-quality.md`.
- No persistence, public API, dependency, or workspace-member (`crates/*`) changes.
