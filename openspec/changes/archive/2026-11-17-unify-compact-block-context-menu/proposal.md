## Why

The Visual Edit right-click menu still uses a tall nested list while slash completion and grip menus use compact controls. Users need consistent presentation and more of the application's existing source-safe commands directly available on the clicked block.

## What Changes

- Share the slash palette's themed 248px panel and compact 36×30px controls, 2px gaps, localized tooltips, grouped rows, and viewport scrolling. Right-click/keyboard editing menus and grip block commands use distinct action catalogs and target scopes.
- Show Text/H1–H5 and list/quote/code/divider controls directly. Keep Table, Link, Formula, and Image settings reachable without text/heading or list submenus.
- Retain transforms, Duplicate, Move Up/Down, Add Below, explicit block Delete and image controls in the right-click/keyboard menu alongside selection Copy, Cut, Paste, Delete, Select All and inline formatting. Generic clipboard/deletion actions retain selection scope; explicit block actions target the clicked block. Add Strikethrough alongside Bold, Italic, Inline Code and Link.
- Capture separate block, text/content and table-cell targets. Preserve an active selection; with no selection, use the exact mouse location for pointer editing and the caret for keyboard invocation. Add table row/column insertion, movement, alignment, duplication, clearing and deletion for the clicked cell or keyboard-owned cell, with existing header/edge/version guards. Opening settings remains non-mutating; confirming uses existing editors, clipboard/resource policies, version checks, and undo.
- Preserve current-type indication, visibly disabled operations, selection preservation on right-click, grip preferences, keyboard reachability, tab isolation, and cached-per-version Markdown state.
- Close document menus on direct table-cell interaction and every document/tab transition, and reject stale menu state during rendering so New Tab and related shortcuts cannot crash on a previous document's target. Verify both reported failures and major editing flows in a native Windows test profile.

- Refine table-cell context menus to inline editing and explicit table operations: omit block conversion and block/image insertion, deduplicate Link, and expose Row/Column operations through compact second-level navigation. Support Shift+Enter as a cell-local `<br>` break. Diagnose and correct code background bounds after tab switches while preserving derived caches.

- Put paragraph/heading/block formatting and whole-block actions directly at the top of the editing context menu. Retain a bottom Insert flyout with new paragraphs, H1–H5, bulleted/numbered/task lists, quotes, code blocks, dividers, tables and all five supported GFM callout kinds, plus Link/Formula/Image settings. Block insertion adds a new source unit after the clicked complete block rather than converting or replacing it. Table context menus expose separate Row and Column flyouts with whole-table actions at the root. Image-size controls retain their hover flyout. Whole-unit labels identify their actual table/image/heading/paragraph scope.
- Keep Insert enabled on exactly mapped empty paragraphs, including empty documents, interior blank lines and EOF insertion rows. Reuse the clicked blank line for new block templates, independently of whole-block clipboard/delete availability; preserve exact inline settings targeting. Label empty-paragraph controls Format Block. Preserve a distinct editable blank body in newly inserted callouts, and direct normal title-label clicks to the existing body rather than the alert marker. Keyboard/source title editing remains available.
- Remove the image `</>` source toggle and source expansion from Visual Edit, including selected, hovered, failed and reference/data-URI images. Image Properties provides URL/alt/title editing through a bounded settings pane; resource replacement, sizing, alignment, upload and save remain explicit context commands. The whole-document Source mode remains available. Explicit property edits to a reference image may replace that occurrence with inline syntax, as existing resource replacement already does; no-op/cancel preserves reference syntax and other definitions.

Non-goals: browser search actions, deleting image files, implementing new external image-transfer/export engines, matching Typora's typography pixel-for-pixel, or extending Markion's existing document-column width limit beyond 100%.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `markdown-editing`: unify compact block context presentation, flatten type controls, expand source-safe editing/insertion actions, and retain exact target ownership.
- `tables-outline`: expose exact cell-context row/column operations and Strikethrough formatting in the combined context menu.
- `code-and-math`: keep code-block backgrounds within their document column after tab transitions.

## Impact

Application menu state/rendering/dispatch under `src/app/`, shared palette controls, existing localized labels in `src/i18n.rs`, embedded vector icons, and GPUI regressions. Integrates with completed but unarchived `compact-slash-command-panel` and `fix-block-handle-selection-and-menu` behavior. No new dependencies, persistence changes, or Markdown cache redesign; menu lifecycle uses existing shared derivations without reparsing on hover.
