## Why

Selecting text in a tall Visual Edit table can scroll the entire table away from the pointer. Cells with HTML line breaks can also resolve a click on a text fragment to the edge of the cell, preventing precise partial selections.

## What Changes

- Keep the Visual Edit viewport anchored during in-view cell clicks and drags.
- Share the wrapped text layout between rendered and focused text-only cells and use source-backed hit testing around HTML breaks.
- Cover the reported shape with a synthetic long, multiline Unicode table.

Non-goals include changing Read mode, table structure commands, or introducing new source editing surfaces.

## Capabilities

### Modified Capabilities

- `tables-outline` gains a requirement for stable, precise pointer selection in multiline cells.

## Impact

Visual Edit table layout, caret reveal, and pointer tests. Source mapping reuses the per-version Visual Edit derivation. Pointer interactions preserve source, history, and shared Markdown caches.
