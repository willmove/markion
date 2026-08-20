# Proposal: add-preferences-draggable-scrollbar

## Why

The Preferences panel's scrollable regions (General tab body, Shortcuts category sidebar, Shortcuts action list) only scroll via mouse wheel or trackpad: no scrollbar thumb is visible or draggable, users get no position affordance, and on setups without a wheel the lower part of the panel is unreachable. The main panes (editor, preview, Visual Edit) already solved this exact problem with a draggable overlay scrollbar, so the Preferences panel is inconsistent with the rest of the chrome.

## What Changes

- Add a visible right-side vertical scrollbar thumb to each scrollable Preferences panel region that can be dragged with the left mouse button to scroll up and down.
- Reuse the existing `pane_scrollbar_view` overlay pattern: track each scrollable body with a `ScrollHandle` (`.track_scroll`), reserve a right gutter so content is not occluded, and overlay the draggable thumb inside a `.relative()` wrapper.
- Extend `PaneScrollTarget` with a `Preferences` variant (per-region identification) so the shared drag state (`PaneScrollbarDrag`) and mouse plumbing are reused. Sync scroll is unaffected: `mark_sync_scroll_driver` already ignores targets other than Editor/Preview.
- Hide the thumb when the region's content fits (matching main pane behavior).
- Keep wheel/trackpad scrolling unchanged.

### Non-goals

- No horizontal scrollbars, no touch-gesture support, no changes to main-pane scrollbars or sync scroll.
- No changes to other overlay surfaces (search palette, menus) — they can be addressed in follow-up changes if desired.

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `chrome-platform`: the "Dense pane chrome with draggable scrollbars" requirement is extended to cover the Preferences panel's scrollable regions — a draggable right-side scrollbar SHALL appear whenever a Preferences panel region overflows, and SHALL hide when content fits.

## Impact

- `src/app/root_view.rs` — `preferences_panel_view` (General body), `preferences_shortcuts_body` (categories sidebar + actions list), `pane_scrollbar_view` (target id mapping); wrap scrollables in `.relative()` containers with the thumb overlay.
- `src/app/mod.rs` — `PaneScrollTarget` new variant; new per-region `ScrollHandle` fields on `MarkionApp`.
- `src/app/application.rs` — construct the new `ScrollHandle`s at app init.
- `src/app/tests.rs` — regression tests mirroring existing pane-scrollbar coverage (constants, sync-driver no-op for the new target, hidden-when-fits behavior).
- Invariants preserved: the scrollbar overlay reads only `ScrollHandle` geometry each render; no derived Markdown state is recomputed, and no per-keystroke work is added (the panel is not on the typing path).
