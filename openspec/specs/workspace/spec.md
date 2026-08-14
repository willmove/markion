# workspace

## Purpose

Covers the file tree panel and the auto-save / crash-recovery subsystem. Drag-and-drop file moves and a UI affordance for moving entries are **not** part of this capability (the underlying move API exists but is not surfaced); they are future candidates.
## Requirements
### Requirement: File tree panel with filename filtering
The editor SHALL provide a toggleable file tree panel whose workspace root can be established either by explicitly choosing File → Open Folder or, when opening supported content outside the current workspace, from that content's parent directory. The panel SHALL display Markdown files (`.md`/`.markdown`/`.mdown`), a curated set of plain-text files (`.txt`, `.text`, `.log`, `.csv`, `.tsv`, `.org`, `.rst`, `.adoc`, `.asciidoc`), and supported image files (`.png`, `.jpg`, `.jpeg`, `.gif`, `.webp`, `.bmp`, `.tif`, `.tiff`, `.svg`) nested under their containing folders. It SHALL list folders that exist on disk even when they contain no supported files, open Markdown and plain-text files on click as UTF-8 text, open supported image files on click as read-only images, mark the current file, support filename filtering, and support basic create / rename / delete / refresh operations for files and folders. Directories on a hard-coded ignore list (version-control, build-output, dependency/cache, and IDE directories) and hidden directories (whose name begins with `.`) SHALL NOT be listed. Other unsupported files (binaries, source code, and unsupported image formats) SHALL NOT appear in the tree. An explicitly selected workspace root SHALL be preserved while contained files are opened. The panel SHALL NOT scan the working directory on startup while only the in-memory welcome document is open; instead it SHALL show an empty-state placeholder until a file or folder is opened. Moving entries via the UI is **not** supported.

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

