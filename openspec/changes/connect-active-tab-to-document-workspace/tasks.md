## 1. Document-Aligned Tab Band

- [x] 1.1 Update the GPUI root layout so the multi-tab band reserves a theme-colored leading segment for the visible sidebar and its 1px separator, while the document-tab controls occupy the remaining document-workspace segment and follow `sidebar_width` during resizing.
- [x] 1.2 Preserve the existing menu, main-pane, status-bar, and floating-find geometry, and ensure the entire tab band (including the sidebar segment) contributes no height when only one tab is open.

## 2. Connected Tab Styling

- [x] 2.1 Restyle document tabs to align with the bottom of the band and use rounded upper corners with square lower corners, with inactive tabs retaining a complete visual boundary.
- [x] 2.2 Render the active tab with `surface_bg`, active theme text/accent chrome, and an open lower edge that covers the shared seam into the document workspace in Edit, Visual Edit, Split Preview, and Read modes.
- [x] 2.3 Preserve the dirty marker, close target, new-tab control, hover behavior, stale-index guard, tab switching, and keyboard actions without adding new theme values or touching per-tab document state.

## 3. Verification

- [x] 3.1 Add focused regression coverage for any extracted tab-band visibility or sidebar-alignment geometry helpers, and rely on the existing tab interaction tests to guard state behavior without introducing brittle style-source assertions.
- [x] 3.2 Run `cargo fmt --check` and `cargo test --workspace`, resolving regressions attributable to this change while preserving the document-version cache, highlighting, cached text-handle, and per-tab scroll invariants.
- [x] 3.3 Build and launch Markion, then verify two or more tabs with the sidebar hidden, visible, and resized across Edit, Visual Edit, Split Preview, and Read modes under representative light and dark themes; also check a narrow window and representative Windows scale factors for seam artifacts.
- [x] 3.4 Run `openspec validate connect-active-tab-to-document-workspace` and resolve all reported issues.
