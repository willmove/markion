## Why

The native keyboard-shortcut prompt only renders plain text, so Markdown tables appear as source and ASCII tables remain visually noisy and fragile across localized labels. A dedicated in-app shortcut panel can present the same reference as a clear, theme-aware interface with platform switching and category navigation.

## What Changes

- Replace the Help -> Keyboard Shortcuts native prompt with an in-app modal panel.
- Add Windows/Linux and macOS platform tabs, defaulting to the platform that matches the running application.
- Present shortcut actions as categorized, scrollable lists with a category sidebar and visually distinct key labels instead of table syntax or simulated text columns.
- Keep shortcut section names, action labels, panel controls, and status feedback localized in every supported interface language.
- Reuse one structured shortcut-reference model as the source for both platform views so displayed bindings do not drift between layouts.
- Preserve the existing Ctrl+Shift+B / Cmd+Shift+B sidebar shortcut introduced by `clarify-keyboard-shortcuts`.
- Supersede the unarchived `clarify-keyboard-shortcuts` change by carrying forward its valid binding behavior while replacing its prompt/table presentation requirements.
- Non-goals: user-configurable keybindings, keybinding conflict detection, shortcut search, or changes to editor command behavior.

## Capabilities

### New Capabilities

- None.

### Modified Capabilities

- `ui-i18n`: The localized keyboard-shortcut reference changes from prompt-formatted text to a localized, platform-tabbed in-app panel.
- `chrome-platform`: Help -> Keyboard Shortcuts opens a theme-aware modal with platform tabs, category navigation, scrolling, and explicit dismissal.

## Impact

- Affected code: shortcut-reference data and translations in `src/i18n.rs`; shortcut panel state/actions in `src/app`; modal rendering in `src/app/root_view.rs`; existing shortcut-reference tests.
- The native `window.prompt` path for shortcut help will be removed, while other informational and confirmation prompts remain unchanged.
- The overlapping `clarify-keyboard-shortcuts` change must not later sync its obsolete text-table requirement into the stable specs.
- No new dependencies, persistence format, or public API changes are expected.
- Markdown parsing and the per-document derived-state caches are not touched.
