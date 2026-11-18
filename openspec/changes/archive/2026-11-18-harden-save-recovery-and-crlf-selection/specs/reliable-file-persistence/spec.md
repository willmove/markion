## ADDED Requirements

### Requirement: Explicit saves SHALL be ordered with earlier autosaves
In-process document writes to the same destination SHALL serialize the external-change check and replacement. A successful explicit save SHALL leave its current source on disk and no earlier autosave of that tab SHALL subsequently replace it or update its saved identity. A failed explicit save SHALL retain recoverable work and normal external-conflict handling.

#### Scenario: Manual save overlaps an older autosave
- **WHEN** a pending or active autosave captured older text and the user explicitly saves the same tab
- **THEN** a successful explicit save leaves the newer source on disk and the document clean
- **AND** a delayed older completion leaves the disk identity and recovery state of the successful save intact

#### Scenario: An older internal save completes before the manual save
- **WHEN** the older autosave has replaced the destination and its completion has not yet reached the tab
- **THEN** the explicit save can use that internal write as its baseline
- **AND** external changes after that write still produce a conflict

### Requirement: Unrelated recoveries SHALL remain distinct across launches
A tab's first recovery snapshot SHALL have a distinct path from every retained snapshot. Only a tab that owns or restores a snapshot SHALL update that snapshot. Previously written recovery files SHALL remain readable.

#### Scenario: A retained untitled draft survives a new draft's autosave
- **WHEN** an untitled recovery remains in the inventory after restart and an unrelated untitled draft autosaves
- **THEN** each draft is recoverable with its own complete text
- **AND** subsequent autosaves of the new draft update its snapshot without replacing the retained draft
