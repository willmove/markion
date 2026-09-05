## ADDED Requirements

### Requirement: Recent-workspace switcher chrome SHALL be localized
The system SHALL route every user-visible recent-workspace string through the i18n layer, including the Files-panel header switcher, empty-state recent-folder labels, Open Folder action inside the switcher, workspace-switch status text, and vanished-folder failure status. Adding the new message keys SHALL require translations in every supported language or the build SHALL fail.

#### Scenario: Header switcher reflects the active language
- **WHEN** the active interface language changes and a workspace root is established
- **THEN** the Files-panel workspace-name switcher labels render in the active language via `t(language, Msg::…)`

#### Scenario: Empty-state recent folders reflect the active language
- **WHEN** the file tree has no established root and recent workspaces are listed
- **THEN** empty-state heading and action labels are produced through the active language translation

#### Scenario: Switch status reflects the active language
- **WHEN** a workspace switch succeeds, the Open Folder picker is canceled, or the chosen folder is missing
- **THEN** the corresponding status text is produced through `t` or `tf` in the active language
