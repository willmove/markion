## ADDED Requirements

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

