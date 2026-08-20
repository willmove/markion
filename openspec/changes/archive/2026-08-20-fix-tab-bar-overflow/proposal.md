# fix-tab-bar-overflow

## Why

With enough tabs open, tabs beyond the tab band's available width are clipped with no way to see, reach, or close them — they are invisible and unclickable, and nothing else in the UI can bring them back. Long filenames compound the problem by consuming disproportionate strip space. This is a reachability defect in a core navigation surface, not a cosmetic preference.

## What Changes

- The document tab strip becomes a horizontally scrollable region: mouse-wheel and trackpad horizontal scrolling over the strip move it (GPUI routes vertical wheel deltas to horizontal-only scroll containers natively; no visible scrollbar is drawn).
- Each tab's width becomes bounded: labels truncate with an ellipsis past a maximum tab width, tabs shrink as more are opened, and below a minimum width the strip scrolls instead of clipping.
- The dirty indicator stops being a `" *"` suffix baked into the label string and becomes a separate non-shrinkable element, so truncation can never hide it (a truncated `"name…"` would otherwise read as saved).
- Switching, opening, closing, or focusing an existing tab auto-scrolls the strip the minimal amount needed to reveal the newly active tab.
- The trailing "+" (new tab) button is pinned outside the scroll region so it is always visible and clickable.
- Hovering a tab shows a tooltip with the tab's full title (and its full path when the tab is file-backed), restoring information lost to truncation.
- Non-goals: visible overflow chevrons or a hidden-tabs dropdown menu, drag-to-reorder, middle-click close, multi-row wrapping, tab pinning, and any change to the single-tab (no tab bar) layout.

## Capabilities

### New Capabilities

- `document-tab-bar`: Presentation-layer behavior of the document tab strip when many tabs are open — scrolling, per-tab width bounding and label truncation, dirty-indicator visibility under truncation, active-tab auto-reveal, pinned strip actions, and hover tooltips.

### Modified Capabilities

None. The multi-document tab model in `markdown-editing` (tab state isolation, path uniqueness, single-tab layout, session-only tabs) is unchanged; this change only affects how the strip presents overflow.

## Impact

- `src/app/editing.rs` — `tab_bar_view` restructured: scroll container, bounded tab widths, truncated labels, separate dirty element, tooltip attachment, "+" moved outside the scroll region.
- `src/app/state.rs` — `MarkionApp` gains a `tab_bar_scroll: ScrollHandle` field (app-level, not per-tab; strip scroll position is window-presentation state).
- `src/app/application.rs` / `src/app/documents.rs` — tab switch/new/close/focus-existing paths request `scroll_to_item` for the active tab.
- `src/app/tests.rs` — structural assertions on the new strip layout and behavioral coverage of the auto-reveal request.
- No new user-facing strings (tooltip text is the tab's title/path data, not chrome copy), no i18n changes, no dependency or workspace changes.
- The tab band height (`DOCUMENT_TAB_BAND_HEIGHT = 30`) and all consumers of `document_tab_band_height` are unaffected; the change is presentation-only and does not touch the derived-Markdown-state caching invariants (tab bar rendering performs no document parsing).
