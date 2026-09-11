## ADDED Requirements

### Requirement: Repository status SHALL project one truthful consumer-facing sync state
Markion SHALL retain independent source dimensions for unsaved buffers, full-repository tracked/untracked/index changes, incoming/outgoing commit counts when known, in-progress operations, connectivity, conflicts/recovery, authentication, policy/capability blocks, uncertain delivery, and last remote confirmation time. It SHALL project them into one prioritized ordinary state: not configured, running, pending locally, incoming updates, synchronized with confirmation time, offline with local work retained, authentication needed, conflict/recovery, uncertain delivery, or needs attention. Conflict/recovery, uncertain delivery, policy/capability blocks, and authentication SHALL take precedence over healthy idle summaries; running progress SHALL take precedence over idle counts.

The ordinary state SHALL use note and device language while complete technical dimensions remain available in Advanced Git details. The app SHALL NOT call a file save or local-only commit a remote backup, SHALL NOT call an unknown/stale remote observation synchronized, and SHALL qualify a confirmed delivered snapshot when newer local edits remain. Setup/help SHALL explain that synchronization propagates supported edits and deletions and that Version History, not an implied immutable archive, is the recovery path. The complete Git inventory SHALL include changes hidden or unsupported by the document tree. An unavailable remote observation SHALL be unknown or stale, not zero. Status reads SHALL be asynchronous, bounded, and independent of document parsing.

#### Scenario: Hidden file changes outside the document tree
- **WHEN** Git reports a changed path absent from the document tree
- **THEN** the ordinary state reports an item needing attention and Advanced Git inventory includes that path with its actual status and document-tree omission

#### Scenario: Network is unavailable
- **WHEN** the app cannot refresh remote state
- **THEN** it preserves local status and the last confirmed time while ordinary wording states that work remains saved on this computer and remote information is unavailable
- **AND** it does not report synchronized merely because local files or history are intact

#### Scenario: Confirmed delivery has newer local edits
- **WHEN** a previously captured snapshot is confirmed at the sync location and the user has since changed a note
- **THEN** the ordinary state distinguishes the confirmed snapshot from the new items waiting to synchronize

#### Scenario: Delivery outcome is uncertain
- **WHEN** a push response is lost and remote history cannot yet confirm the intended commit
- **THEN** uncertain delivery overrides a healthy or pending summary and the primary action checks status before another upload attempt

#### Scenario: User reviews backup semantics
- **WHEN** setup or Backup and Sync help explains what synchronization protects
- **THEN** it states that edits and deletions propagate to the sync location and points to Version History for recovery without promising immutable or unlimited retention

### Requirement: Sync Now SHALL provide the complete one-click manual workflow
After initial connection/policy approval, Sync Now SHALL save eligible named buffers, create a nonempty local commit with an automatic message when required, fetch the bound remote, safely integrate changes, push the captured result, and confirm the outcome. Routine eligible changes SHALL NOT require repeated file selection or message entry. Unnamed, foreign and excluded buffers SHALL remain untouched and visibly excluded. Eligible save failure SHALL stop before staging/commit while preserving successful saves and all recoverable work.

#### Scenario: Ordinary notes synchronize with one click
- **WHEN** a connected workspace contains only approved note/attachment changes and the user clicks Sync Now
- **THEN** the app performs the save-to-confirmation sequence without another message or selection dialog

#### Scenario: Silent save is disabled
- **WHEN** eligible named notes are dirty and silent save-to-file is disabled
- **THEN** explicit Sync Now saves those notes through the checked manual-save boundary before committing
- **AND** it does not change the user's autosave preference

#### Scenario: Nothing changed
- **WHEN** local files, local history and the fetched target already agree
- **THEN** Sync Now updates its checked status without creating an empty commit

#### Scenario: One eligible document cannot be saved
- **WHEN** one participating buffer fails a disk identity or durable-save check
- **THEN** the operation stops before staging and committing, identifies the failed document, and preserves all document content

### Requirement: Local commit plans SHALL preserve staging ownership and reviewed content
One-click commits SHALL refuse pre-existing external staged changes, stage only policy-approved paths using exact path semantics, and verify planned index/content identities. If an optional user-reviewed plan changes, the app SHALL request a renewed review. A failed or interrupted app commit SHALL retain a recoverable staging ownership record; retry or unstage SHALL require that the recorded index/path identities still match. No automatic command SHALL clear unrelated staging or discard worktree content.

#### Scenario: Existing partially staged note
- **WHEN** the index already contains a user-staged version different from the worktree
- **THEN** Sync Now pauses without overwriting or committing the existing staging arrangement

#### Scenario: File changes after optional review
- **WHEN** a reviewed file changes before its commit is captured
- **THEN** the reviewed plan becomes stale and the app does not silently include the new bytes

#### Scenario: Hook rejects an app-owned commit
- **WHEN** staging succeeded but a commit hook fails
- **THEN** the app reports the failed stage and offers ownership-checked retry or unstage while retaining the worktree content

### Requirement: Remote integration SHALL preserve both histories and local edits
The app SHALL fast-forward when only the remote is ahead and perform an ordinary merge when histories diverge with a common ancestor. It SHALL stop on unrelated history, a rewritten previously observed target, a deleted target, or unsupported paths/state. Before worktree mutation it SHALL revalidate clean tracked/index/buffer state across the repository and refuse overwriting untracked/ignored collisions. Edits made during fetch SHALL remain local and pause required integration. It SHALL NOT automatically rebase, stash, force-reset, clean, or select a conflict winner.

#### Scenario: Remote-only update
- **WHEN** the remote is ahead and the entire repository satisfies the integration preconditions
- **THEN** the local branch fast-forwards and affected open content is reconciled

#### Scenario: Both devices edited different sections
- **WHEN** local and remote history diverge with nonconflicting changes
- **THEN** an ordinary merge preserves both histories and content before push

#### Scenario: User types during fetch
- **WHEN** new unsaved or uncommitted tracked changes appear while fetching and integration would modify the worktree
- **THEN** integration pauses with those changes intact rather than creating another automatic snapshot or stashing them

#### Scenario: Incoming path would overwrite an untracked file
- **WHEN** integration would collide with an untracked or ignored local path
- **THEN** the app blocks that integration and identifies the path without deleting the local file

#### Scenario: Remote rewrites previously observed history
- **WHEN** a fetched existing target is no longer a descendant of the last observed target
- **THEN** the app reports rewritten history and requires review rather than automatically merging the rewrite

### Requirement: Push SHALL use a fixed commit and explicit non-force target
The app SHALL push a captured commit OID to exactly the bound full remote branch ref with ordinary non-force semantics and no implicit tag, mirror, multi-destination, or all-branch transfer. It SHALL verify target/branch identity before push and retain new local edits outside the captured snapshot. A non-fast-forward rejection SHALL allow at most one automated refresh/integration/retry under the same safety preconditions. Missing confirmation SHALL be reconciled against remote history before declaring delivery or repeating the operation.

#### Scenario: Another device pushes first
- **WHEN** a non-fast-forward rejection occurs because another device updated the branch
- **THEN** the app refreshes and attempts safe integration/retry at most once, otherwise reports attention with local commits retained

#### Scenario: Push succeeds but response is lost
- **WHEN** the process loses confirmation after sending a captured commit
- **THEN** the app checks whether the target history contains that commit and avoids creating another content commit
- **AND** failure to verify is shown as an uncertain result

#### Scenario: New edits remain after successful delivery
- **WHEN** the captured commit is confirmed remotely but the user has since edited a note
- **THEN** the app reports the captured content delivered and separately reports pending local changes

### Requirement: Advanced Git actions SHALL remain available with predictable effects
Commit Locally, Check Remote, Pull Updates, and Push Commits SHALL be available only after an explicit Advanced Git disclosure or command invocation rather than as equal ordinary Backup and Sync actions. Commit Locally SHALL save and commit eligible content without networking. Check Remote SHALL fetch without saving, committing, integrating, or pushing. Pull Updates SHALL fetch and safely integrate without implicitly saving dirty buffers or creating a content snapshot commit, although divergence can create a merge commit. Push Commits SHALL send existing captured history without saving or committing files and without integrating remote changes.

#### Scenario: Ordinary center is opened
- **WHEN** a user opens Backup and Sync details without expanding Advanced Git tools
- **THEN** the app shows one state-appropriate primary action and does not require choosing among commit, fetch, pull, or push

#### Scenario: User checks for remote changes while editing
- **WHEN** Check Remote runs while a buffer is dirty
- **THEN** remote metadata can update but the buffer, worktree files, index and local branch are unchanged by the check

#### Scenario: User pulls with uncommitted changes
- **WHEN** Pull Updates would integrate while tracked changes or unsaved repository buffers exist
- **THEN** the app reports the blocking state without implicitly committing or discarding it

#### Scenario: Offline local commit
- **WHEN** the user selects Commit Locally without network access
- **THEN** eligible content can be saved and committed locally and is displayed as awaiting upload when appropriate

### Requirement: Execution SHALL be bounded cancellable and serialized
The app SHALL serialize operations per repository and coordinate shared Git directories, use structured process arguments and machine-readable/NUL-safe status, drain bounded outputs, and run filesystem/process/network work off the UI thread. It SHALL expose progress, stage-specific timeout/cancellation and actionable errors. Cancellation of a mutation SHALL reconcile actual Git/file state before another app write is admitted. Unknown Git lock files SHALL NOT be automatically deleted.

#### Scenario: Repeated sync clicks
- **WHEN** the same repository already has a running sync and the user clicks again
- **THEN** the app focuses/reports that operation instead of launching another write sequence

#### Scenario: Git lock belongs to an unknown operation
- **WHEN** Git reports an existing repository lock
- **THEN** the app reports the contention and retains the lock rather than removing it

#### Scenario: A local mutation is cancelled
- **WHEN** the user requests cancellation while a local Git mutation is running
- **THEN** the app reaches a safe boundary or stops owned processes and reconciles state before resuming repository writes

### Requirement: Optional background checks SHALL never write synchronization changes
Background remote checking SHALL default off and be separately enabled. Ordinary labels and descriptions SHALL state that it checks for updates from other devices and does not automatically upload or apply changes. It SHALL operate only for the currently active connected workspace, use bounded scheduling/backoff with one request at a time, and perform fetch only. It SHALL NOT save documents, create commits, integrate, push, or open repeated interactive authentication prompts. Inactive repositories SHALL NOT receive new scheduled checks. Unchanged results SHALL stay quiet.

#### Scenario: Background check discovers remote changes
- **WHEN** an enabled background fetch finds new remote commits
- **THEN** the app displays an incoming-update state without changing open documents or the worktree

#### Scenario: Background authentication needs interaction
- **WHEN** credentials cannot be obtained noninteractively
- **THEN** checking pauses with an actionable authentication state and no recurring login dialog

#### Scenario: Workspace changes during a check
- **WHEN** the user changes workspace while a background fetch is in flight
- **THEN** the result stays associated with its captured repository and does not start subsequent scheduled checks for the inactive workspace
