## ADDED Requirements

### Requirement: File tree context menu SHALL duplicate a file or folder
The editor SHALL offer Duplicate on the file-tree context menu for file and folder entries. Choosing it SHALL copy the right-clicked entry on disk into the same parent directory under a new name that does not collide with an existing entry. The first free name SHALL insert a localized copy marker before the extension (`notes.md` becomes a localized form such as `notes - 副本.md` or `notes - copy.md`); further collisions SHALL append an incrementing number. A folder duplicate SHALL copy that folder and its on-disk contents, including files the tree does not list. The copy SHALL use the bytes currently stored on disk and SHALL NOT read unsaved editor buffers. After a successful copy the tree SHALL refresh, the new entry SHALL become the selected tree entry, and the status bar SHALL report localized success. Open tabs SHALL keep their existing paths, text, dirty state, undo history, and derived Markdown caches. The workspace and blank-space context menu SHALL NOT offer Duplicate. If the source is missing, the destination cannot be created, or the copy fails, the editor SHALL report a localized failure status and SHALL NOT leave a partial duplicate behind.

#### Scenario: Duplicate a file beside itself
- **WHEN** the user right-clicks a file in the file tree and chooses Duplicate
- **THEN** a new file appears in the same folder with a localized non-colliding copy name
- **AND** the new file's bytes match the source file on disk
- **AND** the tree selects the new file and the status bar reports localized success
- **AND** open tabs for the source path stay on that path with unchanged text, dirty state, and undo history

#### Scenario: Duplicate a folder recursively
- **WHEN** the user right-clicks a folder and chooses Duplicate
- **THEN** a new sibling folder is created with a localized non-colliding copy name
- **AND** the new folder contains a copy of the source folder's on-disk contents
- **AND** the tree refreshes and selects the new folder

#### Scenario: A second duplicate picks the next free name
- **WHEN** the user duplicates an entry whose first localized copy name already exists
- **THEN** the editor creates the copy under the next free numbered name
- **AND** the existing copy is left unchanged

#### Scenario: Duplicate copies disk contents, not unsaved edits
- **WHEN** the user duplicates a file that is open with unsaved edits
- **THEN** the new file matches the source file on disk
- **AND** the open tab keeps its unsaved text and dirty state

#### Scenario: Duplicate failure removes a partial copy
- **WHEN** Duplicate cannot finish because the source is gone or the copy fails
- **THEN** the editor reports a localized failure status
- **AND** no partial duplicate remains
- **AND** document text, dirty state, undo history, and derived Markdown caches stay unchanged

#### Scenario: Workspace menu does not duplicate
- **WHEN** the user right-clicks blank space in the Files panel
- **THEN** the context menu does not offer Duplicate

### Requirement: File tree SHALL refresh when the workspace changes on disk
While a workspace root is open, the editor SHALL detect create, delete, and rename changes under that root and refresh the file tree without a manual Refresh. Detection SHALL watch the workspace directory, and SHALL fall back to a periodic refresh when a watch cannot be started. Automatic refreshes SHALL reuse the existing background scan, preserve collapse state, keep bounded row rendering, and SHALL NOT replace the status bar text with the manual-refresh message. Manual Refresh SHALL continue to refresh the tree and report its existing localized status. Changing or clearing the workspace root SHALL stop watching or polling the previous root. An automatic refresh SHALL NOT reload open document text, dirty state, undo history, or derived Markdown caches.

#### Scenario: An external create appears in the tree
- **WHEN** a supported file or a folder is created under the open workspace root outside the editor
- **THEN** the file tree updates to show it without the user choosing Refresh
- **AND** the status bar does not switch to the manual-refresh message

#### Scenario: An external delete or rename updates the tree
- **WHEN** a listed file or folder under the open workspace root is deleted or renamed outside the editor
- **THEN** the file tree updates to match the disk
- **AND** collapse state of unaffected folders is preserved

#### Scenario: Watch failure falls back to a timed refresh
- **WHEN** the editor cannot watch the open workspace root
- **THEN** it refreshes the file tree on a timer until the root changes or a watch can be started
- **AND** manual Refresh still works

#### Scenario: Leaving the workspace stops updates for the old root
- **WHEN** the workspace root changes or is cleared
- **THEN** the editor stops watching or polling the previous root
- **AND** a late scan of the previous root does not replace the current tree

#### Scenario: Automatic refresh leaves open documents alone
- **WHEN** the file tree refreshes because of a watched or timed update
- **THEN** open document text, dirty state, undo history, and derived Markdown caches stay unchanged
