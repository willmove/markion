## MODIFIED Requirements

### Requirement: File tree panel with filename filtering
The editor SHALL provide a toggleable file tree panel that scans the directory containing the currently open Markdown file, displays only Markdown files (`.md`/`.markdown`/`.mdown`) nested under the folders that contain them, opens Markdown files, marks the current document, supports filename filtering without a persistent filter input in the panel, and supports create / rename / delete / refresh operations for files and folders through a right-click context menu. Create File, Create Folder, and Rename SHALL collect a name from the user through an in-app inline name prompt (no hard-coded default is applied without user confirmation). Deleting a folder SHALL remove it recursively, including all of its contents. The panel SHALL NOT render an always-visible file-tree filter field or always-visible file-operation toolbar. The panel SHALL NOT scan the working directory on startup while only the in-memory welcome document is open; instead it SHALL show an empty-state placeholder until a file is opened. Moving entries via the UI is **not** supported. Non-Markdown files SHALL NOT appear in the tree.

#### Scenario: Workspace is scanned and displayed as Markdown-only
- **WHEN** a Markdown file is opened (via the sidebar, the File -> Open dialog, or Save As)
- **THEN** the file tree scans that file's parent directory, displays only Markdown files nested under the folders that contain them, and renders a bounded number of rows per frame
- **AND** non-Markdown files and folders that contain no Markdown files are not listed

#### Scenario: Empty state on startup
- **WHEN** the editor launches with the in-memory welcome document and no file is open
- **THEN** the file tree does not scan the working directory and shows an empty-state placeholder instead of the directory hierarchy

#### Scenario: Open, filter, and current-file marking
- **WHEN** the user opens a Markdown file from the tree, invokes the file filter from the context menu, or switches documents
- **THEN** the file opens in the editor, the tree filters by filename when a filter is supplied, and the current document is marked in the tree

#### Scenario: Persistent file-tree controls are hidden
- **WHEN** the Files panel renders with a loaded file tree
- **THEN** it does not show an always-visible filter input
- **AND** it does not show always-visible `New`, `Dir`, `Ren`, `Del`, or `Ref` toolbar buttons

#### Scenario: Context menu on a file row
- **WHEN** the user right-clicks a Markdown file row
- **THEN** the file-tree context menu offers Open, Open in New Tab, Rename, Delete, Show in System File Manager, and Refresh actions
- **AND** choosing Open opens the file in the current editor flow
- **AND** choosing Open in New Tab opens that Markdown file in a separate tab

#### Scenario: Context menu on a folder row
- **WHEN** the user right-clicks a folder row
- **THEN** the file-tree context menu offers Create File, Create Folder, Rename, Delete, Show in System File Manager, and Refresh actions scoped to that folder

#### Scenario: Context menu on blank tree space
- **WHEN** the user right-clicks blank space inside the Files panel
- **THEN** the file-tree context menu offers Create File, Create Folder, Refresh, Show Workspace in System File Manager, and Filter Files actions scoped to the workspace root or selected folder

#### Scenario: Create file and create folder collect a name
- **WHEN** the user chooses Create File or Create Folder from the context menu
- **THEN** the editor opens an in-app inline name prompt pre-filled with a default name (`untitled.md` for files, `New Folder` for folders)
- **AND** pressing Enter creates the entry under the target folder (or workspace root) with the typed name and refreshes the tree
- **AND** pressing Escape cancels without creating anything
- **AND** confirming with an empty name is rejected and no entry is created

#### Scenario: Rename collects a name and preserves open tabs
- **WHEN** the user chooses Rename from the context menu on a file or folder
- **THEN** the editor opens an in-app inline name prompt pre-filled with the entry's current name
- **AND** if the entry is the active document and that document is dirty, the editor refuses the rename and prompts the user to save first
- **AND** pressing Enter renames the entry to the typed name and refreshes the tree
- **AND** any tab whose document path was the old path is reloaded from the new path in place
- **AND** pressing Escape cancels without renaming

#### Scenario: Deleting a folder is recursive
- **WHEN** the user chooses Delete on a folder and confirms
- **THEN** the editor removes the folder and all of its contents recursively
- **AND** the tree updates to reflect the removal
- **AND** any tab whose document path was inside the removed folder is reset to a fresh untitled document

#### Scenario: Deleting a non-empty folder requires a second confirmation
- **WHEN** the user chooses Delete on a non-empty folder
- **THEN** the editor first shows the standard delete confirmation
- **AND** after the first confirmation it shows a second warning that the folder and all of its contents will be removed
- **AND** the folder is removed only if both confirmations are accepted
- **AND** cancelling either confirmation aborts the delete

#### Scenario: Deleting a file or empty folder requires a single confirmation
- **WHEN** the user chooses Delete on a file or on an empty folder
- **THEN** the editor shows a single delete confirmation
- **AND** confirming removes the entry and refreshes the tree

#### Scenario: Reveal target in system file manager
- **WHEN** the user chooses Show in System File Manager for a file, folder, or workspace
- **THEN** the editor asks the operating system file manager to reveal that target
- **AND** failures are surfaced as localized status text without modifying editor state
