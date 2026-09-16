# workspace

## Purpose

Covers the file tree panel and the auto-save / crash-recovery subsystem. The file tree supports left-drag moves into folders or the workspace root, plus Copy Path and Copy Relative Path on file and folder context menus.
## Requirements
### Requirement: File tree panel with filename filtering
The editor SHALL provide a toggleable file tree panel whose workspace root can be established by explicitly choosing File → Open Folder, by opening supported content outside the current workspace (from that content's parent directory), by restoring a previous session's workspace root on launch, or by a CLI folder open intent. The panel SHALL display Markdown files (`.md`/`.markdown`/`.mdown`), a curated set of plain-text files (`.txt`, `.text`, `.log`, `.csv`, `.tsv`, `.org`, `.rst`, `.adoc`, `.asciidoc`), and supported image files (`.png`, `.jpg`, `.jpeg`, `.gif`, `.webp`, `.bmp`, `.tif`, `.tiff`, `.svg`) nested under their containing folders. It SHALL list folders that exist on disk even when they contain no supported files, open Markdown and plain-text files on click as UTF-8 text, open supported image files on click as read-only images, mark the current file, support filename filtering, and support basic create / rename / delete / refresh operations for files and folders. Create File, Create Folder, and Rename SHALL collect a name through an in-app inline name editor: the editor renders inside the panel in place of the renamed row or directly below the parent folder row for create actions, and falls back to the top of the panel when that row is not visible; the editor is also rendered as a labeled prompt under the tab bar when the Files panel is hidden. Editing keys and clicks SHALL act on the name buffer only and SHALL NOT move the document caret or selection. Deleting a folder SHALL remove it recursively, including all of its contents, gated by a second confirmation for non-empty folders. Directories on a hard-coded ignore list (version-control, build-output, dependency/cache, and IDE directories) and hidden directories (whose name begins with `.`) SHALL NOT be listed. Other unsupported files (binaries, source code, and unsupported image formats) SHALL NOT appear in the tree. An explicitly selected workspace root SHALL be preserved while contained files are opened. The panel SHALL NOT scan the working directory on startup while only the in-memory welcome document is open and no session or CLI workspace root is available; instead it SHALL show an empty-state placeholder until a file or folder is opened or a session workspace root is restored. The panel SHALL let the user move a listed file or folder into another listed folder or the workspace root by left-dragging it onto that folder, onto a file inside that folder, or onto the workspace root. File and folder context menus SHALL include Copy Path and Copy Relative Path.

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

### Requirement: File tree entries SHALL move by left-drag onto a folder
The editor SHALL let the user left-drag a listed file or folder in the file tree and drop it onto a listed folder, onto a listed file (treated as that file's parent folder), or onto the workspace root (the workspace-name header or blank space in the Files panel). A successful drop SHALL move the entry on disk into that destination, keep its name, refresh the tree, and remap every open tab whose path was the moved entry or a descendant of a moved folder to the new location through the same remapping pipeline as rename. A left-click that does not start a drag SHALL continue to open a file or toggle a folder. Dropping onto the dragged entry itself, onto a descendant of a dragged folder, or onto the entry's current parent (including a sibling file) SHALL NOT move anything. A destination that already contains an entry with the same name SHALL be refused with localized status. If any open tab whose path would be remapped is dirty, the editor SHALL refuse the move and ask the user to save first. The inline name editor SHALL disable tree-row dragging while it is open. Bounded row rendering, derived Markdown caches, syntax-highlight memoization, and cached text handles SHALL remain unchanged except for the remapped tabs.

#### Scenario: Drag a file into a folder
- **WHEN** the user left-drags a listed file onto a different listed folder and releases
- **THEN** the file is moved into that folder on disk under its original name
- **AND** the file tree refreshes so the file appears under the destination
- **AND** the status bar reports a localized success message

#### Scenario: Drag a folder into another folder
- **WHEN** the user left-drags a listed folder onto a different listed folder that is not itself or a descendant
- **THEN** the folder and its contents are moved into the destination on disk
- **AND** the tree refreshes to show the new nesting

#### Scenario: Drop onto the workspace root
- **WHEN** the user left-drags a nested file or folder onto the workspace-name header or blank space in the Files panel
- **THEN** the entry is moved to the workspace root
- **AND** the tree refreshes

#### Scenario: Drop onto a file inside a folder moves into that folder
- **WHEN** the user left-drags a listed file or folder and drops it onto a listed file that lives in a different folder
- **THEN** the dragged entry is moved into that file's parent folder
- **AND** the tree refreshes so the entry appears under that parent

#### Scenario: Drop onto a collapsed folder still moves into it
- **WHEN** the user drops a file onto a collapsed folder
- **THEN** the file is moved into that folder
- **AND** the destination folder's collapsed state is unchanged

#### Scenario: A drag does not also open or toggle
- **WHEN** the user left-drags a file or folder and then releases on an invalid target or cancels the drag
- **THEN** that pointer gesture does not open the file or toggle the folder
- **AND** a separate left-click that never started a drag still opens or toggles as before

#### Scenario: Invalid drop targets leave the tree unchanged
- **WHEN** the user drops an entry onto itself, onto a descendant of a dragged folder, onto its current parent, or onto a sibling file in the same folder
- **THEN** no filesystem rename occurs
- **AND** the tree, open tabs, and document caches stay unchanged

#### Scenario: Name collision refuses the move
- **WHEN** the destination folder already contains an entry with the same name as the dragged entry
- **THEN** the editor does not move or silently rename the entry
- **AND** it reports a localized failure status

#### Scenario: Dirty tabs block the move
- **WHEN** the user drops a file that is open and dirty, or a folder that contains an open dirty tab
- **THEN** the editor refuses the move and reports a localized save-first status
- **AND** the entry remains at its original path

#### Scenario: Clean open tabs follow the moved path
- **WHEN** the user moves a clean open file, or a folder that contains a clean open file, to another folder
- **THEN** each matching tab is remapped to the new path in place
- **AND** unrelated tabs are unchanged

### Requirement: File tree context menu SHALL copy absolute and relative paths
The editor SHALL offer Copy Path and Copy Relative Path on the file-tree context menu for file and folder entries. Copy Path SHALL write the entry's platform-normal absolute path to the clipboard. Copy Relative Path SHALL write the path relative to the current workspace root, using platform path separators. Both actions SHALL operate on the right-clicked entry, report localized success or failure status, and SHALL NOT change document text, dirty state, undo history, or derived Markdown caches. The workspace/blank-space context menu SHALL NOT gain these actions.

#### Scenario: Copy Path writes the absolute path
- **WHEN** the user right-clicks a file or folder in the file tree and chooses Copy Path
- **THEN** the clipboard receives that entry's platform-normal absolute path
- **AND** the status bar reports a localized success message that includes the copied path

#### Scenario: Copy Relative Path writes the workspace-relative path
- **WHEN** the user right-clicks a file or folder under the current workspace root and chooses Copy Relative Path
- **THEN** the clipboard receives the path relative to the workspace root
- **AND** the status bar reports a localized success message that includes the copied relative path

#### Scenario: Path copy failure reports status
- **WHEN** Copy Path or Copy Relative Path cannot complete because the entry is gone, the relative path cannot be formed, or the clipboard write fails
- **THEN** the editor reports a localized failure status
- **AND** document text and the file tree stay unchanged

#### Scenario: Workspace menu does not copy an entry path
- **WHEN** the user right-clicks blank space in the Files panel
- **THEN** the context menu does not offer Copy Path or Copy Relative Path

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

### Requirement: Open documents SHALL observe external file changes safely
Markion SHALL periodically compare every named open tab with its last known on-disk identity and SHALL also compare synchronously before save. A clean tab whose file changed SHALL reload the new source in the same tab with user-facing status. A dirty tab, or a tab whose file disappeared, SHALL preserve its in-memory source and enter an explicit conflict state until the user chooses Reload, Overwrite, or Save a Copy.

#### Scenario: Clean tab reloads an external edit
- **WHEN** a named clean tab's on-disk bytes change externally
- **THEN** the same tab reloads the complete new source and refreshes its disk identity
- **AND** tab identity and unrelated tabs are preserved

#### Scenario: Dirty tab preserves both versions
- **WHEN** a named dirty tab's on-disk bytes change externally
- **THEN** Markion retains the dirty in-memory source and does not overwrite the disk file
- **AND** the conflict UI identifies the file and exposes Reload, Overwrite, and Save a Copy

#### Scenario: Open file is deleted externally
- **WHEN** the destination of an open tab disappears
- **THEN** Markion retains the in-memory document and reports the missing destination
- **AND** no automatic save recreates it without an explicit user action

### Requirement: Folder expansion reveals one level

When a folder is expanded interactively (by clicking it while collapsed), the file tree SHALL reveal only that folder's immediate children — its direct subfolders and the supported Markdown, plain-text, and image files it directly contains. Deeper subfolders SHALL remain collapsed until individually expanded, so each click drills down exactly one further level rather than opening the whole subtree at once. Collapsing a folder SHALL hide its entire subtree. This requirement governs interactive expansion only; the initial workspace-open collapse policy and filename filtering are unaffected.

#### Scenario: Expanding a collapsed folder reveals only its immediate children
- **WHEN** the user clicks a collapsed folder that contains nested subfolders and supported files
- **THEN** only that folder's direct children (immediate subfolders and directly contained supported files) become visible
- **AND** every deeper subfolder remains collapsed and its contents stay hidden

#### Scenario: Each click drills down exactly one more level
- **WHEN** the user clicks a now-visible collapsed subfolder that was revealed by the previous expand
- **THEN** only that subfolder's immediate children become visible
- **AND** levels deeper than it remain collapsed

#### Scenario: Collapsing a folder hides its entire subtree
- **WHEN** the user clicks an expanded folder
- **THEN** the folder's entire subtree is hidden, regardless of how deep individual descendants had been expanded

#### Scenario: Expanding a folder that contains only direct files
- **WHEN** the user clicks a collapsed folder that contains only supported files and no subfolders
- **THEN** those direct files become visible and no deeper structure is revealed

### Requirement: File tree hidden-entry visibility SHALL be preference-controlled

The file tree SHALL classify a file or folder as **hidden** when its file name begins with `.` (on every platform) or, on Windows, when the entry carries the hidden file attribute. Hidden entries SHALL be omitted from the file tree when the Show-hidden-files preference is **off** (the default), and SHALL be included when the preference is **on**, subject in both states to the supported-file filter and the always-excluded build/dependency noise list. Hidden-entry visibility SHALL apply identically to files and folders — hidden Markdown, curated text, and supported image files SHALL follow the same rule as hidden folders. The noise list (e.g. `target`, `node_modules`) SHALL remain excluded regardless of the preference.

#### Scenario: Hidden entries are omitted by default
- **WHEN** a workspace root contains hidden supported files and a hidden folder and the Show-hidden-files preference is off
- **THEN** neither the hidden files nor any entry under the hidden folder appears in the file tree
- **AND** non-hidden supported files and their ancestor folders continue to appear as before

#### Scenario: Toggling the preference on reveals hidden entries
- **WHEN** the user turns the Show-hidden-files preference on
- **THEN** hidden Markdown, curated text, and supported image files and their containing folders appear in the tree on the next scan
- **AND** the supported-file filter still excludes unsupported hidden files such as `.env`

#### Scenario: Toggling the preference off re-hides hidden entries
- **WHEN** the user turns the Show-hidden-files preference off after having revealed hidden entries
- **THEN** hidden files and folders are removed from the tree on the next scan
- **AND** the tree returns to the same visible set as the default-off state

#### Scenario: The build/dependency noise list stays excluded when hidden entries are revealed
- **WHEN** the Show-hidden-files preference is on and the workspace contains entries on the always-excluded noise list (e.g. `target/`, `node_modules/`)
- **THEN** those noise-list entries still do not appear in the file tree
- **AND** only OS-hidden entries that otherwise pass the supported-file filter are newly revealed

#### Scenario: A hidden folder and its contents are omitted together, revealed together
- **WHEN** a hidden folder contains supported files and the Show-hidden-files preference is off
- **THEN** neither the hidden folder nor any of its contents appear in the tree, because the scan never enters a skipped subtree
- **AND** when the preference is turned on, the hidden folder appears alongside its supported children

#### Scenario: Non-hidden folders are kept regardless of their children
- **WHEN** a non-hidden folder contains only a hidden supported file and the Show-hidden-files preference is off
- **THEN** the folder still appears in the tree because folders are not content-pruned, while its hidden child stays hidden
- **AND** when the preference is turned on, the hidden supported file appears under that folder

#### Scenario: Hidden-entry visibility persists across restarts
- **WHEN** the user sets the Show-hidden-files preference on, restarts the editor, and opens the same workspace
- **THEN** hidden supported files appear in the file tree without any further user action

### Requirement: Tab bar context menu

When two or more tabs are open, the editor SHALL show a context menu when the user right-clicks a tab-bar tab. The menu SHALL offer actions targeting the right-clicked tab: Close Tab, Close Others, Close to the Right, Rename, Copy File Path, and Reveal in File Manager. Activating an action SHALL first make the clicked tab the active tab, then perform the action (the same switch-then-operate idiom as the tab close button). The menu SHALL close on click-away, on Escape-equivalent dismissal paths, and when any other menu opens, and only one context menu SHALL be open at a time. Items that require a file-backed tab (Rename, Copy File Path, Reveal in File Manager) SHALL be disabled for untitled tabs. Middle-clicking a tab SHALL close it with the same behavior as the Close Tab action.

#### Scenario: Right-click opens the menu targeting the clicked tab

- **WHEN** two or more tabs are open and the user right-clicks a tab that is not active
- **THEN** a context menu appears at the pointer with all tab actions
- **AND** choosing Close Tab first activates the clicked tab and then closes it, running the existing dirty-document confirmation

#### Scenario: Untitled tab disables file-backed items

- **WHEN** the user right-clicks a tab whose document has never been saved to disk
- **THEN** Rename, Copy File Path, and Reveal in File Manager are visually disabled and dispatch nothing
- **AND** the close actions remain available

#### Scenario: Middle-click closes a tab

- **WHEN** the user middle-clicks a tab
- **THEN** that tab activates and closes exactly as the Close Tab context-menu action would

#### Scenario: Menu closes and stays exclusive

- **WHEN** a tab context menu is open and the user clicks elsewhere in the window or opens another menu (menu bar, file tree, preview)
- **THEN** the tab context menu closes without dispatching an action

### Requirement: Batch tab closing preserves dirty tabs by default

The Close Others and Close to the Right actions SHALL close only tabs with no unsaved changes. If one or more of the tabs in scope are dirty, the editor SHALL keep every dirty tab open, clean up only the clean tabs, and show a summary dialog stating how many tabs were kept because of unsaved changes. The dialog SHALL offer an explicit "discard all" choice that, when confirmed, closes the kept dirty tabs and discards their changes through the existing discard path (including recovery-file cleanup); declining it SHALL leave the dirty tabs open. Dirty tabs SHALL never be closed silently.

#### Scenario: Clean tabs close silently

- **WHEN** the user chooses Close Others and every other tab is clean
- **THEN** all other tabs close immediately with no dialog, and the clicked tab becomes the only tab

#### Scenario: Dirty tabs are kept and reported

- **WHEN** the user chooses Close Others and two other tabs have unsaved changes
- **THEN** all clean other tabs close, the two dirty tabs remain open
- **AND** a summary dialog reports the kept dirty tabs and offers a discard-all confirmation

#### Scenario: Discard all closes the dirty tabs

- **WHEN** the summary dialog is confirmed with the discard-all choice
- **THEN** the kept dirty tabs close and their recovery snapshots are discarded via the existing close/discard path
- **AND** declining the dialog leaves the kept dirty tabs open with their edits intact

### Requirement: Tab rename reuses the file rename pipeline

The tab-context-menu Rename action SHALL rename the tab's file on disk through the same pipeline as the file-tree rename: an inline name prompt, unique-name collision avoidance, refusal while the document has unsaved changes (with a save-first status message), and re-pointing every open tab that referenced the old path to the renamed file. The inline prompt SHALL be visible regardless of whether the file-tree panel is currently shown.

#### Scenario: Renaming a clean saved tab

- **WHEN** the user picks Rename on a clean, file-backed tab and confirms a new name in the inline prompt
- **THEN** the file is renamed on disk and the tab (and any duplicate tab for the old path) now refers to the renamed file, keeping its open state

#### Scenario: Dirty tab refuses rename

- **WHEN** the user picks Rename on a tab with unsaved changes
- **THEN** no prompt opens and a status message instructs the user to save first

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

### Requirement: Session snapshot includes chrome layout
The session file (`session.toml` under the Markion config directory) SHALL accept an optional `[layout]` table that records chrome geometry: window origin (`x`, `y`), window size (`width`, `height`), maximized flag, sidebar width, and editor/preview split ratio. Every field SHALL be optional. A missing table or missing field SHALL leave that value at the built-in default. Unknown extra keys SHALL be ignored. Loading a session for a CLI file or folder open intent SHALL still load `[layout]` even when document and workspace-root restore is skipped for that launch. Saving layout SHALL reuse the existing atomic session write and MUST NOT require a second session file.

#### Scenario: Layout table round-trips with the session file
- **WHEN** the editor persists window bounds, sidebar width, and split ratio
- **THEN** those values are written under `[layout]` in `session.toml`
- **AND** a subsequent load returns the same numeric values

#### Scenario: Older session files without layout still load
- **WHEN** `session.toml` exists but has no `[layout]` table
- **THEN** workspace-root, open-files, and recent-files fields load as before
- **AND** chrome geometry falls back to built-in defaults

#### Scenario: CLI open still restores layout
- **WHEN** the app launches with a CLI file or folder open intent and `session.toml` contains a valid `[layout]` table
- **THEN** the recorded chrome geometry is still applied
- **AND** conflicting document or workspace-root restore remains skipped as today
