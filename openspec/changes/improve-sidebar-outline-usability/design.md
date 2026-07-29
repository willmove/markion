## Context

`MarkionApp::render` currently stacks the menu bar, a full-width document-tab band, the main content row, and the status bar. The tab band reserves a left segment equal to the live sidebar width so document tabs align with the document panes; that segment is intentionally empty, and the actual sidebar begins one row lower. This produces the unused strip identified in the screenshot.

The Files panel already places its entries in a tracked `overflow_y_scroll` container. The Outline panel instead renders all heading rows directly into a flex column with `mb_1` and `py_1`, so its rows are relatively loose and the column has no scroll owner when headings overflow.

The rendering/state flow after this change remains presentation-only:

```text
menu bar
   |
   v
workspace row -------------------------------+
   |                                         |
   +-> full-height sidebar                   +-> document column
       Files / Outline tabs                      document tabs
       tracked panel scroll                      editor / preview panes

document version -> cached outline -> compact rows -> outline ScrollHandle
```

The document version, cached outline/preview/stat state, memoized highlighting, cached text handle, and bounded file-tree row collection are unaffected.

## Goals / Non-Goals

**Goals:**

- Make the visible sidebar begin directly below the menu and occupy the former empty tab-band segment.
- Keep document tabs connected to the document workspace and aligned to the resized sidebar boundary.
- Make outline rows substantially denser without removing hierarchy indentation or click targets.
- Allow wheel/trackpad scrolling to reach every outline heading.
- Preserve sidebar resizing, pane resizing, file-tree behavior, outline navigation, and theme-derived chrome.

**Non-Goals:**

- Do not add outline folding, virtualization, auto-reveal of the active heading, or preview-scroll-driven highlighting.
- Do not change document tabs, heading parsing, cache invalidation, persistence, or localization.

## Decisions

1. Reparent the sidebar and document controls into sibling workspace columns.

   The root stays a vertical menu/workspace/status stack. Inside the workspace row, the sidebar and its resize divider become the left column; a shrink-safe document column on the right contains the existing zero-or-30px document-tab band followed by the editor/preview content row. `tab_bar_view` no longer creates a sidebar-width leading spacer.

   This directly models the visual ownership requested by the user and makes the sidebar fill the reclaimed area at every live width. The alternative of painting sidebar-colored content over the existing spacer would preserve the duplicate layout regions and make sidebar header/body alignment and pointer behavior harder to maintain.

2. Scope drag geometry to the column each divider actually controls.

   Sidebar resize drag handling remains on the outer workspace row because its x-coordinate begins at the window's left edge. Editor/preview split drag handling moves to the inner document content row so its bounds exclude the sidebar and document-tab band. This preserves correct ratios after the hierarchy change and prevents the sidebar width from biasing the split calculation.

3. Use a persistent application-level `ScrollHandle` for the outline.

   Add one `outline_scroll` handle alongside `file_tree_scroll`, and bind it to an `overflow_y_scroll` outline container with a narrow scrollbar width. The handle survives ordinary rerenders, so wheel/trackpad position does not reset each frame. A single handle is sufficient because only the active document's outline is visible; GPUI clamps it when content or viewport bounds change.

   A newly-created handle inside `outline_panel_body` was rejected because it would lose scroll position on rerender. A virtualized list was also rejected because outline sizes are normally modest and virtualization is outside the requested scope.

4. Match the file tree's compact row rhythm.

   Outline rows remove their inter-row margin, use 1px vertical padding and a 17px line height, while keeping the existing 12px font, 12px-per-level indentation, rounded hover/active background, and full-row click handler. This yields a predictable approximately 19px single-line row without reducing text legibility.

## Risks / Trade-offs

- [Risk] Moving the sidebar above the former content-row boundary can change divider drag event bounds. → Mitigation: keep sidebar drag on the outer workspace row and move editor split drag to the document-only row, then verify both dividers manually.
- [Risk] A persistent outline offset may point past the end after switching to a shorter document. → Mitigation: rely on the tracked scroll container's bound clamping and test switching/rerendering with short and long outlines.
- [Risk] The sidebar header and document tab band can differ slightly in height when multiple tabs are visible. → Mitigation: both live in independent columns by design; verify the shared top edge and border continuity in light/dark themes and at representative scale factors.
- [Risk] A scrollable outline could accidentally trigger a fresh Markdown derivation on wheel events. → Mitigation: keep scrolling entirely in GPUI's `ScrollHandle`; the existing cached `document.outline()` data remains the only row source.

## Migration Plan

Apply the layout and state changes locally, add focused regression coverage, run formatting and the workspace test suite, then launch Markion with long/short outlines, one/multiple documents, visible/hidden/resized sidebar, and representative light/dark themes. No persisted-data migration is required. Rollback restores the full-width tab band and removes the outline scroll handle.

## Open Questions

None.
