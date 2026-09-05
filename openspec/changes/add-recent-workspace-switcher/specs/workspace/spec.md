## ADDED Requirements

### Requirement: Session restore uses per-workspace snapshots
The editor SHALL persist workspace continuity in `session.toml` under the Markion config directory as a bounded list of workspace snapshots (most recent first). Each snapshot SHALL record a workspace root, the ordered list of open **path-backed** tabs whose paths sit inside that root (Markdown, curated plain-text, and supported images), and the active path when it is among those tabs. Untitled, welcome, and recovery-only tabs SHALL NOT be written into a snapshot. Cursor, selection, scroll, undo history, dirty buffer text, view mode, and file-tree expand/collapse SHALL NOT be stored; dirty text continues to use crash recovery. On launch with no CLI file or folder open intent, the editor SHALL restore the current (most recent) snapshot: re-establish a still-valid workspace root and scan it asynchronously, reopen still-existing snapshot paths as tabs in the recorded order, and focus the recorded active path when it is among the restored tabs. Missing roots and missing tab paths SHALL be skipped. A CLI file or folder open intent SHALL take precedence over conflicting document and workspace-root restore for that launch while still loading the recent-workspace list and recent-files list. Crash-recovery prompting SHALL continue to run after session restore. A legacy session that has top-level workspace-root / open-files / active-file fields and no workspace list SHALL be treated as a single snapshot.

#### Scenario: Current workspace restores on launch
- **WHEN** the previous session recorded a current workspace root that still exists and the app launches without a CLI open intent
- **THEN** that directory becomes the file-tree workspace root and is scanned asynchronously
- **AND** the Files panel shows the restored workspace instead of the empty-state placeholder

#### Scenario: Path-backed tabs restore from the current snapshot
- **WHEN** the current workspace snapshot recorded one or more Markdown, curated text, or supported image paths that still exist and the app launches without a CLI file open intent
- **THEN** those paths reopen as tabs in the recorded order
- **AND** the recorded active path is focused when it is among the restored tabs

#### Scenario: Missing session paths are skipped
- **WHEN** a recorded workspace root or snapshot tab path no longer exists at launch
- **THEN** that path is skipped without failing startup
- **AND** any remaining valid session paths still restore

#### Scenario: CLI open intent overrides session restore
- **WHEN** the app launches with a CLI file or folder open intent and a session snapshot also exists
- **THEN** the CLI intent is applied for the requested file or folder
- **AND** conflicting session restore for those fields is not applied on that launch
- **AND** the recent-workspace list remains available for later switching

#### Scenario: Untitled tabs are not persisted
- **WHEN** the only open tab is an untitled welcome or recovery document with no saved path
- **THEN** the current workspace snapshot does not record that tab as an open file
- **AND** a previously recorded workspace root may still be persisted and restored

#### Scenario: Legacy single-snapshot sessions still load
- **WHEN** `session.toml` has top-level workspace-root and open-files fields and no workspace list
- **THEN** the editor treats those fields as the current workspace snapshot
- **AND** the next session save writes the workspace list

### Requirement: Explicit workspace switch restores that folder's tabs
Choosing a workspace from the Files-panel header, from the file-tree empty-state recent list, or via File → Open Folder SHALL be an **explicit workspace switch**. Before changing the root, the editor SHALL write the current workspace snapshot (in-root path-backed tabs only). The editor SHALL then close only tabs with no unsaved changes and SHALL keep every dirty document tab on the tab bar with its text, dirty flag, undo history, and recovery snapshot intact. An explicit workspace switch SHALL NOT show Save / Don't Save / Cancel; that prompt SHALL remain on close-tab and quit / window close. The editor SHALL then open the target snapshot: a remembered directory restores its surviving recorded paths in order, reuses an already-open tab for the same path, and focuses its recorded active path when that path is among the resulting tabs; a directory with no snapshot (or whose recorded paths are all missing) SHALL show a welcome tab only when no tabs remain. The target directory then becomes the file-tree workspace root and is scanned asynchronously. Selecting the already-current workspace SHALL close the switcher and SHALL NOT replace tabs. Derived Markdown caches, syntax-highlight memoization, and cached text handles SHALL be built only through the existing open/close APIs for newly opened or closed tabs.

#### Scenario: Switching to a remembered workspace restores its tabs
- **WHEN** the user selects a recent workspace that has a stored snapshot with surviving paths
- **THEN** the current workspace snapshot is written first
- **AND** clean tabs are closed and those snapshot paths open in the recorded order
- **AND** the file tree scans the selected root

#### Scenario: Switching to a new folder starts an empty session
- **WHEN** the user chooses File → Open Folder and selects a directory that has no stored workspace snapshot
- **THEN** the current workspace snapshot is written first
- **AND** clean tabs are closed
- **AND** a welcome document is shown only when no tabs remain
- **AND** that directory becomes the current workspace root and is scanned

#### Scenario: Dirty tabs stay open across a workspace switch
- **WHEN** the user starts an explicit workspace switch and one or more document tabs have unsaved changes
- **THEN** those dirty tabs remain open with their text, dirty flag, undo history, and recovery snapshots unchanged
- **AND** the editor does not show Save / Don't Save / Cancel
- **AND** clean tabs are closed and the target snapshot's surviving paths are opened
- **AND** an already-open dirty tab for a snapshot path is reused instead of being reloaded from disk

#### Scenario: Selecting the current workspace is a no-op
- **WHEN** the user chooses the already-current workspace from the Files header
- **THEN** the switcher closes
- **AND** tabs, dirty state, and the file tree remain unchanged

#### Scenario: A vanished recent workspace is pruned
- **WHEN** the user selects a recent workspace whose root is no longer a directory
- **THEN** the editor reports localized failure status
- **AND** that entry is removed from the recent-workspace list
- **AND** the current workspace and tabs remain unchanged

### Requirement: Files header offers a recent-workspace switcher
When a workspace root is established, the Files-panel workspace-name header SHALL present the current folder name and SHALL open a localized switcher listing the bounded recent workspaces (current folder marked, most recent first) plus an Open Folder action. Activating a listed folder SHALL perform an explicit workspace switch. The header SHALL remain a valid drop target for file-tree moves onto the workspace root; opening the switcher SHALL require a click that is not a drag. When no workspace root is established and the recent-workspace list is non-empty, the file-tree empty state SHALL list those folders as the same explicit-switch actions. File → Open Recent SHALL continue to list recent files only.

#### Scenario: Header lists recent workspaces
- **WHEN** a workspace root is established and the user opens the Files-panel workspace-name switcher
- **THEN** the current folder is marked
- **AND** other recent workspace roots appear with the most recent first
- **AND** Open Folder is available from that switcher

#### Scenario: Empty state lists recent workspaces
- **WHEN** the file tree has no established root and the recent-workspace list is non-empty
- **THEN** the empty state lists those folders
- **AND** choosing one performs an explicit workspace switch

## MODIFIED Requirements

### Requirement: File tree panel with filename filtering
The editor SHALL provide a toggleable file tree panel whose workspace root can be established by File → Open Folder, by choosing a recent workspace from the Files-panel header or empty state, by restoring the current workspace on launch, or, when opening supported content outside the current workspace, from that content's parent directory (implicit rebase). The panel SHALL display Markdown files (`.md`/`.markdown`/`.mdown`), a curated set of plain-text files (`.txt`, `.text`, `.log`, `.csv`, `.tsv`, `.org`, `.rst`, `.adoc`, `.asciidoc`), and supported image files (`.png`, `.jpg`, `.jpeg`, `.gif`, `.webp`, `.bmp`, `.tif`, `.tiff`, `.svg`) nested under their containing folders. It SHALL list folders that exist on disk even when they contain no supported files, open Markdown and plain-text files on click as UTF-8 text, open supported image files on click as read-only images, mark the current file, support filename filtering, and support basic create / rename / delete / refresh operations for files and folders. Create File, Create Folder, and Rename SHALL collect a name through an in-app inline name editor: the editor renders inside the panel in place of the renamed row or directly below the parent folder row for create actions, and falls back to the top of the panel when that row is not visible; the editor is also rendered as a labeled prompt under the tab bar when the Files panel is hidden. Editing keys and clicks SHALL act on the name buffer only and SHALL NOT move the document caret or selection. Deleting a folder SHALL remove it recursively, including all of its contents, gated by a second confirmation for non-empty folders. Directories on a hard-coded ignore list (version-control, build-output, dependency/cache, and IDE directories) and hidden directories (whose name begins with `.`) SHALL NOT be listed. Other unsupported files (binaries, source code, and unsupported image formats) SHALL NOT appear in the tree. An explicitly selected workspace root SHALL be preserved while contained files are opened. The panel SHALL NOT scan the working directory on startup while only the in-memory welcome document is open and no restorable session workspace root is available; instead it SHALL show the recent-workspace list when one exists, or an empty-state placeholder, until a file or folder is opened or a session workspace is restored. When a workspace root is established, the workspace-name header SHALL offer the recent-workspace switcher. The panel SHALL let the user move a listed file or folder into another listed folder or the workspace root by left-dragging it onto that folder, onto a file inside that folder, or onto the workspace root. File and folder context menus SHALL include Copy Path and Copy Relative Path.

#### Scenario: Workspace scan displays supported files
- **WHEN** a workspace root is established via File → Open Folder, a recent-workspace switch, session restore, the File → Open dialog, the sidebar, or Save As
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
- **WHEN** the editor launches with the in-memory welcome document and no file, folder, or restorable session workspace root is available
- **THEN** the file tree does not scan the working directory
- **AND** when recent workspaces exist, the empty state lists those folders for explicit switch
- **AND** when the recent-workspace list is empty, it shows the empty-state placeholder instead of the directory hierarchy

#### Scenario: Open folder establishes the workspace and reveals Files
- **WHEN** the user chooses File → Open Folder and selects one directory
- **THEN** that directory becomes the file-tree workspace root through an explicit workspace switch
- **AND** the left sidebar becomes visible on the Files tab
- **AND** the selected directory is scanned asynchronously, including when it contains no supported files

#### Scenario: Folder selection cancellation preserves state
- **WHEN** the user cancels the Open Folder picker
- **THEN** the current workspace root, file tree, sidebar selection, active content, and every document's dirty state and undo history remain unchanged
- **AND** the editor reports localized cancellation feedback

#### Scenario: Folder scan failure is non-destructive
- **WHEN** the selected directory cannot be scanned
- **THEN** the editor reports a localized failure status
- **AND** a completed explicit workspace switch is not rolled back
- **AND** remaining documents' dirty state, undo history, and derived Markdown caches remain unchanged

#### Scenario: Contained files preserve the selected root
- **WHEN** a supported document or image inside the current workspace root is opened or focused
- **THEN** the current workspace root remains unchanged and the file is marked in the tree

#### Scenario: External file rebases the workspace
- **WHEN** supported content outside the current workspace root is opened through an existing interactive file-opening flow
- **THEN** the workspace root changes to that file's parent directory and the file tree rescans it
- **AND** the live tab set is not replaced by that parent directory's stored snapshot

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

### Requirement: Session snapshot includes chrome layout
The session file (`session.toml` under the Markion config directory) SHALL accept an optional `[layout]` table that records chrome geometry: window origin (`x`, `y`), window size (`width`, `height`), maximized flag, sidebar width, and editor/preview split ratio. Every field SHALL be optional. A missing table or missing field SHALL leave that value at the built-in default. Unknown extra keys SHALL be ignored. The workspace-snapshot list SHALL be stored in the same file and MUST NOT require a second session file. Loading a session for a CLI file or folder open intent SHALL still load `[layout]` even when document and workspace-root restore is skipped for that launch. Saving layout SHALL reuse the existing atomic session write.

#### Scenario: Layout table round-trips with the session file
- **WHEN** the editor persists window bounds, sidebar width, and split ratio
- **THEN** those values are written under `[layout]` in `session.toml`
- **AND** a subsequent load returns the same numeric values

#### Scenario: Older session files without layout still load
- **WHEN** `session.toml` exists but has no `[layout]` table
- **THEN** workspace-root, open-files, recent-files, and workspace-snapshot fields load as before
- **AND** chrome geometry falls back to built-in defaults

#### Scenario: CLI open still restores layout
- **WHEN** the app launches with a CLI file or folder open intent and `session.toml` contains a valid `[layout]` table
- **THEN** the recorded chrome geometry is still applied
- **AND** conflicting document or workspace-root restore remains skipped as today
