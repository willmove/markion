## MODIFIED Requirements

### Requirement: File tree panel with filename filtering
The editor SHALL provide a toggleable file tree panel that scans the directory containing the currently open Markdown file, displays only Markdown files (`.md`/`.markdown`/`.mdown`) nested under the folders that contain them, opens Markdown files, marks the current document, supports filename filtering without a persistent filter input in the panel, and supports basic create / rename / delete / refresh operations for files and folders through a right-click context menu. The panel SHALL NOT render an always-visible file-tree filter field or always-visible file-operation toolbar. The panel SHALL NOT scan the working directory on startup while only the in-memory welcome document is open; instead it SHALL show an empty-state placeholder until a file is opened. Moving entries via the UI is **not** supported. Non-Markdown files SHALL NOT appear in the tree.

#### Scenario: Workspace is scanned and displayed as Markdown-only
- **WHEN** a Markdown file is opened (via the sidebar, the File → Open dialog, or Save As)
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

#### Scenario: Create, rename, delete, refresh
- **WHEN** the user creates a file or folder, renames or deletes an entry, or refreshes the tree from the context menu or existing keyboard shortcuts
- **THEN** the workspace reflects the change and the tree updates accordingly

#### Scenario: Reveal target in system file manager
- **WHEN** the user chooses Show in System File Manager for a file, folder, or workspace
- **THEN** the editor asks the operating system file manager to reveal that target
- **AND** failures are surfaced as localized status text without modifying editor state

#### Scenario: Workspace scan displays supported files
- **WHEN** a workspace root is established via File → Open Folder, the File → Open dialog, the sidebar, Save As, session restore, or a CLI folder open intent
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

#### Scenario: New workspace initially shows one level
- **WHEN** the first successful scan for a newly established workspace contains files and Markdown-bearing folders at multiple depths
- **THEN** the file tree shows the files and folders directly inside the workspace root
- **AND** every deeper descendant remains hidden behind its collapsed top-level folder row

#### Scenario: User controls directory expansion
- **WHEN** the user expands or collapses a visible directory row after the initial tree is shown
- **THEN** that directory's descendants become visible or hidden accordingly without changing other directory branches

#### Scenario: Same-workspace refresh preserves expansion state
- **WHEN** the user changes directory expansion state and refreshes the current workspace
- **THEN** surviving directory paths retain their prior expanded or collapsed state
- **AND** the one-level initial default is not reapplied

#### Scenario: Filtering searches collapsed descendants
- **WHEN** the user filters by a filename or relative path that matches a descendant of a collapsed directory
- **THEN** the matching nested entry is included in the filtered results subject to the bounded row limit
- **AND** clearing the filter restores the prior collapse state

#### Scenario: Contained documents preserve the selected root
- **WHEN** a Markdown document inside the current workspace root is opened or focused
- **THEN** the current workspace root remains unchanged and the document is marked in the tree

#### Scenario: External document rebases the workspace
- **WHEN** a Markdown document outside the current workspace root is opened through an existing document-opening flow
- **THEN** the workspace root changes to that document's parent directory and the file tree rescans it
