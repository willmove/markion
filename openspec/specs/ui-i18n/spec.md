# ui-i18n

## Purpose

Covers runtime translation of user-visible UI chrome across a fixed set of interface languages. The layer is dependency-free: a compile-time-checked `Msg` enum (one variant per user-visible string) with exhaustive per-language `match` arms, so a missing translation site is a compile error rather than a runtime fallback. Adding a language is a `match`-arm extension. Document content, the welcome Markdown, and user files are never translated — only UI chrome.

This capability covers the language-selection surface reachable through the **Preferences panel**. A View → Language menu submenu is **not** implemented; it is a future-change candidate.
## Requirements
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
- **THEN** the prompt label (e.g. "Name"), the pre-filled default name, the empty-name warning status text, the commit/cancel behavior, and the caret/selection/click-away behavior are presented in the active language through the i18n layer

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

### Requirement: Preview context menu strings are localized
The system SHALL route all user-visible labels and status feedback for the preview pane context menu (Copy as Plain Text, Copy as Markdown, Copy as HTML, Select All, Copy Link Address, and related status messages) through the i18n `Msg` / `t` / `tf` layer for every supported interface language.

#### Scenario: Preview context menu labels follow the active language
- **WHEN** the active interface language is Simplified Chinese and the preview context menu is open
- **THEN** every menu item label is produced via `t(language, Msg::…)` in Simplified Chinese

#### Scenario: Preview copy status feedback is localized
- **WHEN** the user copies preview content as plain text, Markdown, or HTML from the context menu
- **THEN** the status bar message is produced through the active language translation

### Requirement: Open Folder UI chrome SHALL be localized
The system SHALL route every user-visible Open Folder string through the i18n layer, including the native and in-window File-menu labels, directory-picker prompt, and opening, success, cancellation, and failure status feedback.

#### Scenario: Open Folder menu label reflects the active language
- **WHEN** the active interface language changes
- **THEN** both the native File menu and the in-window File dropdown render Open Folder in the active language via `t(language, Msg::…)`

#### Scenario: Folder picker prompt reflects the active language
- **WHEN** the user invokes Open Folder
- **THEN** the directory-picker prompt is produced through the active language translation

#### Scenario: Folder selection feedback reflects the active language
- **WHEN** folder selection starts, succeeds, is canceled, or fails
- **THEN** the corresponding status text is produced through `t` or `tf` in the active language

### Requirement: Sync scroll UI chrome SHALL be localized
The system SHALL route every user-visible string for the Sync scroll preference through the i18n layer, including the Preferences panel label, the on/off status feedback when the preference is toggled, and any related preferences summary text. Adding the new message keys SHALL require translations in every supported language or the build SHALL fail.

#### Scenario: Preferences panel label reflects active language
- **WHEN** the active interface language changes
- **THEN** the Sync scroll label in the Preferences panel renders in the active language

#### Scenario: Toggle status feedback reflects active language
- **WHEN** the user toggles Sync scroll
- **THEN** the status bar message indicating Sync scroll on or off is produced through the active language translation

#### Scenario: New message keys require all-language translations
- **WHEN** a developer adds the Sync scroll message variants to `Msg`
- **THEN** the project fails to compile until both English and Simplified Chinese cover them via the exhaustive `match`

### Requirement: Heading menu depth UI chrome SHALL be localized
The system SHALL route every user-visible string for Heading menu depth and H4–H6 Format menu entries through the i18n layer, including menu item labels, Preferences panel control labels, and keyboard shortcut reference entries when H1–H6 depth is active.

#### Scenario: H4–H6 menu labels reflect active language
- **WHEN** Heading menu depth is H1–H6 and the active interface language is Simplified Chinese
- **THEN** H4, H5, and H6 Format menu items render in Simplified Chinese

#### Scenario: Preferences panel label reflects active language
- **WHEN** the active interface language changes
- **THEN** the Heading menu depth control label and option text render in the active language

### Requirement: P0 authoring and conflict workflows SHALL be localized
Every user-visible label, status, prompt, validation error, missing-resource message, contextual-control tooltip, link/image editor field, and external-change/recovery action introduced by this change SHALL be routed through the localization catalog for every supported language.

#### Scenario: P0 workflows render in each supported language
- **WHEN** the application language changes while a resource, link-editor, missing-resource, external-change, or recovery surface is visible
- **THEN** all user-facing text on that surface uses the selected language
- **AND** no implementation-only English literal is presented as UI text

### Requirement: P1 recovery and block-authoring workflows SHALL be localized
Every user-visible recovery-manager and block-authoring string SHALL be provided for every supported language, including recovery states and bulk/per-entry actions, slash command names and empty results, block transform/duplicate/delete/move labels, stale/unsupported feedback, and drag/reorder status. Catalog completeness SHALL be enforced by exhaustive compilation or an explicit all-language catalog test.

#### Scenario: Recovery manager follows active language
- **WHEN** recovery snapshots are available and the interface language changes
- **THEN** manager title, entry states, Restore, Discard, Restore All, Discard All, and status feedback render in the active language

#### Scenario: Block command chrome follows active language
- **WHEN** the slash palette or block menu is visible
- **THEN** every command and operation label renders through the active language catalog
- **AND** no newly introduced workflow text is hard-coded in English

#### Scenario: P1 catalog remains complete
- **WHEN** a P1 workflow message is added
- **THEN** tests fail until every supported language contains a non-empty translation

### Requirement: Show hidden files UI chrome SHALL be localized

The system SHALL route every user-visible string for the Show-hidden-files preference through the i18n layer, including the Preferences panel toggle label and any on/off status feedback reported when the preference is toggled. Adding the new message keys SHALL require translations in every supported language or the build SHALL fail.

#### Scenario: Preferences panel label reflects active language
- **WHEN** the active interface language changes
- **THEN** the Show-hidden-files toggle label in the Preferences panel renders in the active language

#### Scenario: Toggle status feedback reflects active language
- **WHEN** the user toggles Show-hidden-files
- **THEN** any status bar message indicating the new on/off state is produced through the active language translation

#### Scenario: New message keys require all-language translations
- **WHEN** a developer adds the Show-hidden-files message variants to `Msg`
- **THEN** the project fails to compile until every supported language covers them via the exhaustive `match`

### Requirement: Export preferences UI chrome SHALL be localized
Every user-visible string of the Preferences panel Export tab — the tab label, backend choice labels, availability status lines, pandoc path / reference template rows and their actions, PDF-engine choices, and every format option label — SHALL be routed through the i18n layer's `t` / `tf` functions and localized in every supported UI language.

#### Scenario: Export tab labels reflect the active language
- **WHEN** the active interface language is any of the supported languages and the user opens the Export tab
- **THEN** every label, action, and status line in the tab renders in that language with no hard-coded English literals

### Requirement: Markdown Reference tutorial link is localized
The system SHALL route the Markdown Reference overlay's Kenhuang tutorial-link label through the i18n `Msg` / `t` layer for every supported interface language. The destination URL SHALL be displayed verbatim rather than translated. Adding the label message SHALL require translations in every supported language or the build SHALL fail.

#### Scenario: Tutorial-link label follows the active language
- **WHEN** the Markdown Reference overlay is opened after the interface language changes
- **THEN** the tutorial-link label renders in the active language
- **AND** the visible URL is the language-selected Kenhuang destination displayed verbatim

