## Why

The Format menu's list, blockquote, and fenced-code actions are available only through pointer navigation even though they are frequent structural-editing commands. Adding the established shortcuts shown in the reference screenshot makes these commands faster to invoke and discoverable through Markion's existing menu and shortcut-preferences surfaces.

## What Changes

- Add default platform-aware shortcuts for five existing Format actions: Ordered List (`Ctrl/Cmd+Shift+[`), Unordered List (`Ctrl/Cmd+Shift+]`), Task List (`Ctrl/Cmd+Shift+X`), Blockquote (`Ctrl/Cmd+Shift+Q`), and Code Fence (`Ctrl/Cmd+Shift+K`).
- Register the five actions in the existing customizable menu-shortcut registry so dispatch, in-window menu labels, shortcut-reference rows, override persistence, conflict detection, and live rebinding all use the same effective bindings.
- Show the effective shortcut beside each of the five Format menu items and include each action in the localized Editing shortcut category.
- Add regression coverage for registry completeness, platform labels, key dispatch, Format-menu metadata, shortcut-catalog exposure, and customization behavior.
- Non-goals: adding an equation-block command, changing the existing table-format shortcut, changing any formatting action's Markdown transformation semantics, or touching Markdown parsing and per-document derived-state caches.

## Capabilities

### New Capabilities

- None.

### Modified Capabilities

- `markdown-editing`: Define the default bindings and discoverability contract for the existing ordered-list, unordered-list, task-list, blockquote, and fenced-code Format actions.

## Impact

- Affected code: `src/app/mod.rs` (shortcut registry), `src/app/bootstrap.rs` (keymap), `src/app/root_view.rs` (Format menu shortcut labels), `src/i18n.rs` (localized shortcut catalog), and `src/app/tests.rs` / shortcut tests.
- Configuration: the five stable action ids become valid `[shortcuts]` override keys under the existing preferences model; no persistence schema change is required.
- Dependencies and APIs: no new dependencies or public API changes.
- Architecture: formatting handlers remain unchanged, and the document-version cache, shared derived Markdown state, syntax-highlight memoization, and cached text handle are untouched.
