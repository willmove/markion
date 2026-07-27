## ADDED Requirements

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

