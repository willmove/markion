## Why

Choosing Divider from a table's block-grip menu replaces the entire table with `---`, silently discarding its cells and metadata. The same template conversions can discard prose or code; creating a divider or a fresh table should preserve existing content.

## What Changes

- Remove Divider and Table conversions from Visual Edit block-grip and ordinary editing menus. Provide compact Insert Divider Below and Insert Table Below commands next to Insert Paragraph Below in the whole-block operations group, with localized labels naming the current block.
- Insert after the complete captured source unit, preserving table cells, column-width comments, neighboring bytes, unrelated selection, and LF/CRLF. Divider insertion creates an editable blank paragraph below it and places the caret there; table insertion focuses the new table.
- Reuse the existing checked insertion/history path, reject stale captures, and make each insertion one undoable edit. Reject non-empty Divider/Table replacement at the underlying transformation boundary as a safeguard.
- Keep the existing ordinary Insert flyout and slash palette available. Table-cell context menus retain their selection and row/column catalog.
- **Non-goals:** insertion above, new dialogs, changing other block conversions, table cell/row/column behavior, or redesigning the existing compact Insert flyout.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `markdown-editing`: replace destructive divider/table block conversions with content-preserving insertion below and specify continuation-caret, history, and stale-target behavior.

## Impact

- `src/block_edit.rs`: reject content-discarding template transformations and extend divider insertion to leave a blank continuation paragraph.
- `src/app/mod.rs`, `src/app/editing.rs`, `src/app/preview.rs`: shared menu catalog, compact operations grouping, checked dispatch, icons, and localized tooltip labels.
- `src/i18n.rs`: translate the two scoped insertion labels in all interface languages.
- Existing block-edit, grip, and context-menu tests gain content-loss, exact Undo/Redo, table metadata, LF/CRLF, small-window, pointer/keyboard, and stale-capture coverage.
- No new dependencies or migration. Menu presentation preserves shared per-version Markdown caches, memoized highlighting, cached text handles, selection, dirty state, and history until a command is confirmed; confirmed edits use the established canonical mutation path.
- Tables and dividers retain their rendered WYSIWYG coverage classification; the change affects command semantics and continuation selection only.
