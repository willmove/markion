# Design

## Context

The previous change moved one shared sidebar and retained mutual exclusion between Files and Outline. The clarified Right setting requires two simultaneous panes. Left placement must keep the existing shared tabs. Document derivation and outline folding already live outside the sidebar view and can be reused.

## Goals / Non-Goals

**Goals:** Provide Files | Document | Outline with independently controlled visibility and widths when Right is selected, while maintaining Left behavior and cached document state.

**Non-Goals:** Duplicate document models or outline caches, add arbitrary docking, or alter Markdown editing.

## Decisions

1. Keep `sidebar_visible`, `sidebar_tab`, and `sidebar_width` for the left column. In Right mode its content is always Files; the right pane only renders Outline and has no Files tab. In Left mode the existing two-tab header remains.
2. Add persisted `outline_visible` to preferences. Missing values inherit `sidebar_visible` so configurations from the first implementation show both panes when visible and keep both hidden when previously hidden. Selecting Right from a visible left sidebar opens both panes. Outline/File Tree actions toggle their own panes in Right mode; the master Sidebar action hides/shows both. Returning Left merges the surviving visible pane into the shared sidebar.
3. Store `outline_width` separately in session layout, defaulting to the same 230px and 150–480px bounds. Carry the pane identity in divider drag state so resizing and double-click reset affect only that pane, including when AI reserves space at the right of the workspace.
4. Reuse a common pane renderer with explicit content, side, visibility, and width. Render left sidebar, left divider, document column, right divider, right outline in that order. Do not reverse the workspace row. Construct outline contents only when the outline is visible.

Data flow:

```text
outline_position + sidebar_visible + outline_visible -> visible left/right panes
left pane -> Files (Right mode) / selected Files or Outline (Left mode)
right pane -> Outline only
divider pane identity + workspace bounds -> corresponding width -> session layout
preference actions -> visibility/position -> config.toml
```

The same per-tab heading cache, folding state, navigation handler, and outline scroll handle are reused in either position. Position/visibility/resize actions never mutate document text, version, selection, history, or cached Markdown data.

## Risks / Trade-offs

- [Two panes consume more horizontal space] → Keep existing bounded pane widths, a flexible document column, and independent hide controls.
- [Legacy tab checks incorrectly hide a left-side file prompt] → Derive file-panel visibility from the effective layout instead of `sidebar_tab` alone.
- [State changes could couple the panes] → Assert simultaneous concrete file rows and outline rows, independent action behavior, and independent divider widths in rendered GPUI tests.
