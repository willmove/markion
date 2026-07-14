# ui-i18n

## Purpose

Covers runtime translation of user-visible UI chrome across a fixed set of interface languages. The layer is dependency-free: a compile-time-checked `Msg` enum (one variant per user-visible string) with exhaustive per-language `match` arms, so a missing translation site is a compile error rather than a runtime fallback. Adding a language is a `match`-arm extension. Document content, the welcome Markdown, and user files are never translated — only UI chrome.

This capability covers the language-selection surface reachable through the **Preferences panel**. A View → Language menu submenu is **not** implemented; it is a future-change candidate.
## Requirements
### Requirement: All user-visible UI chrome SHALL be translated through the i18n layer
The system SHALL route every user-visible UI string (menu bar titles and items, in-app dropdown labels, status bar text, dialog text, search panel labels, file tree labels, preferences panel labels, and the keyboard-shortcut reference) through the i18n module's `t` / `tf` / `shortcut_reference` / `sidebar_tab_label` functions. Hard-coded user-visible English literals in these surfaces SHALL NOT remain.

#### Scenario: Menu labels reflect the active language
- **WHEN** the active interface language is Simplified Chinese
- **THEN** the native OS menu bar and the in-app dropdown render every menu title and item label in Simplified Chinese via `t(language, Msg::…)`

#### Scenario: Templatized status text interpolates in the active language
- **WHEN** the editor produces a dynamic status message (e.g. word count, save path)
- **THEN** the status bar text is produced by `tf(language, msg, args)` and rendered in the active language with positional arguments substituted

#### Scenario: Document content is never translated
- **WHEN** the active language is Simplified Chinese
- **THEN** document content, the welcome Markdown, and user files remain untouched (only UI chrome is translated)

### Requirement: The interface language SHALL be selectable and shall persist
The system SHALL let the user choose an interface language from the set of supported languages (`Language::all()`) via the Preferences panel. The chosen language SHALL persist across launches in the preferences file as a stable lowercase code (`language=<code>`).

#### Scenario: Unknown or missing language code falls back to English
- **WHEN** the preferences file has no `language=` line, an empty value, or an unrecognized code (e.g. `language=klingon`)
- **THEN** the system sets the interface language to English (`Language::default`) without error

#### Scenario: Common Chinese aliases are accepted
- **WHEN** the persisted language code is `zh`, `chs`, `zh-cn`, `zh-hans`, or `chinese` (case-insensitive)
- **THEN** the system selects Simplified Chinese as the interface language

#### Scenario: Switching language takes effect immediately for menus
- **WHEN** the user selects a different language in the Preferences panel
- **THEN** the active language is updated in app state, preferences are persisted, and native OS menus are retranslated and reinstalled before the next render

### Requirement: Adding a UI string without a translation SHALL fail at compile time
The i18n module SHALL expose a closed `Msg` enum where each variant is a distinct user-visible string. The translation functions `t` and `tf` SHALL be exhaustive over `Msg` for every supported `Language`, so that adding a message key without providing translations for all languages is a compile error rather than a runtime fallback.

#### Scenario: New message key requires translations in every language
- **WHEN** a developer adds a new variant to `Msg`
- **THEN** the project fails to compile until `en()` and `zh()` both cover the new variant (exhaustive `match`)

#### Scenario: Every message returns non-empty text for every language
- **WHEN** the i18n test suite runs the exhaustiveness guard
- **THEN** every `Msg` variant returns non-empty text for both English and Simplified Chinese

### Requirement: View mode UI chrome SHALL be localized
The system SHALL route all user-visible UI strings for Edit, Split Preview, and Read mode controls through the i18n layer, including native menu items, in-app menu items, status feedback, and keyboard shortcut reference text.

#### Scenario: View mode menu labels reflect the active language
- **WHEN** the active interface language changes
- **THEN** the native View menu and in-app View dropdown render the Edit, Split Preview, and Read mode entries in the active language

#### Scenario: View mode status feedback reflects the active language
- **WHEN** the user switches to Edit, Split Preview, or Read mode
- **THEN** the status bar message naming the active mode is produced through the active language translation

#### Scenario: Shortcut reference lists direct mode shortcuts
- **WHEN** the user opens the keyboard shortcut reference from the Help menu
- **THEN** the reference lists the direct shortcuts for Edit, Split Preview, and Read mode in the active language

### Requirement: Preview adaptive width UI chrome SHALL be localized
The system SHALL route every user-visible string for the Preview adaptive width preference through the i18n layer, including the Preferences panel label, preferences summary text, and any related status feedback.

#### Scenario: Preferences panel label reflects active language
- **WHEN** the active interface language changes
- **THEN** the Preview adaptive width label in the Preferences panel renders in the active language

#### Scenario: Preferences summary reflects active language
- **WHEN** the user opens preferences summary text
- **THEN** the Preview adaptive width value is displayed through localized text
