## ADDED Requirements

### Requirement: Git conflicts SHALL own a durable resolution session
An app-started merge conflict SHALL create a session tied to repository/operation identity and base/local/remote commits with indexed conflict stages. It SHALL persist editable drafts outside the repository and keep conflict-owned paths out of ordinary document autosave. The app SHALL distinguish this session from an existing unsaved-buffer/external-disk conflict, which must be resolved before integration.

#### Scenario: Same note conflicts across devices
- **WHEN** Git cannot automatically merge two versions of a note
- **THEN** the app presents local, base, remote and editable result content in a conflict session
- **AND** ordinary tabs for that path cannot overwrite the conflict result through autosave

#### Scenario: App closes with an unfinished resolution
- **WHEN** a conflict draft has been durably saved and the app restarts
- **THEN** the matching session and draft remain recoverable after actual repository-state verification

### Requirement: Conflict choices SHALL reflect content and path semantics
The resolver SHALL support text hunk choices and manual source editing, delete/modify decisions, image/binary side selection or distinct-path preservation, and explicit simple rename choices. Unsupported complex/encoding/path conflicts SHALL remain unresolved with external-tool guidance. Draft saving SHALL NOT mark a conflict resolved. Explicit resolution SHALL validate current identities, write the chosen result through repository coordination, and clear the corresponding index conflict stages. Marker text alone SHALL NOT determine resolution state.

#### Scenario: User keeps both text fragments
- **WHEN** the user selects both sides of a text conflict
- **THEN** the result remains editable and reviewable before explicit resolution, without a promise that Markdown structure or links are automatically correct

#### Scenario: Remote deletion conflicts with local modification
- **WHEN** one side deletes a note and the other modifies it
- **THEN** the app offers an explicit delete-or-preserve decision and does not choose by timestamp

#### Scenario: Image versions conflict
- **WHEN** both sides change a binary image
- **THEN** the app shows available previews/metadata and permits side selection or validated distinct-path copies without textual merging

#### Scenario: Document legitimately quotes merge markers
- **WHEN** a user resolves the Git index conflict while keeping literal marker text in the document
- **THEN** resolved state follows the index and explicit user action rather than marker-string detection alone

### Requirement: Merge completion and abort SHALL preserve recoverable work
Finish Merge SHALL require no remaining index conflicts and current written draft versions, respect author/signing/hooks, and create the actual merge commit. After interactive resolution the app SHALL expose explicit Finish and Push intent. Abort SHALL durably preserve edited drafts, verify app-owned merge state, and preserve the pre-merge local snapshot commit. External state drift SHALL block unattended completion/abort instead of triggering reset/clean.

#### Scenario: All conflicts are resolved
- **WHEN** every conflict is resolved and the user chooses Finish and Push
- **THEN** the app verifies the session, creates the merge commit, and resumes fixed-target push under normal checks

#### Scenario: User aborts a merge after editing the result
- **WHEN** the user cancels an app-owned merge with edited drafts
- **THEN** drafts remain recoverable and successful abort leaves the local snapshot commit available for later synchronization

#### Scenario: External Git changed the merge state
- **WHEN** the actual HEAD or merge/index state no longer matches the app session
- **THEN** completion/abort pauses for reconciliation without discarding external changes

### Requirement: Operation checkpoints SHALL survive partial completion
The app SHALL durably record intent and confirmation around staging, committing, integration and push, including operation/target identity, relevant OIDs and file/index identities. Required recovery preimages and commit references SHALL be persisted before destructive worktree replacement; failure to persist them SHALL block the mutation. The journal SHALL exclude credentials and SHALL not be treated as more authoritative than the actual repository. Recovery SHALL avoid automatically overwriting paths that changed after the checkpoint.

#### Scenario: Recovery storage is full before integration
- **WHEN** the required integration preimages cannot be durably stored
- **THEN** integration does not begin and existing repository content remains available

#### Scenario: Crash occurs immediately after a commit
- **WHEN** the commit exists but its journal confirmation was not persisted
- **THEN** restart inspects actual parent/tree/HEAD state and recognizes completed work instead of creating a duplicate snapshot commit

#### Scenario: Interrupted checkout left mixed files
- **WHEN** a checkout/integration process stops before all expected worktree effects are confirmed
- **THEN** restart compares actual paths, refs and index with recorded checkpoints and offers safe recovery or attention
- **AND** it does not blindly reset/clean the worktree or overwrite independently changed files

### Requirement: Restart reconciliation SHALL preserve pending work and uncertain delivery
Startup SHALL inspect pending operation records and actual repository state before permitting further app mutations in that repository. It SHALL recover owned staging, conflict sessions and incomplete integration, verify uncertain push delivery, and retain local commits on network failure. Unresolved drafts/recovery SHALL remain until explicit resolution/discard; successful summaries SHALL be bounded. Disconnecting sync SHALL NOT delete pending recovery or Git history.

#### Scenario: Remote is unavailable after a local commit
- **WHEN** synchronization fails after a confirmed local commit
- **THEN** restart shows its pending delivery and can retry networking without another content commit

#### Scenario: A pending operation differs from the repository
- **WHEN** restart detects independent repository changes after an interrupted operation
- **THEN** it reports needs-attention state and preserves recovery material instead of rolling back to journal assumptions

#### Scenario: User disconnects with a pending conflict
- **WHEN** a user disconnects a repository that still has unresolved recovery
- **THEN** scheduled checks stop but the conflict/recovery remains accessible and ordinary writes to conflict-owned paths remain guarded
