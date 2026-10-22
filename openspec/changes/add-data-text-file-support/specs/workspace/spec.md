## MODIFIED Requirements

### Requirement: File tree panel with filename filtering
The editor SHALL provide a toggleable file tree panel whose workspace root can be established by explicitly choosing File → Open Folder, by opening supported content outside the current workspace (from that content's parent directory), by restoring a previous session's workspace root on launch, or by a CLI folder open intent. The panel SHALL display Markdown files (`.md`/`.markdown`/`.mdown`), a curated set of plain-text files (`.txt`, `.text`, `.log`, `.csv`, `.tsv`, `.org`, `.rst`, `.adoc`, `.asciidoc`), a curated set of data files (`.json`, `.yaml`, `.yml`, `.toml`), and supported image files (`.png`, `.jpg`, `.jpeg`, `.gif`, `.webp`, `.bmp`, `.tif`, `.tiff`, `.svg`) nested under their containing folders. It SHALL list folders that exist on disk even when they contain no supported files, open Markdown and plain-text files on click as UTF-8 text, open data files on click as data text documents (source-only editing per the data-text-editing capability), open supported image files on click as read-only images, mark the current file, support filename filtering, and support basic create / rename / delete / refresh operations for files and folders. Create File, Create Folder, and Rename SHALL collect a name through an in-app inline name editor: the editor renders inside the panel in place of the renamed row or directly below the parent folder row for create actions, and falls back to the top of the panel when that row is not visible; the editor is also rendered as a labeled prompt under the tab bar when the Files panel is hidden. Editing keys and clicks SHALL act on the name buffer only and SHALL NOT move the document caret or selection. Deleting a folder SHALL remove it recursively, including all of its contents, gated by a second confirmation for non-empty folders. Directories on a hard-coded ignore list (version-control, build-output, dependency/cache, and IDE directories) and hidden directories (whose name begins with `.`) SHALL NOT be listed. Other unsupported files (binaries, source code, and unsupported image formats) SHALL NOT appear in the tree. An explicitly selected workspace root SHALL be preserved while contained files are opened. The panel SHALL NOT scan the working directory on startup while only the in-memory welcome document is open and no session or CLI workspace root is available; instead it SHALL show an empty-state placeholder until a file or folder is opened or a session workspace root is restored. The panel SHALL let the user move a listed file or folder into another listed folder or the workspace root by left-dragging it onto that folder, onto a file inside that folder, or onto the workspace root. File and folder context menus SHALL include Copy Path and Copy Relative Path.

#### Scenario: Workspace scan displays supported files
- **WHEN** a workspace root is established via File → Open Folder, the File → Open dialog, the sidebar, Save As, session restore, or a CLI folder open intent
- **THEN** the file tree scans the applicable root on a background executor, displays Markdown, curated plain-text, curated data, and supported image files nested under the folders that contain them, lists folders that exist on disk even when empty, and renders a bounded number of rows per frame
- **AND** unsupported files and ignored or hidden directories are not listed

#### Scenario: Plain-text file opens from the tree
- **WHEN** the user clicks a curated plain-text file (e.g. `.txt`, `.log`, `.csv`) in the file tree
- **THEN** the file opens in the editor as UTF-8 text in its own tab, or focuses an existing tab for the same path
- **AND** the file is rendered through the existing preview pipeline

#### Scenario: Data file opens from the tree
- **WHEN** the user clicks a curated data file (`.json`, `.yaml`, `.yml`, `.toml`) in the file tree
- **THEN** the file opens in its own tab as a data text document (or focuses an existing tab for the same path), rendered on its source editing surface per the data-text-editing capability

#### Scenario: Image file opens from the tree
- **WHEN** the user clicks a supported image file in the file tree
- **THEN** the file opens in a read-only image tab, or focuses an existing tab for the same path
- **AND** its bytes are not interpreted as UTF-8 document text

#### Scenario: Empty folders are shown
- **WHEN** a non-ignored, non-hidden directory exists on disk but contains no supported Markdown, plain-text, data, or image files
- **THEN** the directory is still listed in the file tree as a nesting row

#### Scenario: Empty state on startup
- **WHEN** the editor launches with the in-memory welcome document and no file, folder, session workspace root, or CLI workspace root is available
- **THEN** the file tree does not scan the working directory and shows an empty-state placeholder instead of the directory hierarchy

#### Scenario: Open folder establishes the workspace and reveals Files
- **WHEN** the user chooses File → Open Folder and selects one directory
- **THEN** that directory becomes the file-tree workspace root without replacing or modifying the active content
- **AND** the left sidebar becomes visible on the Files tab
- **AND** the selected directory is scanned asynchronously, including when it contains no supported files

#### Scenario: Folder selection cancellation preserves state
- **WHEN** the user cancels the Open Folder picker
- **THEN** the current workspace root, file tree, sidebar selection, active content, and every document's dirty state and undo history remain unchanged
- **AND** the editor reports localized cancellation feedback

#### Scenario: Folder scan failure is non-destructive
- **WHEN** the selected directory cannot be scanned
- **THEN** the editor reports a localized failure status
- **AND** the active content and every document's dirty state, undo history, and derived Markdown caches remain unchanged

#### Scenario: Contained files preserve the selected root
- **WHEN** a supported document or image inside the current workspace root is opened or focused
- **THEN** the current workspace root remains unchanged and the file is marked in the tree

#### Scenario: External file rebases the workspace
- **WHEN** supported content outside the current workspace root is opened through an existing interactive file-opening flow
- **THEN** the workspace root changes to that file's parent directory and the file tree rescans it

#### Scenario: Open, filter, and current-file marking
- **WHEN** the user clicks a supported document or image, types in the filename filter, or switches tabs
- **THEN** the file opens in its appropriate content surface, the tree filters by filename, and the current file is marked in the tree

#### Scenario: Create, rename, delete, refresh
- **WHEN** the user creates a file or folder, renames or deletes an entry, or refreshes the tree
- **THEN** the workspace reflects the change and the tree updates accordingly

#### Scenario: Create file and create folder collect a name
- **WHEN** the user chooses Create File or Create Folder from the context menu
- **THEN** the editor opens an inline name editor pre-filled with a default name (`untitled.md` for files, `New Folder` for folders) with the whole name selected, rendered directly below the target folder row (or at the top of the panel when that row is not visible)
- **AND** pressing Enter creates the entry under the target folder (or workspace root) with the typed name and refreshes the tree
- **AND** pressing Escape cancels without creating anything
- **AND** clicking elsewhere commits through the same pipeline as Enter
- **AND** confirming with an empty name is rejected, keeps the editor open, and no entry is created

#### Scenario: Rename collects a name and preserves open tabs
- **WHEN** the user chooses Rename from the context menu on a file or folder
- **THEN** the editor opens an inline name editor in place of the entry's row, pre-filled with the entry's current name and with the base name selected so the extension is preserved (dotfile names are selected whole)
- **AND** if the entry is the active document and that document is dirty, the editor refuses the rename and prompts the user to save first
- **AND** pressing Enter renames the entry to the typed name and refreshes the tree
- **AND** any tab whose document path was the old path is reloaded from the new path in place
- **AND** pressing Escape cancels without renaming

#### Scenario: Name editor editing keys stay inside the name buffer
- **WHEN** the inline name editor is open and the user types, moves the caret with Left / Right / Home / End, extends the selection with Shift+Arrow or Select All, or deletes with Backspace / Delete
- **THEN** those keys edit the name buffer only (typing replaces the selection, including an active IME composition, at the caret)
- **AND** the document caret and selection are unchanged while the editor is open

#### Scenario: Click-away commit does not leak into the tree
- **WHEN** the inline name editor is open and the user left-clicks outside the field (below the menu bar)
- **THEN** the click commits the typed name through the same pipeline as Enter and the editor closes
- **AND** that same click does not also open a file, toggle a folder, or switch tabs
- **AND** if the commit is refused because the name is empty, the editor stays open and the click still does not trigger other actions
- **AND** opening a context menu or menu-bar menu cancels the editor without touching the filesystem

#### Scenario: Editor-pane clicks leave the document untouched while renaming
- **WHEN** the inline name editor is open and the user clicks or drags in the document editor pane
- **THEN** the document caret does not move and no drag selection starts
- **AND** the mouse-down commits the name editor (click-away semantics)

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

### Requirement: Session restore for workspace root and open documents
The editor SHALL persist a session snapshot containing the current file-tree workspace root (when one is established), the ordered list of open saved document and image paths (Markdown, curated plain-text, data text, and supported image tabs), and the active document path when it has a file path. The snapshot SHALL be stored in a dedicated `session.toml` under the Markion config directory, separate from `config.toml`. On launch with no CLI file/folder open intent, the editor SHALL restore that session: re-establish a still-valid workspace root and scan it asynchronously, reopen still-existing recorded paths as tabs of their appropriate kind (document or image), and focus the restored active document when it is among the reopened tabs. Untitled or recovery-only tabs SHALL NOT be written into the session snapshot. Missing paths SHALL be skipped on restore. Crash-recovery prompting SHALL continue to run after session restore. A CLI file or folder open intent SHALL take precedence over the conflicting session fields for that launch while still allowing the recent-files list to load.

#### Scenario: Workspace root restores on launch
- **WHEN** the previous session recorded a workspace root that still exists and the app launches without a CLI open intent
- **THEN** that directory becomes the file-tree workspace root and is scanned asynchronously
- **AND** the Files panel shows the restored workspace instead of the empty-state placeholder

#### Scenario: Open saved tabs restore on launch
- **WHEN** the previous session recorded one or more saved document paths (Markdown, plain-text, data text, or image) that still exist and the app launches without a CLI file open intent
- **THEN** those files reopen as tabs of their appropriate kind in the recorded order
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
The editor SHALL maintain a bounded recent-files list of document paths (Markdown, curated plain-text, and data text files) the user has opened or successfully saved, stored alongside the session snapshot. Opening or saving a path SHALL move it to the front of the list and deduplicate it. The list SHALL drop entries that fail to open because the file is missing. Clearing recent files SHALL empty the persisted list.

#### Scenario: Opening a file updates recent files
- **WHEN** the user opens or successfully saves a document with a file path
- **THEN** that path becomes the most recent entry in the recent-files list
- **AND** duplicate older entries for the same path are removed

#### Scenario: Recent list is bounded
- **WHEN** more distinct document paths are opened than the configured recent-files bound
- **THEN** the oldest entries beyond the bound are dropped from the persisted list

#### Scenario: Clear recent files empties the list
- **WHEN** the user invokes Clear Recent Files
- **THEN** the recent-files list becomes empty and the empty list is persisted

## ADDED Requirements

### Requirement: External drop and CLI open intents admit plain-text and data documents
Files dragged from the operating system onto the editor window, and file paths passed to the editor as launch open intents, SHALL be admitted for every document kind the file tree lists: Markdown files SHALL open as Markdown documents, curated plain-text files SHALL open as plain-text documents rendered through the existing preview pipeline, data files (`.json`, `.yaml`, `.yml`, `.toml`) SHALL open as data text documents per the data-text-editing capability, and supported image files SHALL follow the existing image drop behavior. Files with other extensions, and directories, SHALL be skipped by these open paths without error. Each admitted document kind SHALL open through the same tab-uniqueness and session/workspace rules as files opened from the tree.

#### Scenario: Mixed external drop opens all document kinds
- **WHEN** the user drags a `.md` file, a `.txt` file, and a `.json` file onto the editor window in one drop
- **THEN** all three open as tabs of their respective kinds
- **AND** a dropped file with an unsupported extension or a directory is skipped without error

#### Scenario: CLI open intent opens a data file
- **WHEN** the editor is launched with a `.toml` file path as its open intent
- **THEN** that file opens as a data text document and the workspace root is established from its parent directory like any other supported content

#### Scenario: Dropped plain-text file opens as a document
- **WHEN** the user drags a `.csv` file onto the editor window
- **THEN** the file opens as a plain-text document in its own tab instead of being ignored
