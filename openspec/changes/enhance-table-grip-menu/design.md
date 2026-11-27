## Context

The Visual Edit table block grip menu (`BlockMenuState::is_table_block_menu()`) currently exposes whole-table operations at the root (including a root `AddBelow` row) and a two-item `BlockContextSubmenu::Insertion` flyout (`AddDividerBelow`, `AddTableBelow`). Meanwhile, the ordinary context menu's `BlockContextSubmenu::Insertion` already renders a compact 21-item catalog (Paragraph + H1–H5, lists/quote/code/divider, Table, five GFM callouts, and Link/Formula/Image).

Table column widths are persisted via a leading `<!-- markion-cols:P1,P2,… -->` comment applied by `MarkdownDocument::set_table_column_widths_at` for top-level GFM tables with 2–100 columns.

## Goals / Non-Goals

**Goals:**
- Provide one-click equal column width distribution on editable top-level multi-column GFM tables via the existing `markion-cols` persistence path.
- Keep the table block grip root menu focused on whole-table actions by moving paragraph insertion into the Insert Below flyout.
- Unify the table block grip's Insert Below flyout with the paragraph Insert flyout's compact 21-item catalog and make every item (including Link, Formula, and Image) insert cleanly below the table in one undoable edit.
- Preserve all per-version derived Markdown caches, memoized highlighting, and cached text handles until a mutation is confirmed.

**Non-Goals:**
- Changing the table cell right-click menu or row/column handle menus.
- Adding `markion-cols` comment support for tables nested inside blockquotes.
- Modifying ordinary block/paragraph grip or context menus.

## Decisions

### Equalize Column Widths via `equal_column_percents` + `set_table_column_widths_at`

Add `equal_column_percents(columns: usize) -> Option<Vec<u8>>` in `src/table.rs`: for `2 <= columns <= 100`, distribute `100 / columns` with `100 % columns` remainder added to the leading columns (e.g. `[50, 50]`, `[34, 33, 33]`, `[25, 25, 25, 25]`). Every share satisfies `column_percent_floor(columns)` and sums to 100.

Check `can_equalize_table_columns` when building `BlockMenuState` and `BlockMenuPresentation` (top-level unquoted table with a `VisualBlockEditor::Table`, `2..=100` columns, and valid `can_set_table_column_widths_at`). Revalidate `BlockTarget` on activation so stale targets do nothing. Delegate the mutation to `MarkdownDocument::set_table_column_widths_at`, which is a clean no-op when the comment already matches and otherwise commits one atomic undoable edit.

### Unify `BlockContextSubmenu::Insertion` for table block grips

Remove the two-item `is_table_block_menu()` special case in `BlockMenuState::submenu_items(BlockContextSubmenu::Insertion)` and remove `BlockMenuItem::AddBelow` from the table grip root menu. The shared `rows_for` and `block_menu_item_button` paths automatically render the compact Paragraph/H1–H5 row, block row, labeled Table row, compact 5-callout row, and labeled Link/Formula/Image rows under the localized "Insert Below" flyout heading.

All 18 block/callout items already route through `markion::block_edit::insert_block_below` (where `SlashCommand::Text` delegates directly to `add_paragraph_below`) and `insert_callout_below`.

### Deferred below-block insertion for Link, Formula, and Image

Opening Link, Formula, or Image from a table block grip's Insert Below flyout must NOT mutate the document before the user confirms.

Data flow:
`table grip Insert Below -> validate target via add_paragraph_below (read-only) -> dismiss menu -> open LinkEditorState / TokenEditorState with block_below: Some(target)`
- On cancel: clear overlay state; document text, version, selection, undo stack, and derived caches are untouched.
- On confirm (Link / Formula): serialize Markdown (`[label](url)` or `$payload$`), build one checked `BlockEdit` via `markion::block_edit::insert_content_below`, and apply via `apply_exact_block_edit` as a single undoable edit.
- On confirm (Image URL or Choose File): carry `block_below: Option<BlockTarget>` on `PendingImageBatch`; in `finish_image_insertion`, wrap the serialized image Markdown through `insert_content_below` so remote URLs and local/uploaded files insert cleanly below the table in one undoable edit while preserving stale-target safety.

## Risks / Trade-offs

- [Table already equalized] -> `set_table_column_widths_at` detects identical text and skips undo/version bump; the menu closes cleanly without dirtying the document.
- [Document changes while Link/Formula/Image modal or async image upload is open] -> Revalidating `BlockTarget` via `insert_content_below` (and existing `document_version` / anchor checks) rejects stale mutations safely without corrupting surrounding blocks.
- [Existing table grip tests expect root `AddBelow` and two-item flyout] -> Update those assertions to match the new catalog and expand tests for equalize and all Insert Below item families.
