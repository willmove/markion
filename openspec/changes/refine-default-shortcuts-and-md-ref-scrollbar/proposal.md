## Why

After the default-shortcut refresh, several high-traffic bindings still feel awkward: inline code occupies `Ctrl+E`, Visual Edit stays on `Ctrl+Alt+4`, Read mode needs Shift, and Open Folder has no factory shortcut. The Markdown Reference overlay also scrolls without a visible right-side scrollbar, so long cheat-sheet content is hard to discover.

## What Changes

- Move inline code to `Ctrl+Shift+\`` / `Cmd+Shift+\`` (action id `inline-code`).
- Move Visual Edit mode to `Ctrl+E` / `Cmd+E` (action id `set-visual-edit-mode`).
- Move Read mode to `Ctrl+R` / `Cmd+R` (action id `set-read-mode`).
- Give File → Open Folder a factory default of `Ctrl+Shift+O` / `Cmd+Shift+O` (new registry id `open-folder`), including keymap dispatch, in-window menu label, and shortcut catalog.
- Show a visible right-side vertical scrollbar on the Markdown Reference overlay body when content overflows (shared `pane_scrollbar_view` chrome, not OS-native-only).
- Update docs and the localized shortcut catalog to match.

## Capabilities

### New Capabilities

_(none)_

### Modified Capabilities

- `markdown-editing`: Update factory defaults for inline code, Visual Edit, and Read mode.
- `workspace`: Open Folder gains a factory keyboard shortcut via the shared registry.
- `chrome-platform`: Markdown Reference overlay body exposes a right-side scrollbar when overflowing.
- `ui-i18n`: Shortcut catalog lists Open Folder and the revised bindings.

## Impact

- Code: `menu_shortcuts` registry, `bind_app_keys`, File-menu shortcut label, shortcut catalog/`ShortcutLabels`, Markdown Reference view, `docs/keyboard-shortcuts.md`, `docs/faq.md`, related tests.
- Configuration: users with no overrides receive the new defaults; existing `[shortcuts]` overrides for these ids still win. New id `open-folder` becomes a valid override key.
- Non-goals: changing Edit/source, Split Preview, New Tab, Markdown Reference `F1`, or cycle-mode defaults; custom scrollbar drag chrome for the reference overlay (native overflow scrollbar width is enough).
