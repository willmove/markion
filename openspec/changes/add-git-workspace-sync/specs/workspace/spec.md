## ADDED Requirements

### Requirement: Workspace synchronization SHALL retain explicit repository ownership
Workspace sync controls SHALL act on the explicitly connected workspace repository, independently of the active document's repository. Operations and callbacks SHALL retain captured repository identity and operation epoch across tab/workspace switches. Eligible buffers SHALL be selected by resolved path/repository membership and policy, not by all visible tabs. Foreign and untitled buffers SHALL retain their source, undo state, and recovery while being reported as excluded where relevant.

#### Scenario: Previous workspace leaves dirty tabs
- **WHEN** switching to workspace B retains a dirty tab from repository A and the user synchronizes B
- **THEN** A's tab is not saved, committed, reloaded or uploaded by B's operation

#### Scenario: Operation finishes after switching workspace
- **WHEN** a sync started for A finishes while B is visible
- **THEN** its state is recorded for A without replacing B's repository status or reloading unrelated tabs

#### Scenario: Untitled notes are open
- **WHEN** a connected workspace synchronizes while an untitled note is dirty
- **THEN** the note remains unsaved and recoverable, and the app identifies that it did not participate without selecting an implicit filename

### Requirement: Workspace mutations SHALL participate in repository write coordination
File-tree create, rename, move, delete and refresh effects that write into a connected repository SHALL respect the repository write barrier and conflict ownership. Cross-root writes SHALL coordinate all affected roots. The file tree SHALL keep its supported-file visibility policy and bounded rendering; Git decorations SHALL annotate only visible rows while the Sync inventory remains complete.

#### Scenario: Rename is attempted during checkout
- **WHEN** Git holds exclusive worktree mutation admission and the user attempts a file-tree rename
- **THEN** the action waits or reports the busy state and does not execute a competing filesystem rename

#### Scenario: Git updates paths omitted by the tree
- **WHEN** a synchronization changes both visible notes and unsupported or hidden files
- **THEN** the tree refresh preserves its filtering behavior and the complete Git inventory still accounts for all changed paths
