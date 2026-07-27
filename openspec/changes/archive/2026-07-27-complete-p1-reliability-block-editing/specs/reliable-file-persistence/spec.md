## ADDED Requirements

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
