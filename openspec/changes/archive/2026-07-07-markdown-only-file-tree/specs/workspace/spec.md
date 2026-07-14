## MODIFIED Requirements

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
