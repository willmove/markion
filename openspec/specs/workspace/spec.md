# workspace

## Purpose

Covers the file tree panel and the auto-save / crash-recovery subsystem. Drag-and-drop file moves and a UI affordance for moving entries are **not** part of this capability (the underlying move API exists but is not surfaced); they are future candidates.
## Requirements
### Requirement: File tree panel with filename filtering
The editor SHALL provide a toggleable file tree panel that scans the directory containing the currently open Markdown file, displays only Markdown files (`.md`/`.markdown`/`.mdown`) nested under the folders that contain them, opens Markdown files on click, marks the current document, supports filename filtering, and supports basic create / rename / delete / refresh operations for files and folders. The panel SHALL NOT scan the working directory on startup while only the in-memory welcome document is open; instead it SHALL show an empty-state placeholder until a file is opened. Moving entries via the UI is **not** supported. Non-Markdown files SHALL NOT appear in the tree.

#### Scenario: Workspace is scanned and displayed as Markdown-only
- **WHEN** a Markdown file is opened (via the sidebar, the File → Open dialog, or Save As)
- **THEN** the file tree scans that file's parent directory, displays only Markdown files nested under the folders that contain them, and renders a bounded number of rows per frame
- **AND** non-Markdown files and folders that contain no Markdown files are not listed

#### Scenario: Empty state on startup
- **WHEN** the editor launches with the in-memory welcome document and no file is open
- **THEN** the file tree does not scan the working directory and shows an empty-state placeholder instead of the directory hierarchy

#### Scenario: Open, filter, and current-file marking
- **WHEN** the user clicks a Markdown file, types in the filename filter, or switches documents
- **THEN** the file opens in the editor, the tree filters by filename, and the current document is marked in the tree

#### Scenario: Create, rename, delete, refresh
- **WHEN** the user creates a file or folder, renames or deletes an entry, or refreshes the tree
- **THEN** the workspace reflects the change and the tree updates accordingly

### Requirement: Auto-save and recovery
The editor SHALL auto-save after a period of inactivity, write saved documents to their file path, and write unsaved documents to a recovery copy that can be restored on the next launch. The inactivity interval SHALL come from the `[auto_save] delay_secs` config value (default 5 seconds) and auto-save SHALL be disableable via `[auto_save] enabled = false`; both are configurable only through the config file, not the Preferences panel.

#### Scenario: Saved document auto-saves after the configured interval
- **WHEN** a saved document is modified and the user is inactive past the configured auto-save interval
- **THEN** the document is written to its file path and the status bar reports the auto-save

#### Scenario: Unsaved document writes a recovery copy
- **WHEN** an unsaved document is modified and the user is inactive past the configured auto-save interval
- **THEN** a recovery copy is written and offered for restoration on the next launch

#### Scenario: Auto-save disabled by config
- **WHEN** `[auto_save] enabled = false` is set in `config.toml`
- **THEN** no auto-save or recovery copy is written on inactivity; manual save is unaffected
