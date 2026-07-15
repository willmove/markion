## Why

The current Find / Replace panel is an embedded full-width row between the tab bar and editor content, so opening search shifts the workspace and reads unlike the compact overlay used by mainstream editors. It also uses hard-coded light colors, which makes it visually inconsistent under dark and custom themes.

## What Changes

- Render Find / Replace as a compact floating bar positioned above the editor/preview workspace near the upper-right corner, without consuming layout height or pushing content down.
- Add an explicit close control for the floating bar. Closing the bar clears active match highlighting and focus state while preserving the current query and replacement text for the next open.
- Theme the floating bar, inputs, buttons, borders, hover states, and summary text from the active `ThemePalette` instead of hard-coded light colors.
- Preserve the existing Find and Replace workflows: shortcuts, query editing, case-sensitive and regex toggles, next/previous navigation, match counts, replace current, and replace all.
- Non-goals: Do not change the search algorithm, regex behavior, keyboard shortcuts, persistence model, or introduce a new configurable search preference.

## Capabilities

### New Capabilities

- None.

### Modified Capabilities

- `chrome-platform`: Find / Replace changes from an embedded row to a floating, closable, theme-aware overlay while preserving existing search and replace behavior.

## Impact

- Affected code: `src/main.rs` search panel rendering, close handling, Escape/close focus cleanup, and search toolbar/button styling.
- Affected specs: `chrome-platform`.
- No new dependencies or persistence migrations are expected.
- UI chrome must continue to use the active i18n labels already exposed by `src/i18n.rs`; a symbol-only close control should avoid adding a new translation key unless the implementation adds visible text.
- The derived Markdown cache, syntax highlighting cache, and cached text handle invariants are not touched because the change is limited to application chrome and search UI state.
