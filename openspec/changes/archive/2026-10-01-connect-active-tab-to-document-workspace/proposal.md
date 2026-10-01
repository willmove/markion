## Why

The current document tabs render as independently rounded pills above the entire main row, so the active tab remains visually detached from its editor, preview, or reading surface and can appear to point at the sidebar when the sidebar is visible. Connecting the active tab to the document workspace will make document ownership immediately legible while preserving the compact, theme-aware chrome.

## What Changes

- Place the multi-document tab bar within the document-workspace column rather than across the sidebar and document panes together.
- Render document tabs with rounded top corners and square lower corners; make the active tab share the document-surface background and visually open into the workspace below, while inactive tabs remain clearly separated.
- Move active emphasis away from the lower seam so no border or accent line visually cuts the active tab off from its content.
- Preserve tab switching, closing, dirty markers, the new-tab control, sidebar resizing, all four view modes, and the single-tab rule that hides the tab bar without consuming layout space.
- Keep tab and workspace chrome derived from the active theme across light, dark, and custom themes.

Non-goals: this change does not add tab persistence, reordering, overflow navigation, per-tab view modes, new theme tokens, or any document/editing behavior.

## Capabilities

### New Capabilities

- None.

### Modified Capabilities

- `chrome-platform`: Define the visual and layout relationship between the active document tab, the sidebar, and the shared document workspace across all view modes.

## Impact

- Affected GPUI layout and styling are expected in `src/app/root_view.rs` and `src/app/editing.rs`, with focused regression coverage in `src/app/tests.rs` where practical.
- No public APIs, persisted data, localization strings, dependencies, or theme-file fields change.
- Per-tab document state, scroll handles, derived Markdown caches, memoized highlighting, cached text handles, and input behavior remain untouched.
