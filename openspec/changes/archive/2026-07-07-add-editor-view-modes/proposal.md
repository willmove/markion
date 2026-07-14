## Why

Markion currently centers the editing workflow on a split source-and-preview layout, which is useful while composing but unnecessarily busy when the user wants to focus only on writing or only on reading rendered Markdown. Adding explicit editor view modes gives users a fast, predictable way to move between focused editing, split preview, and reading without changing documents or recomputing derived Markdown state beyond the existing per-version caches.

Non-goals: this change does not introduce single-surface WYSIWYG editing, editable rendered preview blocks, or new Markdown parsing behavior.

## What Changes

- Add three editor view modes: Edit, Split Preview, and Read.
- Preserve the current split source-and-preview layout as one of the supported modes.
- Add menu/UI affordances for selecting the active mode.
- Add keyboard shortcuts for switching directly among the three modes, with platform-appropriate modifier conventions.
- Update the in-app shortcut reference and localized UI strings for the new mode controls.
- Ensure mode switches reuse the existing document, selection, scroll state, preview blocks, outline, stats, syntax highlighting, and cached text-handle invariants.

## Capabilities

### New Capabilities

- None.

### Modified Capabilities

- `markdown-editing`: Adds the editor view-mode contract and shortcuts for switching between Edit, Split Preview, and Read modes.
- `ui-i18n`: Adds localized UI chrome and shortcut-reference entries for the new view-mode controls.

## Impact

- Affected code likely includes `src/main.rs` for editor state, rendering, menus, and action handling; `src/i18n.rs` for translated labels and shortcut reference entries; and tests around shortcuts, menu labels, and mode-specific rendering.
- No new runtime dependencies are expected.
- The change touches rendering/layout behavior while preserving cached derived Markdown state per document version, memoized syntax highlighting, and cached text handles.
