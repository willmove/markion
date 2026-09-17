## MODIFIED Requirements

### Requirement: All user-visible UI chrome SHALL be translated through the i18n layer
The system SHALL route every core-owned user-visible UI string (menu bar titles and items, in-app dropdown labels, status bar text, dialog text, search panel labels, file tree labels, file tree context-menu labels, the file-tree create/rename inline name-prompt label and placeholder, the recursive-folder-delete confirm dialog title and detail, preferences panel labels, the keyboard-shortcut reference, and plugin-management chrome) through the i18n module's `t` / `tf` / `shortcut_reference` / `sidebar_tab_label` functions. Hard-coded user-visible English literals in these surfaces SHALL NOT remain. Dynamic first-party plugin names and descriptions SHALL come only from the signed catalog's locale-complete identity records and SHALL be rendered through a validating plugin-localization adapter rather than treated as untrusted markup.

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

#### Scenario: Plugin identity follows the active language
- **WHEN** the plugin manager displays a signed first-party plugin and the interface language changes
- **THEN** the host chrome uses the root i18n catalog and the plugin name/description use the matching validated signed-catalog locale
- **AND** missing or malformed plugin locale data cannot become raw markup or executable content

#### Scenario: Document content is never translated
- **WHEN** the active language is Japanese, French, German, or Spanish
- **THEN** document content, the welcome Markdown, and user files remain untouched (only UI chrome is translated)

## ADDED Requirements

### Requirement: Plugin management and failure chrome SHALL be localized
Every host-owned string for discovering, installing, verifying, enabling, disabling, updating, rolling back, uninstalling, measuring, starting, stopping, timing out, quarantining, and recovering plugins SHALL be present in every supported language. Missing-provider document surfaces, permission disclosures, signature and compatibility failures, disk usage, progress, cancellation, crash/timeout states, and offline guidance SHALL use stable localized host categories and SHALL NOT display raw process, protocol, operating-system, or native-library diagnostics.

#### Scenario: Plugin installation follows the active language
- **WHEN** the user views or confirms an official plugin installation
- **THEN** all host labels, size and permission explanations, progress, confirmation, and failure text render in the active language

#### Scenario: Plugin process failure is sanitized and localized
- **WHEN** a plugin crashes, times out, or violates the protocol
- **THEN** Markion shows the corresponding localized stable category and recovery action
- **AND** raw stderr, command lines, paths, protocol bytes, and native diagnostics are excluded from user-facing text

#### Scenario: Catalog locale completeness is enforced
- **WHEN** an official plugin catalog entry lacks a non-empty safe name and description for any supported Markion language
- **THEN** catalog validation and release publication fail before users can install that entry

