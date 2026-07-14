## MODIFIED Requirements

### Requirement: The interface language SHALL be selectable and shall persist
The system SHALL let the user choose an interface language from the set of supported languages (`Language::all()`) via the Preferences panel. The chosen language SHALL persist across launches in the preferences file as a stable lowercase code (`language=<code>`). The supported set includes English, Simplified Chinese (`zh-hans`), Traditional Chinese (`zh-hant`), German, Spanish, French, and Japanese; the two Chinese entries appear adjacent in the language picker and each is labelled with its own native-script name (`简体中文` / `繁體中文`).

#### Scenario: Unknown or missing language code falls back to English
- **WHEN** the preferences file has no `language=` line, an empty value, or an unrecognized code (e.g. `language=klingon`)
- **THEN** the system sets the interface language to English (`Language::default`) without error

#### Scenario: Common Simplified-Chinese aliases are accepted
- **WHEN** the persisted language code is `zh`, `chs`, `zh-cn`, `zh-hans`, or `chinese` (case-insensitive)
- **THEN** the system selects Simplified Chinese as the interface language

#### Scenario: Common Traditional-Chinese aliases are accepted
- **WHEN** the persisted language code is `zh-hant`, `zh-tw`, `zh-hk`, `cht`, or `traditional chinese` (case-insensitive)
- **THEN** the system selects Traditional Chinese as the interface language

#### Scenario: Switching language takes effect immediately for menus
- **WHEN** the user selects a different language in the Preferences panel
- **THEN** the active language is updated in app state, preferences are persisted, and native OS menus are retranslated and reinstalled before the next render

### Requirement: Adding a UI string without a translation SHALL fail at compile time
The i18n module SHALL expose a closed `Msg` enum where each variant is a distinct user-visible string. The translation functions `t` and `tf` SHALL be exhaustive over `Msg` for every supported `Language`, so that adding a message key without providing translations for all languages is a compile error rather than a runtime fallback. Adding a new `Language` variant SHALL likewise be a compile error until every dispatch site (`t`, `tf`, `shortcut_reference`, `sidebar_tab_label`, and the in-app menu layout table) covers it.

#### Scenario: New message key requires translations in every language
- **WHEN** a developer adds a new variant to `Msg`
- **THEN** the project fails to compile until `en()`, `zh_hans()`, and `zh_hant()` (and every other language table) all cover the new variant (exhaustive `match`)

#### Scenario: Every message returns non-empty text for every language
- **WHEN** the i18n test suite runs the exhaustiveness guard
- **THEN** every `Msg` variant returns non-empty text for English, Simplified Chinese, and Traditional Chinese
