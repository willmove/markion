## MODIFIED Requirements

### Requirement: File tree panel with filename filtering
The editor SHALL provide a toggleable file tree panel whose workspace root can be established either by explicitly choosing File → Open Folder or, when opening a Markdown document outside the current workspace, from that document's parent directory. The panel SHALL display only Markdown files (`.md`/`.markdown`/`.mdown`) nested under the folders that contain them, open Markdown files on click, mark the current document, support filename filtering, and support basic create / rename / delete / refresh operations for files and folders. An explicitly selected workspace root SHALL be preserved while Markdown documents contained within it are opened. The panel SHALL NOT scan the working directory on startup while only the in-memory welcome document is open; instead it SHALL show an empty-state placeholder until a file or folder is opened. Moving entries via the UI is **not** supported. Non-Markdown files SHALL NOT appear in the tree.

#### Scenario: Workspace is scanned and displayed as Markdown-only
- **WHEN** a workspace root is established via File → Open Folder, the File → Open dialog, the sidebar, or Save As
- **THEN** the file tree scans the applicable root on a background executor, displays only Markdown files nested under the folders that contain them, and renders a bounded number of rows per frame
- **AND** non-Markdown files and folders that contain no Markdown files are not listed

#### Scenario: Empty state on startup
- **WHEN** the editor launches with the in-memory welcome document and no file or folder is open
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
