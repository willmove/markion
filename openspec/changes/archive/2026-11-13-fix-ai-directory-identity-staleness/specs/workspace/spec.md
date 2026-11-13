## ADDED Requirements

### Requirement: Reviewed workspace directory identity survives child changes
AI workspace read grants and durable reviewed-action journals SHALL identify their root and scope directories by filesystem volume and object id. Creating or removing an ordinary child directory SHALL NOT invalidate an unchanged root or scope solely because its directory link count changes. Replacing a root or scope with another filesystem object SHALL invalidate the grant. Regular-file baseline comparisons, hard-link rejection, symlink/reparse rejection, containment, review and dependency checks SHALL remain enforced. Existing version-1 journal identities SHALL remain readable without a schema migration when only a recorded directory link count differs.

#### Scenario: Reviewed folder creation precedes note creation
- **WHEN** a reviewed plan creates a folder and then a note inside that folder
- **THEN** both operations can validate the unchanged workspace root and scope
- **AND** the durable journal can be reopened and the operations restored in dependency order

#### Scenario: Directory is replaced
- **WHEN** the granted directory is renamed away and a different directory is created at the original path
- **THEN** validation rejects the stale grant even if its path and link count match

#### Scenario: File has multiple hard links
- **WHEN** an existing text file has more than one hard link
- **THEN** the ordinary-file policy continues to reject that file
