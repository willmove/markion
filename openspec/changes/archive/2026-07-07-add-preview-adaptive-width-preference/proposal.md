## Why

Read mode should feel like a Markdown reader rather than a stretched editing canvas: very wide lines reduce readability on large displays. A default capped reading width, with an opt-in adaptive-width preference, gives readable defaults while preserving the current full-width behavior for users who prefer it.

Non-goals: this change does not alter Markdown parsing, preview block generation, editor view-mode switching, export layout, theme colors, or document content.

## What Changes

- In Read mode, constrain the rendered preview content to a default maximum reading width of 860px and center it within the available preview pane.
- Add a Preferences panel setting named "Preview adaptive width" that is off by default.
- When "Preview adaptive width" is enabled, Read mode preview content uses the current full-width behavior instead of the 860px cap.
- Persist the new preference in the existing preferences file and restore it on launch, tolerating older files that omit the setting.
- Include the new preference in reset/summary behavior and localized UI chrome.

## Capabilities

### New Capabilities

- None.

### Modified Capabilities

- `chrome-platform`: Adds Read mode preview width behavior and a persisted preview adaptive-width preference.
- `theme-preferences`: Extends the Preferences panel/persistence surface with the new preview adaptive-width toggle.
- `ui-i18n`: Adds localized UI strings for the preview adaptive-width preference and related summary/status text.

## Impact

- Affected code likely includes `src/main.rs` for Read mode preview layout and the Preferences panel, `src/model.rs` and `src/storage/preferences.rs` for preference data/persistence, and `src/i18n.rs` for localized labels/status/summary text.
- No new dependencies are expected.
- The change touches layout and preferences only and must preserve cached derived Markdown state per document version, memoized highlighting, and cached text handles.
