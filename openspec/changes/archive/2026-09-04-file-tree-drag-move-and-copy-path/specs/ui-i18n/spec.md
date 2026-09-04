## ADDED Requirements

### Requirement: File tree move and path-copy chrome SHALL be localized
The system SHALL route every user-visible string introduced for file-tree drag-move and path-copy through the i18n `Msg` / `t` / `tf` layer for every supported interface language. This includes the Copy Path and Copy Relative Path context-menu labels, move success and failure status, name-collision status, invalid-move status, save-before-move status, and path-copy success and failure status. Hard-coded user-visible English literals SHALL NOT remain on these surfaces.

#### Scenario: Path-copy labels follow the active language
- **WHEN** the active interface language is Simplified Chinese and the user right-clicks a file-tree file or folder
- **THEN** Copy Path and Copy Relative Path render in Simplified Chinese through the i18n layer

#### Scenario: Move and copy status follow the active language
- **WHEN** a drag-move succeeds or fails, or a path-copy succeeds or fails
- **THEN** the status bar text is produced by `t` / `tf` in the active language
- **AND** templatized messages interpolate the path or error through positional arguments
