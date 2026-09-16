## ADDED Requirements

### Requirement: Session restore for workspace root and open documents
The editor SHALL persist a session snapshot containing the current file-tree workspace root (when one is established), the ordered list of open saved Markdown document paths, and the active document path when it has a file path. The snapshot SHALL be stored in a dedicated `session.toml` under the Markion config directory, separate from `config.toml`. On launch with no CLI file/folder open intent, the editor SHALL restore that session: re-establish a still-valid workspace root and scan it asynchronously, reopen still-existing Markdown document paths as tabs, and focus the restored active document when it is among the reopened tabs. Untitled or recovery-only tabs SHALL NOT be written into the session snapshot. Missing paths SHALL be skipped on restore. Crash-recovery prompting SHALL continue to run after session restore. A CLI file or folder open intent SHALL take precedence over the conflicting session fields for that launch while still allowing the recent-files list to load.

#### Scenario: Workspace root restores on launch
- **WHEN** the previous session recorded a workspace root that still exists and the app launches without a CLI open intent
- **THEN** that directory becomes the file-tree workspace root and is scanned asynchronously
- **AND** the Files panel shows the restored workspace instead of the empty-state placeholder

#### Scenario: Open saved tabs restore on launch
- **WHEN** the previous session recorded one or more saved Markdown paths that still exist and the app launches without a CLI file open intent
- **THEN** those documents reopen as tabs in the recorded order
- **AND** the recorded active path is focused when it is among the restored tabs

#### Scenario: Missing session paths are skipped
- **WHEN** a recorded open-file or workspace-root path no longer exists at launch
- **THEN** that path is skipped without failing startup
- **AND** any remaining valid session paths still restore

#### Scenario: CLI open intent overrides session restore
- **WHEN** the app launches with a CLI file or folder open intent and a session snapshot also exists
- **THEN** the CLI intent is applied for the requested file or folder
- **AND** conflicting session restore for those fields is not applied on that launch

#### Scenario: Untitled tabs are not persisted in the session
- **WHEN** the only open tab is an untitled welcome or recovery document with no saved path
- **THEN** the session snapshot does not record that tab as an open file
- **AND** a previously recorded workspace root may still be persisted and restored

### Requirement: Recent files list
The editor SHALL maintain a bounded recent-files list of Markdown paths the user has opened or successfully saved, stored alongside the session snapshot. Opening or saving a path SHALL move it to the front of the list and deduplicate it. The list SHALL drop entries that fail to open because the file is missing. Clearing recent files SHALL empty the persisted list.

#### Scenario: Opening a file updates recent files
- **WHEN** the user opens or successfully saves a Markdown document with a file path
- **THEN** that path becomes the most recent entry in the recent-files list
- **AND** duplicate older entries for the same path are removed

#### Scenario: Recent list is bounded
- **WHEN** more distinct Markdown paths are opened than the configured recent-files bound
- **THEN** the oldest entries beyond the bound are dropped from the persisted list

#### Scenario: Clear recent files empties the list
- **WHEN** the user invokes Clear Recent Files
- **THEN** the recent-files list becomes empty and the empty list is persisted

## MODIFIED Requirements

### Requirement: File tree panel with filename filtering
The editor SHALL provide a toggleable file tree panel whose workspace root can be established by explicitly choosing File → Open Folder, by opening a Markdown document outside the current workspace (from that document's parent directory), by restoring a previous session's workspace root on launch, or by a CLI folder open intent. The panel SHALL display only Markdown files (`.md`/`.markdown`/`.mdown`) nested under the folders that contain them, open Markdown files on click, mark the current document, support filename filtering, and support basic create / rename / delete / refresh operations for files and folders. An explicitly selected workspace root SHALL be preserved while Markdown documents contained within it are opened. The panel SHALL NOT scan the working directory on startup while only the in-memory welcome document is open and no session or CLI workspace root is available; instead it SHALL show an empty-state placeholder until a file or folder is opened or a session workspace root is restored. Moving entries via the UI is **not** supported. Non-Markdown files SHALL NOT appear in the tree.

#### Scenario: Workspace is scanned and displayed as Markdown-only
- **WHEN** a workspace root is established via File → Open Folder, the File → Open dialog, the sidebar, Save As, session restore, or a CLI folder open intent
- **THEN** the file tree scans the applicable root on a background executor, displays only Markdown files nested under the folders that contain them, and renders a bounded number of rows per frame
- **AND** non-Markdown files and folders that contain no Markdown files are not listed

#### Scenario: Empty state on startup
- **WHEN** the editor launches with the in-memory welcome document and no file, folder, session workspace root, or CLI workspace root is available
- **THEN** the file tree does not scan the working directory and shows an empty-state placeholder instead of the directory hierarchy

#### Scenario: Open folder establishes the workspace and reveals Files
- **WHEN** the user chooses File → Open Folder and selects one directory
- **THEN** that directory becomes the file-tree workspace root without replacing or modifying the active document
- **AND** the left sidebar becomes visible on the Files tab
- **AND** the selected directory is scanned asynchronously, including when it contains no Markdown files

#### Scenario: Folder selection cancellation preserves state
- **WHEN** the user cancels the Open Folder picker
- **THEN** the current workspace root, file tree, sidebar selection, active document, dirty state, and undo history remain unchanged
- **AND** the editor reports localized cancellation feedback

#### Scenario: Folder scan failure is non-destructive
- **WHEN** the selected directory cannot be scanned
- **THEN** the editor reports a localized failure status
- **AND** the active document, dirty state, undo history, and derived Markdown caches remain unchanged

#### Scenario: Contained documents preserve the selected root
- **WHEN** a Markdown document inside the current workspace root is opened or focused
- **THEN** the current workspace root remains unchanged and the document is marked in the tree

#### Scenario: External document rebases the workspace
- **WHEN** a Markdown document outside the current workspace root is opened through an existing document-opening flow
- **THEN** the workspace root changes to that document's parent directory and the file tree rescans it

#### Scenario: Open, filter, and current-file marking
- **WHEN** the user clicks a Markdown file, types in the filename filter, or switches documents
- **THEN** the file opens in the editor, the tree filters by filename, and the current document is marked in the tree

#### Scenario: Create, rename, delete, refresh
- **WHEN** the user creates a file or folder, renames or deletes an entry, or refreshes the tree
- **THEN** the workspace reflects the change and the tree updates accordingly
