## 1. Preference Model and Persistence

- [x] 1.1 Add a `preview_adaptive_width` boolean to app preferences with default `false`.
- [x] 1.2 Parse and render the new preference in the TOML preferences file while tolerating older files that omit it.
- [x] 1.3 Include the setting in reset behavior and preferences summary data.
- [x] 1.4 Add or update preference round-trip/default tests for the new field.

## 2. Preferences Panel and Localization

- [x] 2.1 Add a Preview adaptive width toggle to the Preferences panel's display/other settings.
- [x] 2.2 Apply the toggle immediately, persist the setting, and refresh Read mode layout on the next render.
- [x] 2.3 Add English and Simplified Chinese i18n strings for the label, status/summary text, and exhaustiveness tests.

## 3. Read Mode Layout

- [x] 3.1 Add a constant for the default Read mode preview max width of 860px.
- [x] 3.2 In Read mode with Preview adaptive width disabled, center preview content and constrain it to 860px max width.
- [x] 3.3 In Read mode with Preview adaptive width enabled, keep full-width preview behavior.
- [x] 3.4 Confirm Split Preview mode is unaffected by the preference.

## 4. Verification

- [x] 4.1 Add focused tests for preference defaults/round-trip and any layout decision helpers introduced.
- [x] 4.2 Run `cargo test`.
- [x] 4.3 Run `openspec validate add-preview-adaptive-width-preference`.
