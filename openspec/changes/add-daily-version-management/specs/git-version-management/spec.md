## ADDED Requirements

### Requirement: Users SHALL create explicit local versions from selected paths
Markion SHALL provide a local-version composer that displays current repository changes, lets the user select whole repository-relative paths, requires a nonblank authored commit message, and creates a local commit containing only the selected paths. It SHALL operate without fetching, integrating, pushing, or otherwise contacting a remote. The automatic policy-driven Sync Now and Commit Locally actions SHALL retain their existing behavior.

#### Scenario: Commit a selected group of changes
- **WHEN** the repository has several committable changes and the user selects a subset, enters a nonblank message, and chooses Create Local Version
- **THEN** Markion saves eligible selected named buffers, stages only the selected paths with literal path semantics, and creates one local commit with that message
- **AND** unselected worktree changes remain uncommitted
- **AND** no network operation runs

#### Scenario: Empty message or selection
- **WHEN** the commit message is blank after trimming or no committable path is selected
- **THEN** Create Local Version remains unavailable or reports the missing input
- **AND** the index, worktree, and HEAD remain unchanged

#### Scenario: Selected buffer cannot be safely saved
- **WHEN** a selected open buffer is dirty and cannot pass the existing disk-identity or durable-save checks
- **THEN** Markion stops before staging, identifies the blocking file, and preserves the buffer and commit draft

#### Scenario: Unselected dirty buffer exists
- **WHEN** an unselected repository file has unsaved editor changes
- **THEN** creating a local version leaves that buffer unsaved and excludes its path from the commit

### Requirement: Explicit local versions SHALL preserve index ownership and reviewed content
The local-version composer SHALL bind its selection to a captured HEAD and status identity. Before staging, Markion SHALL reject conflicts, active repository mutations, pre-existing staged content, unsafe paths, a changed HEAD, or selected content that no longer matches the reviewed state. A failed app-owned stage or commit SHALL use the existing ownership journal and SHALL NOT reset unrelated index or worktree content.

#### Scenario: External staged content is present
- **WHEN** Git reports an index change that was not staged by the active Markion operation
- **THEN** Markion disables or refuses Create Local Version and explains that the existing staging arrangement must be resolved externally
- **AND** it does not modify the index

#### Scenario: Selected path changes before commit
- **WHEN** a selected path or HEAD changes after the draft was reviewed and before staging begins
- **THEN** Markion refreshes the draft and requests renewed review instead of committing unseen content

#### Scenario: Commit hook rejects the commit
- **WHEN** exact-path staging succeeds but a configured commit hook or signing operation rejects the commit
- **THEN** Markion reports the failure, retains the draft and worktree content, and exposes only ownership-checked recovery actions

### Requirement: Repository and file history SHALL be browsable in bounded pages
Markion SHALL show bounded pages of local repository history and history for the active repository file. Each entry SHALL retain its full object identity and display a subject, author, authored time, and parent information. Selecting an entry SHALL show its changed paths and allow bounded detail inspection. History reads SHALL run outside rendering and text input, and stale results SHALL not replace a newer repository or active-file request.

#### Scenario: Browse repository history
- **WHEN** the user opens Versions for a repository with commits
- **THEN** Markion displays the newest bounded page with commit metadata and supports requesting the next page

#### Scenario: Browse active-file history
- **WHEN** the active named file belongs to the repository and the user selects File History
- **THEN** Markion displays bounded commits affecting that file and follows ordinary single-file renames where Git can determine them

#### Scenario: Active file changes while history loads
- **WHEN** the user switches files or repositories before a history request completes
- **THEN** the stale result is discarded and does not replace the history for the new context

### Requirement: Historical versions SHALL compare against the working version safely
For a selected commit and repository path, Markion SHALL provide a bounded comparison between that historical version and the current working version. The comparison SHALL disable external diff drivers and text conversion, pass paths literally, identify binary or oversized results, and SHALL NOT mutate the worktree, index, documents, or HEAD.

#### Scenario: Compare text with a historical version
- **WHEN** the user selects Compare with Current for a text path changed since the selected commit
- **THEN** Markion displays a bounded source diff from the selected commit to the working version
- **AND** the repository and editor state remain unchanged

#### Scenario: Comparison is binary or too large
- **WHEN** Git identifies binary content or the configured output bound is reached
- **THEN** Markion shows a localized binary or truncated fallback instead of rendering unbounded content

### Requirement: Historical text SHALL restore as an unsaved editor change
Markion SHALL restore an eligible historical UTF-8 text blob by opening or focusing its worktree path and applying the historical source through the normal document edit transaction. Restore SHALL create an unsaved, undoable editor change and SHALL NOT write the file, alter the index, move HEAD, or create a commit. Markion SHALL revalidate repository path containment, tab identity, and disk identity before applying the result.

#### Scenario: Restore a clean text file
- **WHEN** the user selects Restore in Editor for a bounded UTF-8 historical version whose current path and disk identity still match
- **THEN** the historical source replaces the editor buffer as one undoable edit and the tab becomes dirty
- **AND** the on-disk file and Git metadata remain unchanged until a later explicit save or commit

#### Scenario: Restore over unsaved edits
- **WHEN** the target buffer already contains unsaved edits
- **THEN** Markion requires explicit replacement confirmation before applying the historical source
- **AND** cancellation preserves the existing buffer exactly

#### Scenario: Restore target changed during read
- **WHEN** the target path, tab identity, repository identity, or disk identity changes while historical content is loading
- **THEN** Markion refuses the stale restore and leaves the current editor state unchanged

#### Scenario: Historical content cannot be edited safely
- **WHEN** the selected version is binary, oversized, invalid UTF-8, a symlink, deleted at that commit, or outside the worktree
- **THEN** Restore in Editor is unavailable and Markion retains the guarded Save a Copy option when bytes can be exported safely

### Requirement: Version-management UI SHALL keep results and feedback coherent
The Git sidebar SHALL expose local-version drafting, repository/file history modes, selection detail, comparison, restore, and Save a Copy with localized labels and statuses. Successful commits SHALL refresh Git inventory, file-tree decorations, history, and outgoing counts. Failed actions SHALL retain user-authored draft input when retry is safe, and ordinary document editing SHALL remain responsive during Git reads and writes.

#### Scenario: Local version succeeds
- **WHEN** Create Local Version completes successfully
- **THEN** the sidebar reports the new commit, clears the committed selection and message, and refreshes status, decorations, history, and outgoing state

#### Scenario: Local version fails before commit
- **WHEN** validation, save, staging, signing, or hook execution fails
- **THEN** Markion reports an actionable localized error and retains the message and safe selected paths for correction

#### Scenario: History query is slow
- **WHEN** history, comparison, or blob loading takes longer than one render frame
- **THEN** the sidebar shows bounded progress while document input and rendering remain responsive

### Requirement: Repository operations SHALL have a dedicated application menu
Markion SHALL provide a localized Repository top-level menu in both its native and in-window menu bars. All existing Git and synchronization commands SHALL appear in that menu instead of the File menu, while retaining their existing actions and configurable shortcut identities.

#### Scenario: Browse repository commands
- **WHEN** the user opens the Repository menu
- **THEN** it offers synchronization details, Sync Now, local commit, remote check, pull, push, conflict resolution, and synchronization setup
- **AND** the File menu contains no Git or synchronization command

#### Scenario: Switch menus with the pointer
- **WHEN** an in-window application menu is open and the user moves the pointer to Repository
- **THEN** the Repository dropdown replaces the prior dropdown using the same hover-switch behavior as every other top-level menu
