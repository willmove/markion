## Why

After dragging column widths on a GFM table in Visual Edit, users have no one-click way to restore equal column widths across the table. At the same time, the table block grip menu keeps "Insert Paragraph Below Table" as a separate root item while its "Insert Below" flyout only offers divider and table insertion—far more limited than the compact 21-item Insert flyout available on ordinary paragraphs.

## What Changes

- Add an **Equalize Column Widths** ("均分列宽") action to the Visual Edit table block grip menu for top-level editable multi-column GFM tables, writing equal persisted `<!-- markion-cols:… -->` percentages as a single undoable edit.
- Move paragraph insertion out of the table block grip root menu into its **Insert Below** flyout.
- Expand the table block grip's **Insert Below** flyout to use the same compact 21-item catalog and layout as the paragraph context menu's Insert flyout: Paragraph + H1–H5, lists/quote/code/divider, Table, five GFM callouts, and Link/Formula/Image—inserting the chosen block or inline content into a fresh block below the table.
- Reuse existing checked block mutations, per-version derived caches, and localized UI strings (`src/i18n.rs`).

Non-goals: changing the table cell context menu, supporting `markion-cols` comments inside quoted tables, or altering ordinary paragraph/block menus or Markdown syntax rules.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `tables-outline`: add Equalize Column Widths to the table block grip root menu, move paragraph insertion into the Insert Below flyout, and unify the Insert Below flyout catalog with the compact 21-item insertion palette.

## Impact

- `src/table.rs`, `src/lib.rs`: add `equal_column_percents` helper and table column-width capability check.
- `src/block_edit.rs`: expose `insert_content_below` for inserting confirmed Link/Formula/Image Markdown as a standalone block below a target block.
- `src/i18n.rs`, `src/ui/icon.rs`, `assets/icons/ui/columns.svg`: localized Equalize Column Widths labels across all 8 languages and a dedicated columns icon.
- `src/app/mod.rs`, `src/app/editing.rs`, `src/app/preview.rs`, `src/app/image_operations.rs`: table grip menu catalog, Equalize Column Widths action, and below-block confirmation support for Link/Formula/Image editors.
- `src/app/block_grip_tests.rs`, `src/app/tests.rs`: unit and rendered GPUI tests covering Equalize Column Widths and the unified Insert Below flyout (pointer, keyboard, constrained viewport, undo/redo, cancel-without-mutation, stale target rejection, and cache preservation).
