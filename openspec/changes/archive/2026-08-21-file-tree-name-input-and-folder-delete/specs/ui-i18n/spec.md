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
