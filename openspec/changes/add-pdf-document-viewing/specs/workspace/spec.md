## ADDED Requirements

### Requirement: Registered PDF files SHALL participate in generic workspace handling
The signed immutable file-handler registry SHALL declare `.pdf`
case-insensitively as an official plugin-document type. A workspace scan SHALL
consume one registry snapshot and include registered PDFs alongside supported
core files without process or network work per row. Discovery SHALL retain the
existing hierarchy, ignore/hidden rules, filtering, selection, context actions,
bounded rows, and current-file styling.

#### Scenario: Provider is not installed
- **WHEN** a workspace contains `.pdf`, `.PDF`, or mixed-case PDF files and the official provider is absent
- **THEN** those rows remain visible as known plugin documents
- **AND** opening one creates a closable provider-missing tab and exposes the plugin manager without downloading code

#### Scenario: Provider is installed
- **WHEN** the current registry resolves a PDF to an enabled compatible provider
- **THEN** the normal default-target or forced-new-tab rule opens/focuses its generic plugin-document tab
- **AND** duplicate path identity prevents a second native document from opening

#### Scenario: Registry snapshot changes
- **WHEN** a verified catalog refresh changes registered extensions or lifecycle state
- **THEN** a subsequent background scan uses one new immutable snapshot
- **AND** an in-flight scan remains internally consistent

### Requirement: PDF paths SHALL use generic path-backed lifecycle and sessions
Open PDF paths SHALL participate in recent files, ordered workspace snapshots,
active-path selection, rename/move remapping, deletion, duplicate focus, and
missing-path pruning without adding a PDF-specific session schema field.
Restoration SHALL reclassify saved paths through the current registry.

#### Scenario: Workspace snapshot restores a PDF
- **WHEN** a snapshot records an existing in-root PDF path
- **THEN** restoration reopens it in recorded order as ready or provider-unavailable according to the current registry/store
- **AND** missing or no-longer-supported paths do not prevent other tabs from restoring

#### Scenario: PDF path is renamed or deleted
- **WHEN** an existing Files-panel action renames, moves, or deletes an open PDF
- **THEN** the generic tab path/removal pipeline updates it without a dirty-document prompt
