## Why

Converting a Visual Edit GFM table to a quote can remove its cell mapping and open a source editor that requests a missing font. The Windows text backend can panic in this path, and even with the font installed the quoted table loses direct cell editing.

## What Changes

- Resolve missing Windows font families through available fallback families and the system UI font.
- Keep GFM tables inside their parsed quote and map authored cells and inline breaks to the original Markdown.
- Verify that the grip command produces one undoable edit, and that quoted tables remain visible and their authored cells editable.

Non-goals include new structural editing commands for quoted tables and changes to typography preferences.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `document-typography` covers missing Windows font families with deterministic fallback.
- `tables-outline` covers exact cell editing and undo after quoting a GFM table.

## Impact

Windows GPUI font resolution, preview and Visual Edit table derivation, exact cell mutations, and regression tests. Markdown derivations remain cached per version and mutations use the existing checked history path.
