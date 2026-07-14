## 1. Mode Model and Actions

- [x] 1.1 Rename or map the existing view-mode variants to user-facing Edit, Split Preview, and Read semantics while preserving Split Preview as the default.
- [x] 1.2 Add direct GPUI actions for setting Edit, Split Preview, and Read mode, keeping the existing cycle action as a fallback.
- [x] 1.3 Wire direct mode actions into the root action handlers without mutating document text, tab state, undo/redo history, scroll handles, or derived Markdown caches.

## 2. Menus, Shortcuts, and Localization

- [x] 2.1 Add View menu entries for Edit, Split Preview, and Read in both native menus and the in-app dropdown.
- [x] 2.2 Bind direct mode shortcuts as `Secondary-Alt-1`, `Secondary-Alt-2`, and `Secondary-Alt-3`.
- [x] 2.3 Add translated i18n messages for mode menu labels and status feedback in English and Simplified Chinese.
- [x] 2.4 Update the keyboard shortcut reference in English and Simplified Chinese to list the direct mode shortcuts.

## 3. Rendering and State Preservation

- [x] 3.1 Ensure Edit mode renders only the source editing pane and Read mode renders only the preview pane.
- [x] 3.2 Ensure Split Preview mode renders both panes and the split resize handle exactly when both panes are visible.
- [x] 3.3 Verify mode switches preserve active tab identity, document text, dirty state, cursor/selection, scroll handles, undo/redo stacks, and cached derived Markdown behavior.

## 4. Tests and Validation

- [x] 4.1 Add or update unit tests for `ViewMode` cycling/direct selection behavior and default mode.
- [x] 4.2 Add shortcut-reference/i18n tests covering the new mode labels and direct shortcuts in both supported languages.
- [x] 4.3 Run `cargo test` for the root package.
- [x] 4.4 Run `openspec validate add-editor-view-modes`.
