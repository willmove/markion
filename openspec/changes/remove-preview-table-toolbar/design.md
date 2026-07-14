## Context

Split Preview and Read mode both render tables through the `PreviewBlock::Table` branch of `preview_block_view` in `src/main.rs`. That branch currently builds an editing header above the table grid and wires six buttons to `preview_table_button` and `TableEdit`. Visual Edit uses a separate `visual_table_view`, so its table controls can remain available without exposing mutation actions in read-oriented preview surfaces.

The relevant rendering flow is:

```text
MarkdownDocument.text
  -> cached PreviewBlock::Table for the document version
  -> preview_block_view
  -> Split Preview or Read mode table grid
```

This change only removes children from the final preview rendering step. It does not alter the cached preview block, document versioning, table parser, mutation path, or Visual Edit rendering.

## Goals / Non-Goals

**Goals:**

- Make tables in Split Preview and Read mode purely read-oriented by removing the editing header and all row/column controls.
- Keep the table grid and selectable cell text unchanged.
- Keep Visual Edit and source commands as the supported table-editing paths.
- Preserve the existing per-document-version derived-state and text-handle cache boundaries.

**Non-Goals:**

- Remove or redesign the Visual Edit table toolbar.
- Remove source table commands or keyboard shortcuts.
- Add direct visual cell editing.
- Change GFM parsing, alignment handling, export output, table data structures, or localization strings used elsewhere.

## Decisions

### 1. Remove the complete preview-only table header

The `PreviewBlock::Table` branch will render the bordered table grid directly, without the header row that contains the `Table` label and row/column buttons. A label-only header would retain unused vertical chrome and would not help reading, so the entire header is removed.

Alternative considered: hide only the buttons in Read mode. Rejected because Split Preview's rendered pane is also a preview surface; mutations belong in its adjacent source editor or in Visual Edit.

### 2. Keep Visual Edit controls unchanged

`visual_table_view` remains the rendered table-editing surface and continues to use the existing source-backed `TableEdit` path. The shared button helper remains available because Visual Edit still consumes it.

Alternative considered: remove all rendered table controls and require source commands exclusively. Rejected because that would unnecessarily reduce the dedicated Visual Edit capability and exceed the requested scope.

### 3. Do not change derived table data or editing commands

No mode flag will be added to `PreviewBlock`, and no parser or cache invalidation logic will change. The mode-specific behavior already follows from the separate preview and Visual Edit render functions.

Alternative considered: pass an `editable` flag into a shared table component. Rejected for this small change because the current paths are already separate and a new abstraction would add scope without improving the requested behavior.

## Risks / Trade-offs

- [Risk] A user accustomed to editing tables from Split Preview loses that direct path. → Mitigation: the adjacent source editor, Visual Edit toolbar, menus, and shortcuts remain available.
- [Risk] A broad deletion could remove Visual Edit controls because the button helper is shared. → Mitigation: limit the UI change to the `PreviewBlock::Table` rendering branch and retain `visual_table_view` plus the helper.
- [Risk] Removing the header could disturb table borders or text selection. → Mitigation: leave the grid row/cell rendering intact and add focused rendering tests or structural assertions where practical.

## Migration Plan

No data or preference migration is required. Implement the preview-only render change, run targeted table/view-mode tests and the root test suite, and manually verify one table in Split Preview, Read, and Visual Edit. Rollback is restoring the preview header construction; document contents are unaffected.

The completed `add-visual-edit-mode` change should be archived before this change so the `tables-outline` requirement is updated in dependency order.

## Open Questions

None.
