## Why

Currently, clicking an outline heading always jumps to the source editor position, even in Read mode where the editor pane is hidden. Users in Read mode expect outline clicks to scroll the preview pane to the corresponding rendered content, not to an invisible editor position.

## What Changes

- Outline click behavior becomes context-aware: in Read mode (and optionally Split mode), clicking a heading scrolls the preview pane to the corresponding rendered block instead of jumping the editor cursor.
- The preview ListState gains the ability to scroll to a specific item by index or by searching for a block matching a given heading offset.
- The outline panel's click handler checks the current ViewMode and routes to either `jump_to_offset` (Edit mode) or a new `scroll_preview_to_heading` method (Read/Split modes).

**Non-goals**: This change does not introduce bidirectional sync (preview scroll updating outline highlight) or smooth animated scrolling. Those are future enhancements.

## Capabilities

### New Capabilities
<!-- None — this is an enhancement to existing outline navigation -->

### Modified Capabilities
- `tables-outline`: The "Document outline navigation" requirement's "Click to jump" scenario currently describes jumping to source position. This change extends it to also support jumping to preview content when the editor is not the primary view.

## Impact

- **Affected code**:
  - `src/main.rs`: `outline_panel_body` click handler, new method to scroll preview ListState
  - `src/model.rs`: possibly exposing a helper to map heading offset → preview block index
- **User experience**: Outline becomes useful in Read mode; no breaking changes to existing Edit mode behavior.
- **Architecture**: Preserves the invariant that derived state (outline, preview blocks) is cached and shared via Arc — no new recomputation on every frame.
