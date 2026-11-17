## Why

The Visual Edit context menu uses compact grouped controls, but its Insert flyout still renders all 21 actions as full-width text rows. The flyout is unnecessarily tall and visually inconsistent with its parent, making common insertion commands harder to reach.

## What Changes

- Present paragraph and H1–H5 insertion as one six-button row, followed by a six-button row for bulleted/numbered/task lists, quote, code block, and divider.
- Present the five GFM callout insertion actions as a separate compact icon row with localized tooltips.
- Keep Table and Link/Formula/Image as grouped labeled insertion actions, consistent with the root menu's treatment of specialized actions.
- Reuse the root menu's button dimensions, spacing, theme colors, hover/keyboard-selected states, and tooltips. Heading insertion buttons must display distinct H1–H5 captions.
- Retain the complete insertion catalog, command order, captured targets, keyboard navigation, and viewport-constrained flyout behavior.
- **Non-goals:** adding/removing commands, changing root/table/image menus, changing insertion or undo semantics, or redesigning Markdown derivation.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `markdown-editing`: specify compact grouped controls for the existing context-menu Insert flyout.

## Impact

- `src/app/mod.rs`: insertion item grouping and compact-row classification shared with keyboard scroll positioning.
- `src/app/preview.rs`: render insertion headings with distinct H1–H5 captions using the existing compact button/panel components.
- `src/app/block_context_tests.rs`: extend existing rendered flyout coverage for compact layout, reachability, and dispatch.
- Presentation changes must preserve canonical source, selection, document version, undo history, dirty state, shared per-version derived caches, memoized highlighting, and cached text handles until a command is confirmed. Existing insertion mutations remain unchanged.
- No new dependencies, preference changes, persistence migration, or workspace-member changes are needed.
