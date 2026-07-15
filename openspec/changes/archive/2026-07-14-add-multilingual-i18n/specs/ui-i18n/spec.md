## MODIFIED Requirements

### Requirement: All user-visible UI chrome SHALL be translated through the i18n layer
The system SHALL route every user-visible UI string (menu bar titles and items, in-app dropdown labels, status bar text, dialog text, search panel labels, file tree labels, file tree context-menu labels, the file-tree create/rename inline name-prompt label and placeholder, the recursive-folder-delete confirm dialog title and detail, preferences panel labels, and the keyboard-shortcut reference) through the i18n module's `t` / `tf` / `shortcut_reference` / `sidebar_tab_label` functions. Hard-coded user-visible English literals in these surfaces SHALL NOT remain.

#### Scenario: Menu labels reflect the active language
- **WHEN** the active interface language is Japanese
- **THEN** the native OS menu bar and the in-app dropdown render every menu title and item label in Japanese via `t(language, Msg::…)`

#### Scenario: Menu labels reflect French
- **WHEN** the active interface language is French
- **THEN** the native OS menu bar and the in-app dropdown render every menu title and item label in French via `t(language, Msg::…)`

#### Scenario: Menu labels reflect German
- **WHEN** the active interface language is German
- **THEN** the native OS menu bar and the in-app dropdown render every menu title and item label in German via `t(language, Msg::…)`

#### Scenario: Menu labels reflect Spanish
- **WHEN** the active interface language is Spanish
- **THEN** the native OS menu bar and the in-app dropdown render every menu title and item label in Spanish via `t(language, Msg::…)`

#### Scenario: File tree context menu labels reflect the active language
- **WHEN** the active interface language changes
- **THEN** the file tree context menu renders every action label and related status message in the active language through the i18n layer

#### Scenario: File tree name prompt is localized
- **WHEN** the user invokes Create File, Create Folder, or Rename and the inline name prompt is shown
- **THEN** the prompt label (e.g. "Name"), the pre-filled default name, the empty-name warning status text, and the commit/cancel behavior are presented in the active language through the i18n layer

#### Scenario: Recursive folder delete confirmation is localized
- **WHEN** the user deletes a non-empty folder and the second confirmation dialog is shown
- **THEN** the dialog title and detail text are produced in the active language through the i18n layer, and the confirm/cancel button labels reuse the existing localized delete/cancel strings

#### Scenario: Templatized status text interpolates in the active language
- **WHEN** the editor produces a dynamic status message (e.g. word count, save path, created/renamed/deleted path)
- **THEN** the status bar text is produced by `tf(language, msg, args)` and rendered in the active language with positional arguments substituted

#### Scenario: Document content is never translated
- **WHEN** the active language is Japanese, French, German, or Spanish
- **THEN** document content, the welcome Markdown, and user files remain untouched (only UI chrome is translated)

### Requirement: The interface language SHALL be selectable and shall persist
The system SHALL let the user choose an interface language from the set of supported languages (`Language::all()`) via the Preferences panel. The chosen language SHALL persist across launches in the preferences file as a stable lowercase code (`language=<code>`).

#### Scenario: Unknown or missing language code falls back to English
- **WHEN** the preferences file has no `language=` line, an empty value, or an unrecognized code (e.g. `language=klingon`)
- **THEN** the system sets the interface language to English (`Language::default`) without error

#### Scenario: Common Chinese aliases are accepted
- **WHEN** the persisted language code is `zh`, `chs`, `zh-cn`, `zh-hans`, or `chinese` (case-insensitive)
- **THEN** the system selects Simplified Chinese as the interface language

#### Scenario: Common Japanese aliases are accepted
- **WHEN** the persisted language code is `ja`, `jp`, `japanese`, or `jpn` (case-insensitive)
- **THEN** the system selects Japanese as the interface language

#### Scenario: Common French aliases are accepted
- **WHEN** the persisted language code is `fr`, `francais`, `français`, `french`, or `fra` (case-insensitive)
- **THEN** the system selects French as the interface language

#### Scenario: Common German aliases are accepted
- **WHEN** the persisted language code is `de`, `deutsch`, `german`, `ger`, or `deu` (case-insensitive)
- **THEN** the system selects German as the interface language

#### Scenario: Common Spanish aliases are accepted
- **WHEN** the persisted language code is `es`, `espanol`, `español`, `spanish`, or `spa` (case-insensitive)
- **THEN** the system selects Spanish as the interface language

#### Scenario: Switching language takes effect immediately for menus
- **WHEN** the user selects a different language in the Preferences panel
- **THEN** the active language is updated in app state, preferences are persisted, and native OS menus are retranslated and reinstalled before the next render

### Requirement: Adding a UI string without a translation SHALL fail at compile time
The i18n module SHALL expose a closed `Msg` enum where each variant is a distinct user-visible string. The translation functions `t` and `tf` SHALL be exhaustive over `Msg` for every supported `Language`, so that adding a message key without providing translations for all languages is a compile error rather than a runtime fallback.

#### Scenario: New message key requires translations in every language
- **WHEN** a developer adds a new variant to `Msg`
- **THEN** the project fails to compile until `en()`, `zh()`, `ja()`, `fr()`, `de()`, and `es()` all cover the new variant (exhaustive `match`)

#### Scenario: Every message returns non-empty text for every language
- **WHEN** the i18n test suite runs the exhaustiveness guard
- **THEN** every `Msg` variant returns non-empty text for English, Simplified Chinese, Japanese, French, German, and Spanish
