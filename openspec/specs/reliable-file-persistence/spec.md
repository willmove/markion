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

