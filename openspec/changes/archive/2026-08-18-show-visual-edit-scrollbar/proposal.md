## Why

Visual Edit already virtualizes long documents with a per-tab `ListState` and reserves right-side padding for a scrollbar, but it never draws a draggable overlay. Read mode (and Split Preview) do, so users navigating a long document in Visual Edit can only wheel or trackpad-scroll and have no visible thumb or jump target. The gap is a chrome inconsistency, not a missing document model.

## What Changes

- Expose a visible, right-side vertical scrollbar on the Visual Edit surface whenever the virtualized document exceeds the viewport, matching the existing Read-mode preview overlay in placement, thumb sizing, and drag behavior.
- Drive that overlay from the active tab's existing `visual_list` `ListState`, so wheel/trackpad scrolling and scrollbar dragging share one per-tab scroll position.
- Keep the Visual Edit input overlay from stealing scrollbar pointer events, while leaving caret, selection, IME, and block-menu interactions otherwise unchanged.
- Preserve short-document behavior: hide the thumb when content fits, and keep empty-document placeholder layout.
- **Non-goals:** enabling Sync scroll in Visual Edit; changing Edit/Split Preview/Read scrollbar chrome; adding a scrollbar preference; altering Markdown parsing, visual-block mapping, or derived-state caches.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `chrome-platform`: Extend the existing pane-scrollbar requirement so Visual Edit, not only the source editor and rendered preview, shows a draggable right-side vertical scrollbar for overflow content.
- `markdown-editing`: Clarify that Visual Edit's per-tab virtualized list scroll state is preserved and updated by the same wheel/trackpad/scrollbar inputs, without mutating document text or derived Markdown caches.

## Impact

- Affected UI is concentrated in `src/app/root_view.rs` (`visual_edit_surface_view` and the existing `preview_list_scrollbar_view` overlay) plus any small `PaneScrollTarget` / overlay-hit-testing adjustment needed so Visual Edit drag does not masquerade as preview Sync-scroll input.
- Reuses the current ListState scrollbar API (`viewport_bounds`, `max_offset_for_scrollbar`, `scroll_px_offset_for_scrollbar`, `set_offset_from_scrollbar`, drag-height freeze) already used by Read mode.
- Must preserve per-document-version derived `Arc` caches, memoized highlighting, cached text handles, and Visual Edit's independent `visual_list` (not `preview_list`) scroll handle.
- No new runtime dependencies, preferences, or persistence fields.
