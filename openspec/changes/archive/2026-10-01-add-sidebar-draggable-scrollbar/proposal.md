## Why

The left sidebar's Files (file tree) and Outline panels already scroll with the mouse wheel or trackpad, but GPUI does not draw a usable native scrollbar. When a workspace has many files or a document has many headings, users cannot see their position or drag the view down, which is inconsistent with the editor, preview, Visual Edit, and Preferences overlay scrollbars.

## What Changes

- Add a visible, right-side vertical scrollbar thumb to the Files panel's tree list and the Outline panel's heading list whenever that list overflows its visible height.
- Reuse the existing `pane_scrollbar_view` overlay: wrap each already-tracked scroll region in a `.relative()` container, reserve the standard right gutter, and overlay a draggable thumb that follows the region's `ScrollHandle`.
- Extend `PaneScrollTarget` with `FileTree` and `Outline` variants so the shared drag state can tell the two sidebar thumbs apart. Sync scroll stays unaffected: those targets are never Editor/Preview drivers.
- Hide the thumb when the region's content fits. Wheel and trackpad scrolling continue to work.
- Preserve the file tree's bounded-rows-per-frame rendering and the outline's per-document-version cached headings.

### Non-goals

- No horizontal overlay scrollbar for the file tree (long names already use native overflow-x). No virtualization of outline rows, no change to sidebar resize, tab switching, persistence, or main-pane / Preferences scrollbars.

## Capabilities

### New Capabilities

- None.

### Modified Capabilities

- `chrome-platform`: extend "Dense pane chrome with draggable scrollbars" so the visible Files and Outline sidebar lists get the same draggable right-side overlay as other overflow chrome.
- `workspace`: require that an overflowing file-tree list exposes a draggable vertical scrollbar without changing scan, filter, or bounded-row behavior.
- `tables-outline`: extend "Overflowing outline is vertically scrollable" so overflow is reachable by a visible draggable scrollbar as well as wheel/trackpad input.

## Impact

- `src/app/root_view.rs` — wrap `#file-tree-scroll` and `#outline-scroll` in `.relative()` containers, switch their gutter to `PANE_SCROLLBAR_RESERVED_WIDTH`, overlay `pane_scrollbar_view`; extend the target-id mapping.
- `src/app/mod.rs` — add `FileTree` and `Outline` variants to `PaneScrollTarget`; resolve exhaustive-match fallout (sync-scroll sites keep ignoring non-Editor/Preview targets).
- `src/app/appearance.rs` — treat the new variants as sync-scroll no-ops, matching Preferences/Visual.
- `src/app/tests.rs` — regression tests for hidden-when-fits, independent drag identity, sync-driver no-op, and existing outline/file-tree scroll behavior.
- Invariants preserved: the overlay reads only `ScrollHandle` geometry; file-tree rows stay capped per frame; outline headings stay on the per-document-version `Arc` cache. No Markdown recompute, no new i18n strings, no persistence changes.
