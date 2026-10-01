# reliable-file-persistence Specification

## Purpose
TBD - created by archiving change complete-p0-editor-workflows. Update Purpose after archive.

## Requirements

### Requirement: Document saves SHALL be atomic and failure-safe
Saving or saving as Markdown SHALL write a same-directory temporary file and atomically replace the destination. A failed write, flush, or replacement SHALL leave the previous destination content intact where the platform permits, SHALL clean up temporary files, and SHALL retain the document's prior path and dirty state.

#### Scenario: Existing file is replaced atomically
- **WHEN** a named document is saved successfully
- **THEN** the destination contains the complete current UTF-8 source
- **AND** no partial content or temporary file remains

#### Scenario: Save fails before replacement
- **WHEN** the temporary write or flush fails
- **THEN** the existing destination remains unchanged
- **AND** the document remains dirty and reports the error

### Requirement: Saves SHALL detect conflicting external modifications
Each named document SHALL remember the identity of the disk content it opened or last saved. Before any explicit or automatic save, Markion SHALL compare the current destination with that identity and SHALL refuse an ordinary write when content changed or disappeared externally. A metadata-only touch with identical bytes SHALL NOT create a conflict.

#### Scenario: Dirty document changed externally
- **WHEN** the on-disk bytes change after a document is opened and the in-memory document also has unsaved edits
- **THEN** ordinary save and autosave do not overwrite the external bytes
- **AND** the user is offered Reload, Overwrite, and Save a Copy actions

#### Scenario: Metadata changed but content is identical
- **WHEN** file metadata changes while the on-disk bytes remain identical to the last known content
- **THEN** the next save proceeds without presenting a false conflict

### Requirement: Recovery snapshots SHALL preserve dirty work without overriding newer disk content
Every dirty tab SHALL maintain an atomically replaced recovery snapshot with enough original-path and disk-identity metadata to evaluate it on restart. Successful durable saves and intentional discards SHALL retire the corresponding snapshot. Startup SHALL restore useful recovery content as dirty in-memory work and SHALL NOT overwrite or silently replace a diverged source file.

#### Scenario: Named dirty tab survives interruption
- **WHEN** the application stops after writing a recovery snapshot but before the document is saved
- **THEN** startup offers or restores the recovered source with its original path context
- **AND** the original disk file is not modified

#### Scenario: Successful save retires recovery
- **WHEN** a document save completes and its disk identity is refreshed
- **THEN** the tab's obsolete recovery snapshot is removed

### Requirement: Recovery inventory SHALL be individually manageable and remain durable
On startup Markion SHALL inventory every recovery snapshot and present readable and unreadable entries in one recovery manager with original-path and disk-relationship context where available. The user SHALL be able to restore or discard each entry independently and SHALL also have Restore All and Discard All actions. An unreadable or unselected snapshot SHALL remain on disk. A restored snapshot SHALL remain durable until a successful document save, explicit discard, or successfully written successor recovery replaces it.

#### Scenario: User restores selected snapshots
- **WHEN** multiple readable recovery snapshots exist and the user restores one entry
- **THEN** only that recovered document is opened or attached to its matching session tab
- **AND** every other snapshot remains available in the manager and on disk

#### Scenario: Restored work survives another immediate interruption
- **WHEN** a recovery snapshot is restored as a dirty document and Markion stops before another autosave
- **THEN** the original recovery snapshot still exists for the next launch
- **AND** it is retired only after a durable save, explicit discard, or durable successor recovery

#### Scenario: Unreadable snapshot is retained
- **WHEN** one recovery file cannot be parsed or read
- **THEN** the manager identifies it as unreadable and does not delete it during Restore All
- **AND** the user may explicitly discard it

#### Scenario: Matching session tab remains unique
- **WHEN** session restore already opened the original path for a recovery entry
- **THEN** restoring that entry replaces and activates the matching clean tab in place
- **AND** no duplicate path-backed tab is created and the disk file remains unchanged

### Requirement: Atomic replacement SHALL preserve destination permissions
When replacing an existing destination atomically, Markion SHALL apply the destination's existing filesystem permissions to the complete temporary file before replacement where the platform exposes those permissions. A permission-copy failure SHALL abort replacement and preserve the old destination. New destinations SHALL retain ordinary platform creation defaults.

#### Scenario: Existing permissions survive save
- **WHEN** an existing Markdown or settings file has non-default supported permissions and is atomically replaced
- **THEN** the new complete file retains those permissions
- **AND** no temporary file remains

#### Scenario: Permission preparation fails
- **WHEN** destination permissions cannot be applied to the temporary file before replacement
- **THEN** the old destination bytes remain unchanged
- **AND** the save reports failure without clearing dirty state

### Requirement: Git worktree mutations SHALL drain every admitted application writer
Before a Git worktree mutation, Markion SHALL block new writes into that repository and wait for previously admitted writes and their completions to settle. Manual/automatic saves, Save As targets, quit-save, file operations, image imports/replacements, exports into the repository and historical-copy writes SHALL use this admission boundary. Git mutation SHALL revalidate disk/buffer identities and preserve required recovery before writing. Disabling future autosave timers alone SHALL NOT satisfy this requirement.

#### Scenario: Autosave is already writing
- **WHEN** an autosave has started its disk write before Git requests exclusive worktree admission
- **THEN** Git waits for that write and its completion to settle before capturing its disk baseline and updating files
- **AND** the old autosave cannot subsequently overwrite the Git result

#### Scenario: Save As or export targets a locked repository
- **WHEN** a save/export from another workspace selects a destination inside a repository being updated by Git
- **THEN** the destination write uses that repository's admission boundary rather than bypassing it because the source document is elsewhere

#### Scenario: User quits during a local Git mutation
- **WHEN** quit-save is requested during checkout or merge file replacement
- **THEN** the app waits for a safe local mutation boundary or records resumable state before admitting saves or exiting
- **AND** it does not require successful completion of a remote upload to preserve local content

### Requirement: Git reconciliation SHALL reject stale external reloads and preserve caches
External-file reads SHALL carry repository operation epochs in addition to document identities/versions when applicable. Results from before a Git mutation SHALL NOT reload afterward. Accepted content updates SHALL pass through checked document lifecycle boundaries, update disk identities, and refresh only affected derived state. Git status-only changes SHALL NOT change document versions, undo state, syntax memoization, or cached text handles.

#### Scenario: Old disk read returns after pull
- **WHEN** a pre-pull external-file read completes after Git installed newer content
- **THEN** its stale repository epoch prevents application of the old bytes or disk identity

#### Scenario: Unrelated open document is unchanged
- **WHEN** Git updates one file and another open document's source bytes are unchanged
- **THEN** only the changed document is reloaded and the unrelated document retains its version and cached derived state

### Requirement: Git deletions and renames SHALL not silently destroy or recreate open content
An unambiguously renamed clean open file SHALL be remapped in place after checked reconciliation. A remotely deleted open file SHALL retain recoverable content in a missing-destination state and SHALL NOT be automatically recreated by save/recovery timers. Ambiguous renames SHALL be treated as explicit deletion/addition. Dirty or externally diverged buffers SHALL not be overwritten by reconciliation.

#### Scenario: Remote deletes an open note
- **WHEN** a safe pull removes a clean note that is open locally
- **THEN** its retained source is available for recovery or Save a Copy and normal autosave does not recreate the deleted path

#### Scenario: Remote renames an open note
- **WHEN** a safe pull identifies an unambiguous rename of a clean open note
- **THEN** the tab is remapped to the new path without altering unrelated tabs
