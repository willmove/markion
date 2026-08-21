## MODIFIED Requirements

### Requirement: File tree panel with filename filtering
The editor SHALL provide a toggleable file tree panel whose workspace root can be established either by explicitly choosing File → Open Folder or, when opening supported content outside the current workspace, from that content's parent directory. The panel SHALL display Markdown files (`.md`/`.markdown`/`.mdown`), a curated set of plain-text files (`.txt`, `.text`, `.log`, `.csv`, `.tsv`, `.org`, `.rst`, `.adoc`, `.asciidoc`), and supported image files (`.png`, `.jpg`, `.jpeg`, `.gif`, `.webp`, `.bmp`, `.tif`, `.tiff`, `.svg`) nested under their containing folders. It SHALL list folders that exist on disk even when they contain no supported files, open Markdown and plain-text files on click as UTF-8 text, open supported image files on click as read-only images, mark the current file, support filename filtering, and support basic create / rename / delete / refresh operations for files and folders. Create File, Create Folder, and Rename SHALL collect a name through an in-app inline name editor: the editor renders inside the panel in place of the renamed row or directly below the parent folder row for create actions, and falls back to the top of the panel when that row is not visible; the editor is also rendered as a labeled prompt under the tab bar when the Files panel is hidden. Editing keys and clicks SHALL act on the name buffer only and SHALL NOT move the document caret or selection. Deleting a folder SHALL remove it recursively, including all of its contents, gated by a second confirmation for non-empty folders. Directories on a hard-coded ignore list (version-control, build-output, dependency/cache, and IDE directories) and hidden directories (whose name begins with `.`) SHALL NOT be listed. Other unsupported files (binaries, source code, and unsupported image formats) SHALL NOT appear in the tree. An explicitly selected workspace root SHALL be preserved while contained files are opened. The panel SHALL NOT scan the working directory on startup while only the in-memory welcome document is open; instead it SHALL show an empty-state placeholder until a file or folder is opened. Moving entries via the UI is **not** supported.

#### Scenario: Workspace scan displays supported files
- **WHEN** a workspace root is established via File → Open Folder, the File → Open dialog, the sidebar, or Save As
- **THEN** the file tree scans the applicable root on a background executor, displays Markdown, curated plain-text, and supported image files nested under the folders that contain them, lists folders that exist on disk even when empty, and renders a bounded number of rows per frame
- **AND** unsupported files and ignored or hidden directories are not listed

#### Scenario: Plain-text file opens from the tree
- **WHEN** the user clicks a curated plain-text file (e.g. `.txt`, `.log`, `.csv`) in the file tree
- **THEN** the file opens in the editor as UTF-8 text in its own tab, or focuses an existing tab for the same path
- **AND** the file is rendered through the existing preview pipeline

#### Scenario: Image file opens from the tree
- **WHEN** the user clicks a supported image file in the file tree
- **THEN** the file opens in a read-only image tab, or focuses an existing tab for the same path
- **AND** its bytes are not interpreted as UTF-8 document text

#### Scenario: Empty folders are shown
- **WHEN** a non-ignored, non-hidden directory exists on disk but contains no supported Markdown, plain-text, or image files
- **THEN** the directory is still listed in the file tree as a nesting row

#### Scenario: Empty state on startup
- **WHEN** the editor launches with the in-memory welcome document and no file or folder is open
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
