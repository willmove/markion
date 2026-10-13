## MODIFIED Requirements

### Requirement: Repository status SHALL project one truthful consumer-facing sync state
Markion SHALL retain independent source dimensions for unsaved buffers, full-repository tracked/untracked/index changes, incoming/outgoing commit counts when known, in-progress operations, connectivity, conflicts/recovery, authentication, policy/capability blocks, held-back scope exceptions, uncertain delivery, and last remote confirmation time.

It SHALL project them into one prioritized ordinary state: not configured, running, pending locally, incoming updates, synchronized with confirmation time, offline with local work retained, authentication needed, conflict/recovery, uncertain delivery, or needs attention. Conflict/recovery, uncertain delivery, policy/capability blocks, and authentication SHALL take precedence over healthy idle summaries; running progress SHALL take precedence over idle counts.

Held-back scope exceptions SHALL NOT by themselves select the needs-attention state and SHALL NOT be counted as items waiting to synchronize. Whenever any exist, the ordinary surface SHALL show a separate count of files not included in sync with an entry to review them, alongside the primary state, including when the primary state is synchronized.

The ordinary state SHALL use note and device language while complete technical dimensions remain available in Advanced Git details. The app SHALL NOT call a file save or local-only commit a remote backup, SHALL NOT call an unknown/stale remote observation synchronized, and SHALL qualify a confirmed delivered snapshot when newer local edits remain. Setup/help SHALL explain that synchronization propagates supported edits and deletions and that Version History, not an implied immutable archive, is the recovery path. The complete Git inventory SHALL include changes hidden or unsupported by the document tree. An unavailable remote observation SHALL be unknown or stale, not zero. Status reads SHALL be asynchronous, bounded, and independent of document parsing.

#### Scenario: Hidden file changes outside the document tree
- **WHEN** Git reports a changed path absent from the document tree that the notes policy does not include automatically
- **THEN** the ordinary surface reports it as not included in sync with a review entry
- **AND** Advanced Git inventory includes that path with its actual status and document-tree omission

#### Scenario: Only held-back files remain
- **WHEN** every approved change has been confirmed at the sync location and one held-back file remains on this computer
- **THEN** the primary state reports the approved notes as synchronized with the confirmation time
- **AND** the surface also states that one file is not included in sync

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

## ADDED Requirements

### Requirement: Held-back scope exceptions SHALL NOT block synchronization of approved content
Sync Now, Commit Locally, and Pull Updates SHALL operate on approved changes while held-back scope exceptions remain uncommitted in the worktree.

Integration SHALL tolerate held-back new files that were present when the operation started, provided no incoming path collides with them. Integration SHALL pause, preserving the local commit and the held-back change, when a held-back tracked change exists. A held-back new file that collides with an incoming path SHALL block integration and be identified, as for any untracked collision. A new file or edit that appears while the operation runs SHALL still pause required integration.

Operation results SHALL report how many files were held back.

#### Scenario: Remote updates arrive while a hidden file is held back
- **WHEN** a held-back new file exists, the remote has new commits that do not touch its path, and the user clicks Sync Now
- **THEN** the approved changes are committed, remote updates are integrated, the result is uploaded, and the held-back file remains uncommitted on disk

#### Scenario: Held-back tracked change and incoming updates
- **WHEN** a tracked file outside the approved tracked roots has been modified and the remote has new commits
- **THEN** approved changes are committed locally, integration pauses before changing the worktree, and the app identifies the held-back tracked file that must be reviewed before updates can be applied

#### Scenario: Incoming file collides with a held-back file
- **WHEN** an incoming commit adds a path that exists locally as a held-back new file
- **THEN** integration is refused, the local file is preserved, and the colliding path is identified
