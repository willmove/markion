## Context

`MarkionApp::render` currently stacks the menu bar, a full-window tab bar, the main content row, and the status bar. The tab bar is built by `tab_bar_view` in `src/app/editing.rs`; it uses a centered 30px strip whose tabs have four rounded corners, an active-colored fill, and a lower accent border. The main content row in `src/app/root_view.rs` begins with the optional fixed-width sidebar and its 1px resize divider, followed by the source/visual/preview workspace. Consequently, a tab begins above the sidebar when it is visible, and its fill and lower border remain visually separate from the document surfaces.

The recently established square-corner document surfaces use `palette.surface_bg` with the existing pane borders and zero outer padding. This gives the active tab a stable surface color and boundary to connect to without changing document rendering or theme definitions.

Rendering data flow remains presentation-only:

```text
tabs + active_tab + palette + sidebar visibility/width
                         |
                         v
              document-aligned tab band
                         |
                         v
          existing editor / visual / preview panes

document version -> cached derived Markdown state -> pane children (unchanged)
```

## Goals / Non-Goals

**Goals:**

- Align document tabs horizontally with the document workspace instead of the sidebar.
- Make the active tab read as the open upper edge of the shared document workspace.
- Keep the treatment consistent in Edit, Visual Edit, Split Preview, and Read modes and across built-in and custom themes.
- Preserve the existing vertical layout, sidebar resizing, tab actions, find overlay positioning, and zero-height single-tab behavior.

**Non-Goals:**

- Do not change the tab state model, session persistence, ordering, dirty tracking, close behavior, or keyboard navigation.
- Do not add tab overflow menus, scrolling, drag reordering, per-tab view modes, or new theme tokens.
- Do not modify pane contents, Markdown derivation, cached text handles, highlighting, scrolling state, or input handling.

## Decisions

1. Keep the existing vertical render order and split the tab band into sidebar and document segments.

   When multiple tabs are visible, the tab-band row will reserve a leading segment equal to the visible sidebar width plus its separator, then render the tab controls in a flexible document segment. When the sidebar is hidden, the document segment occupies the full width. The leading segment uses existing panel and border colors and contains no document-tab controls.

   Rationale: this aligns the tabs with the pane boundary while leaving the menu, content row, status bar, search overlay, and pane heights in their current positions. It also naturally follows the existing `sidebar_width` during resize.

   Alternative considered: reparent the sidebar and document workspace into separate full-height columns and put the tab bar inside the document column. That expresses the hierarchy more literally, but it moves the sidebar upward into the former tab row and requires broader changes to resize and overlay geometry for the same visual result.

2. Treat the active tab as the open top edge of the document surface.

   Tabs will align to the bottom of the 30px band and use rounded top corners with square lower corners. The active tab uses `palette.surface_bg`, active text/accent color, left/top/right boundary chrome, and no visible lower boundary. Its fill covers or overlaps the shared 1px seam so the surface continues into the workspace. Active emphasis moves to the upper edge rather than drawing an underline between the tab and its content.

   Inactive tabs retain a visible lower boundary and subdued theme colors, so only one tab appears connected. The `+` control remains a separate action rather than looking like a document surface.

   Rationale: matching the surface fill and removing the active lower seam creates the requested browser-style continuity without changing pane colors or adding theme fields.

   Alternative considered: only replace `rounded_md` with a top-corner radius. This leaves the centered gap, active fill mismatch, and bottom accent intact, so it does not establish ownership.

3. Connect the tab to the shared document workspace, not an individual pane.

   The seam belongs to the workspace segment that contains whichever surfaces the current view mode selects. In Split Preview the one active tab therefore owns both source and preview panes; in the other modes it owns the single source, visual, or read surface.

   Rationale: attaching the tab only to the left pane would falsely imply that the preview is a separate document.

4. Preserve the single-tab and interaction contracts.

   The entire tab band, including the sidebar spacer, remains absent when one tab is open. Existing click, close, dirty marker, hover, stale-index guard, new-tab, and keyboard behavior are retained. Styling and layout consume only existing `ThemePalette` values.

   Rationale: this change is a chrome clarification, not a tab-model change.

## Risks / Trade-offs

- [Risk] Reserving the sidebar-aligned segment reduces the horizontal width available to tabs when the sidebar is open. → Mitigation: preserve the current tab sizing/overflow behavior, keep the document segment shrink-safe, and verify representative multi-tab and narrow-window cases; overflow navigation remains a separate change.
- [Risk] A 1px border can remain visible under the active tab on some scale factors. → Mitigation: use one explicit seam-owning layer and verify at representative Windows scale factors and under both light and dark themes.
- [Risk] Sidebar resize can briefly misalign the tab band and pane boundary if they use different width calculations. → Mitigation: derive both from the same `sidebar_width` state and the existing 1px divider width in the same render pass.
- [Risk] View modes have separate pane builders and could expose inconsistent upper chrome. → Mitigation: connect the tab to the shared workspace boundary outside the mode-specific pane builders and manually exercise all four modes.

## Migration Plan

Apply the local GPUI layout and styling changes, run formatting and the workspace test suite, then manually verify multi-tab rendering with the sidebar both visible and hidden across all view modes and representative light/dark themes. No data or preference migration is required. Rollback restores the former full-width tab bar call and pill styling.

## Open Questions

None.

