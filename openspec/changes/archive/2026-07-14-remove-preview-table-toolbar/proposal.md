## Why

Preview surfaces are intended for reading, but preview tables still expose row and column mutation controls above every table. Now that Visual Edit provides the dedicated rendered editing surface, those controls add clutter and make Split Preview and Read mode appear editable when they should remain read-only.

## What Changes

- Remove the table editing toolbar, including its add/delete/move row and column controls, from rendered preview tables in both Split Preview and Read mode.
- Keep GFM tables visually rendered in preview surfaces without an editing header above the grid.
- Preserve table editing in Visual Edit and through the existing source-table commands.
- Preserve preview table text selection, rendering, scrolling, and export behavior.
- Non-goal: change Visual Edit table controls, source table commands, direct cell editing, Markdown parsing, or table export fidelity.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `tables-outline`: Preview tables become read-only visual grids without row/column toolbars, while Visual Edit and source commands remain the supported table-editing surfaces.

## Impact

- Affected UI rendering: the `PreviewBlock::Table` branch in `src/main.rs`, shared by Split Preview and Read mode.
- Visual Edit's separate table rendering and toolbar remain unchanged.
- Tests should distinguish read-only preview rendering from Visual Edit table editing and continue covering source-table commands.
- No API, persistence, dependency, localization, document-format, or cached derived-state changes are required; the per-document-version caching invariants are untouched.
- This change assumes the completed `add-visual-edit-mode` change remains present and should be archived before or together with this dependent specification update.
