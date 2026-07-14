## ADDED Requirements

### Requirement: File tree entry context menu metadata actions
The editor SHALL provide additional right-click context-menu actions for file-tree file and folder entries: Copy Path, Copy Relative Path, and Properties. The file and folder context menus SHALL continue to include Rename and Delete actions. These actions SHALL operate on the clicked entry, update status feedback where appropriate, and preserve the file tree's bounded row rendering.

#### Scenario: File entry menu includes management and metadata actions
- **WHEN** the user right-clicks a Markdown file in the file tree
- **THEN** the context menu includes Rename, Delete, Copy Path, Copy Relative Path, and Properties actions
- **AND** choosing Copy Path copies the file's absolute path to the clipboard
- **AND** choosing Copy Relative Path copies the file's path relative to the workspace root to the clipboard

#### Scenario: Folder entry menu includes management and metadata actions
- **WHEN** the user right-clicks a folder in the file tree
- **THEN** the context menu includes Rename, Delete, Copy Path, Copy Relative Path, and Properties actions scoped to that folder

#### Scenario: Properties displays file metadata
- **WHEN** the user chooses Properties for a file entry
- **THEN** the editor displays the entry kind, absolute path, workspace-relative path, file size, and modified timestamp when available
- **AND** the document text, dirty flag, undo history, and derived Markdown caches are unchanged

#### Scenario: Properties displays folder metadata without blocking rendering
- **WHEN** the user chooses Properties for a folder entry
- **THEN** the editor displays the entry kind, absolute path, workspace-relative path, and modified timestamp when available
- **AND** folder size is omitted or marked unavailable unless it can be computed without blocking the file-tree render path

#### Scenario: Metadata action failure reports status
- **WHEN** a Copy Path, Copy Relative Path, or Properties action cannot complete because the entry no longer exists or the platform operation fails
- **THEN** the editor reports a localized failure status and keeps the current document unchanged
