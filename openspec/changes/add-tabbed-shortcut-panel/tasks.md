## 1. Structured Shortcut Catalog

- [x] 1.1 Introduce structured shortcut platform, category, section, action, and key-combination types that the UI can consume without parsing formatted text.
- [x] 1.2 Migrate every existing localized shortcut category/action and Windows/Linux/macOS key mapping into the structured catalog, including H1-H5 versus H1-H6 heading depth.
- [x] 1.3 Remove the Markdown/ASCII table formatter and keep tests that assert catalog completeness, explicit platform modifiers, and absence of `Secondary-*` display names.

## 2. Shortcut Panel State and Actions

- [x] 2.1 Add shortcut-panel open state, selected platform, and selected category to `MarkionApp`, initialized without persistence or document-state coupling.
- [x] 2.2 Change `ShowShortcuts` to open the in-app panel, default to the build target's platform and first category, close the active menu, and preserve status feedback.
- [x] 2.3 Add handlers for platform/category selection and dismissal through the close control and Escape without mutating document content, selection, undo history, or view mode.

## 3. Theme-Aware Panel UI

- [x] 3.1 Add a modal shortcut-panel view to the root overlay stack using the established Preferences scrim, occlusion, centered surface, title bar, and active theme palette patterns.
- [x] 3.2 Render a two-option Windows/Linux and macOS platform tab control and a localized File/Tabs/Editing/View/Search/Tables/Export category sidebar with clear active and hover states.
- [x] 3.3 Render the selected category as a bounded, scrollable action list with flexible localized action labels and non-shrinking, visually distinct shortcut key labels.
- [x] 3.4 Remove the native `window.prompt` shortcut-help path while leaving About and confirmation prompts unchanged and keeping Help -> Keyboard Shortcuts plus F1 wired to the panel.

## 4. Verification and Supersession

- [x] 4.1 Add or update tests for platform defaults, platform/category selection, all-language shortcut catalog coverage, sidebar versus Bold bindings, and removal of the plain-text prompt path.
- [ ] 4.2 Run `cargo fmt`, focused shortcut-panel tests, and `cargo test`; confirm the change does not touch Markdown-derived cache behavior.
- [ ] 4.3 Manually verify the panel in representative light and dark themes, at constrained window size, and with short- and long-label interface languages.
- [ ] 4.4 Run `openspec validate add-tabbed-shortcut-panel` and reconcile the superseded `clarify-keyboard-shortcuts` active change so its obsolete table requirement cannot later be synced into stable specs.
