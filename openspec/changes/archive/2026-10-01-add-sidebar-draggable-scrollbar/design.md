## Context

The left sidebar (`sidebar_view` in `src/app/root_view.rs`) switches between Files (`file_tree_panel_body`) and Outline (`outline_panel_body`). Both lists already overflow and track scroll:

- `#file-tree-scroll` uses `overflow_y_scroll()` + `overflow_x_scroll()`, `.track_scroll(&app.file_tree_scroll)`, and `scrollbar_width(px(8.))`.
- `#outline-scroll` uses `overflow_y_scroll()`, `.track_scroll(&app.outline_scroll)`, and `scrollbar_width(px(8.))`.

GPUI does not draw a usable native scrollbar at that width, so users can only wheel/trackpad-scroll. The editor, Visual Edit, preview, and Preferences surfaces already solved this with `pane_scrollbar_view`: an absolutely positioned overlay thumb inside a `.relative()` wrapper, driven by `PaneScrollbarDrag { target, thumb_grab_offset_y }` keyed on `PaneScrollTarget`.

The file tree already caps visible rows at 300 per frame. Outline headings come from the per-document-version cached `Arc` list. Those invariants must stay intact.

See `proposal.md` for motivation; see the delta specs for observable behavior.

## Goals / Non-Goals

**Goals:**
- Give Files and Outline the same visible, left-button-draggable right-side overlay as other overflow chrome, using the existing `ScrollHandle`s and `pane_scrollbar_view`.
- Keep wheel/trackpad scrolling, sidebar resize, tab switching, and file-tree horizontal overflow for long names.
- Keep drag identity independent of main-pane and Preferences thumbs, and never mark Sync scroll.

**Non-Goals:**
- Horizontal overlay scrollbar; outline virtualization; changing the 300-row file-tree cap, scan/filter/expansion policy, or heading derivation; touching main-pane or Preferences scrollbars.

## Decisions

### 1. Reuse `pane_scrollbar_view`; add `FileTree` and `Outline` to `PaneScrollTarget`

The sidebar lists are `ScrollHandle` regions, not `ListState` lists, so they use `pane_scrollbar_view` (not `list_pane_scrollbar_view`). Add two variants and two overlay ids (`file-tree-scrollbar`, `outline-scrollbar`).

- *Why two variants:* drag state is keyed by target. Distinct variants keep a leftover Files drag from moving Outline after a tab switch, and match the Preferences pattern.
- *Alternative rejected — new sidebar-only scrollbar widget:* a third copy of the same geometry/drag math.
- *Alternative rejected — one `Sidebar` variant:* collapsing Files and Outline into one identity would make a mid-drag tab switch apply the grab offset to the other list.

`mark_sync_scroll_driver` already returns early unless the target is `Editor | Preview`. `reconcile_sync_scroll` needs the new variants in its exhaustive ignore arm, next to Visual/Preferences.

### 2. Keep the existing `file_tree_scroll` and `outline_scroll` handles

Unlike Preferences, the sidebar already constructs these in `MarkionApp::new` and resets `file_tree_scroll` when the workspace root changes. Overlay wiring is layout-only; no new app fields.

### 3. Layout: wrap each scroll div in `.relative()`; reserve `PANE_SCROLLBAR_RESERVED_WIDTH`

Mirror the editor pane (`src/app/root_view.rs` around the `#editor-scroll` wrapper):

- Files: keep the workspace-name heading and inline-name-editor fallback outside the scroll wrapper. Only `#file-tree-scroll` moves inside `div().relative().flex_1().min_h_0()`, with `pane_scrollbar_view(FileTree, …)` as a sibling. Keep `overflow_x_scroll()`. Replace `scrollbar_width(px(8.))` with `PANE_SCROLLBAR_RESERVED_WIDTH` so row labels are not under the thumb.
- Outline: the same wrap around `#outline-scroll`. Image-tab placeholder stays unscrollable and has no thumb.

`file_tree_content_width` (used for horizontal overflow of long names) SHALL include the reserved gutter so a name that would sit under the thumb remains reachable by horizontal scroll.

Only the active sidebar tab is built today; at most one sidebar thumb is on screen.

### 4. Rendering data flow (invariant check)

Each frame the overlay reads `ScrollHandle::bounds()`, `max_offset()`, and `offset()` and draws or hides the thumb. A drag writes only `pane_scrollbar_drag` and the handle offset, then `cx.notify`.

```
sidebar tab render
  → existing file-tree row cap (≤300) / outline() cache hit
  → ScrollHandle geometry
  → pane_scrollbar_view thumb
drag
  → set_offset on the same handle
  → notify (no document version bump)
```

No preview/outline/stats recompute, no highlight invalidation, no text-handle replacement. File-tree bounded rendering is unchanged. Outline still uses `document.outline()` (cached per version).

## Risks / Trade-offs

- [Narrow sidebar: 15px gutter eats label width] → Same reserved width as every other overlay; long file-tree names stay reachable via existing horizontal overflow plus the gutter in `file_tree_content_width`.
- [Thumb could steal clicks from rows or the sidebar resize handle] → Thumb lives in the reserved gutter with `block_mouse_except_scroll`, matching other panes. The resize handle sits on the sidebar's outer right border, outside the list overlay.
- [File-tree cap of 300 rows means the thumb tracks rendered rows, not the full scan] → Existing behavior (the "more hidden" footer is already inside the scroll). Do not expand the cap in this change.
- [Headless tests cannot judge drag feel] → Automated tests cover target identity, sync-driver no-op, hidden-when-fits, and proportional offset; a short manual checklist remains in tasks.

## Migration Plan

Pure additive chrome. No persistence, i18n, or API changes. Rollback is a plain revert.

## Open Questions

None. Overlay geometry, drag plumbing, and gutter constants already exist.
