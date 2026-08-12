## MODIFIED Requirements

### Requirement: File tree panel with filename filtering
The editor SHALL provide a toggleable file tree panel whose workspace root can be established either by explicitly choosing File → Open Folder or, when opening a document outside the current workspace, from that document's parent directory. The panel SHALL display Markdown files (`.md`/`.markdown`/`.mdown`) together with a curated set of plain-text files (`.txt`, `.text`, `.log`, `.csv`, `.tsv`, `.org`, `.rst`, `.adoc`, `.asciidoc`) nested under their containing folders, SHALL list folders that exist on disk even when they contain no Markdown or plain-text files, SHALL open Markdown and plain-text files on click as UTF-8 text, mark the current document, support filename filtering, and support basic create / rename / delete / refresh operations for files and folders. Directories on a hard-coded ignore list (version-control, build-output, dependency/cache, and IDE directories) and hidden directories (whose name begins with `.`) SHALL NOT be listed. Non-text files (binaries, images, source code, etc.) SHALL NOT appear in the tree. An explicitly selected workspace root SHALL be preserved while documents contained within it are opened. The panel SHALL NOT scan the working directory on startup while only the in-memory welcome document is open; instead it SHALL show an empty-state placeholder until a file or folder is opened. Moving entries via the UI is **not** supported.

#### Scenario: Workspace is scanned and displayed as Markdown-only
- **WHEN** a workspace root is established via File → Open Folder, the File → Open dialog, the sidebar, or Save As
- **THEN** the file tree scans the applicable root on a background executor, displays Markdown files and the curated plain-text files nested under the folders that contain them, lists folders that exist on disk even when empty, and renders a bounded number of rows per frame
- **AND** non-text files (binaries, images, source code) and ignored or hidden directories are not listed

#### Scenario: Plain-text file opens from the tree
- **WHEN** the user clicks a curated plain-text file (e.g. `.txt`, `.log`, `.csv`) in the file tree
- **THEN** the file opens in the editor as UTF-8 text in its own tab, or focuses an existing tab for the same path
- **AND** the file is rendered through the existing preview pipeline

#### Scenario: Empty folders are shown
- **WHEN** a non-ignored, non-hidden directory exists on disk but contains no Markdown or plain-text files
- **THEN** the directory is still listed in the file tree as a nesting row

#### Scenario: Empty state on startup
- **WHEN** the editor launches with the in-memory welcome document and no file or folder is open
- **THEN** the file tree does not scan the working directory and shows an empty-state placeholder instead of the directory hierarchy

#### Scenario: Open folder establishes the workspace and reveals Files
- **WHEN** the user chooses File → Open Folder and selects one directory
- **THEN** that directory becomes the file-tree workspace root without replacing or modifying the active document
- **AND** the left sidebar becomes visible on the Files tab
- **AND** the selected directory is scanned asynchronously, including when it contains no Markdown or plain-text files

#### Scenario: Folder selection cancellation preserves state
- **WHEN** the user cancels the Open Folder picker
- **THEN** the current workspace root, file tree, sidebar selection, active document, dirty state, and undo history remain unchanged
- **AND** the editor reports localized cancellation feedback

#### Scenario: Folder scan failure is non-destructive
- **WHEN** the selected directory cannot be scanned
- **THEN** the editor reports a localized failure status
- **AND** the active document, dirty state, undo history, and derived Markdown caches remain unchanged

#### Scenario: Contained documents preserve the selected root
- **WHEN** a document inside the current workspace root is opened or focused
- **THEN** the current workspace root remains unchanged and the document is marked in the tree

#### Scenario: External document rebases the workspace
- **WHEN** a document outside the current workspace root is opened through an existing document-opening flow
- **THEN** the workspace root changes to that document's parent directory and the file tree rescans it

#### Scenario: Open, filter, and current-file marking
- **WHEN** the user clicks a Markdown or plain-text file, types in the filename filter, or switches documents
- **THEN** the file opens in the editor, the tree filters by filename, and the current document is marked in the tree

#### Scenario: Create, rename, delete, refresh
- **WHEN** the user creates a file or folder, renames or deletes an entry, or refreshes the tree
- **THEN** the workspace reflects the change and the tree updates accordingly
