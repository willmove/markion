## 1. Full-Height Sidebar Layout

- [x] 1.1 Recompose the GPUI workspace into a full-height sidebar column and an adjacent document column containing the existing conditional tab band and editor/preview content, removing the tab band's sidebar-width leading spacer.
- [x] 1.2 Scope sidebar and editor-split drag handlers to their respective outer-workspace and document-content bounds while preserving sidebar visibility, live resizing, all view modes, tab actions, and the zero-height single-tab rule.

## 2. Compact Scrollable Outline

- [x] 2.1 Add and initialize a persistent outline `ScrollHandle` without changing document versions, derived outline caching, or file-tree scroll state.
- [x] 2.2 Bind the outline rows to a bounded vertical scroll container with wheel/trackpad support and compact row metrics, preserving hierarchy indentation, active highlighting, hover feedback, and click navigation.

## 3. Verification

- [x] 3.1 Add focused regression coverage for the revised tab-band geometry and compact outline metrics where the GPUI test harness permits, while retaining existing tab, file-tree, and cache-invariant coverage.
- [x] 3.2 Run `cargo fmt --check`, `cargo test --workspace`, and `cargo build -p markion`, resolving regressions attributable to this change.
- [x] 3.3 Launch Markion and manually verify long/short outlines, mouse-wheel scrolling to the final heading, compact row spacing, one/multiple tabs, visible/hidden sidebar states, and representative light/dark themes; confirm retained sidebar and editor/preview resize-handler scope through focused regression coverage.
- [x] 3.4 Run `openspec validate improve-sidebar-outline-usability` and resolve all reported issues.
