## ADDED Requirements

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
