# Design: add-preferences-draggable-scrollbar

## Context

The Preferences panel (`preferences_panel_view` in `src/app/root_view.rs`) renders two tabs. Its scrollable regions are plain id'd divs with `overflow_y_scroll()`: the General tab body (`#preferences-panel-body`), the Shortcuts category sidebar (`#preferences-shortcut-categories`), and the Shortcuts action list (`#preferences-shortcut-actions`). None is tracked by a `ScrollHandle`, so only wheel/trackpad scrolling works.

The main panes already have a draggable overlay scrollbar: `pane_scrollbar_view(target, &ScrollHandle, palette, cx)` draws an absolutely-positioned thumb (9px wide, min 32px tall, 3px edge inset) inside a `.relative()` wrapper, sized from the handle's geometry, and drives scrolling during a left-button drag via window-level mouse events registered on a canvas child. Drag state lives on `MarkionApp` as `PaneScrollbarDrag { target, thumb_grab_offset_y }`, keyed by the `PaneScrollTarget` enum (currently `Editor | Preview | Visual`). The scrollable div cooperates by calling `.track_scroll(&handle)` and reserving a 15px gutter via `.scrollbar_width(px(PANE_SCROLLBAR_RESERVED_WIDTH))` (see the editor pane at `src/app/root_view.rs:448-477`).

See `proposal.md` for motivation; see the delta spec for the required observable behavior.

## Goals / Non-Goals

**Goals:**
- Reuse the existing overlay-scrollbar machinery for all three Preferences panel regions with zero new per-keystroke or per-render document work.
- Keep drag disambiguation correct when two scrollable regions are visible at once (Shortcuts tab shows the category sidebar and action list simultaneously).

**Non-Goals:**
- Refactoring `pane_scrollbar_view` into a generic component; horizontal scrollbars; touch gestures; changes to sync scroll, main-pane scrollbars, or other overlay surfaces.

## Decisions

### 1. Reuse `pane_scrollbar_view` and `PaneScrollbarDrag` as-is; extend `PaneScrollTarget` with three variants

Add `PreferencesGeneral`, `PreferencesShortcutCategories`, and `PreferencesShortcutActions` to `PaneScrollTarget`; `pane_scrollbar_view`'s id mapping gains the three corresponding element ids.

- *Why one variant per region:* the drag state is keyed by target, and in the Shortcuts tab two thumbs are visible at the same time. A single `Preferences` variant would make the mouse-move handler treat a drag on either thumb as a drag on both, scrolling both regions together. Distinct variants give compile-checked disambiguation.
- *Alternative rejected — key the drag by `ScrollHandle` pointer:* would refactor the shared `PaneScrollbarDrag` state and every existing match site for no behavioral gain.
- *Alternative rejected — duplicate a preferences-only scrollbar component:* the third copy of the same geometry/drag math; the existing function is already parameterized by everything needed.

`mark_sync_scroll_driver` (`src/app/appearance.rs:415`) already returns early for any target other than `Editor | Preview`, so the new variants are automatically excluded from sync scroll; `list_pane_scrollbar_marks_sync_driver` is unaffected because these regions use `ScrollHandle`, not `ListState`.

### 2. One `ScrollHandle` per region, stored on `MarkionApp`, created at init

Three new fields (`preferences_general_scroll`, `preferences_categories_scroll`, `preferences_actions_scroll`), constructed in `MarkionApp::new` (in `src/app/application.rs`, next to `preferences_panel_focus`). Per-region handles (rather than one shared) keep each region's offset independent, so switching tabs and reopening the panel restores each region's last position — consistent with how the main panes keep their handles.

### 3. Layout: wrap each scrollable region in a `.relative()` container; reserve the standard gutter

Each region's existing div keeps its id and content but gains `.track_scroll(&handle)` and `.scrollbar_width(px(PANE_SCROLLBAR_RESERVED_WIDTH))` (replacing the current `scrollbar_width(px(8.))`, so content no longer runs under the thumb). The div moves inside a `div().relative()` that carries the region's sizing (`flex_1/min_h_0` for the General body and action list; `w(px(152.)).flex_none()` for the category sidebar), with `pane_scrollbar_view` as a sibling overlay child — mirroring the editor pane structure.

### 4. Rendering data flow (invariant check)

The thumb's geometry is computed inside `preferences_panel_view` per render from the handle's `bounds()/max_offset()/offset()` — the same read-only pattern the main panes use. A drag mutates only `PaneScrollbarDrag` and the handle offset, then notifies. No Markdown-derived caches (preview blocks, outline, stats, highlight memoization) are touched, and the panel is not on the typing path, so the architecture invariants hold unchanged.

## Risks / Trade-offs

- [Thumb overlay could intercept clicks on nearby content] → The thumb sits inside the 15px reserved gutter and already uses `block_mouse_except_scroll`, exactly as in the main panes.
- [Scroll position now persists across panel close/reopen] → Intentional (matches main-pane handles); if undesired later, resetting the handle offset on open is a one-line follow-up.
- [Headless tests can't easily assert drag physics] → Automated tests cover the sync-driver no-op, handle wiring, and panel regression rendering; drag feel is verified manually per the tasks' checklist.

## Migration Plan

Pure additive UI change; no persistence or API changes. Rollback is a plain revert.
