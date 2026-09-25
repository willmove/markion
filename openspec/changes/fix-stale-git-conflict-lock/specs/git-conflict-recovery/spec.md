## Purpose

Keeps interrupted or conflicted Git synchronizations recoverable without leaving the notes in a repository locked once the repository no longer matches the recorded session.

## ADDED Requirements

### Requirement: A stale conflict session SHALL release repository admission
When the app attempts to open, continue, finish or abort a conflict session and finds that the actual repository no longer matches that session (HEAD, merge state or conflicted index differ from the checkpoint), it SHALL stop treating the session as active, release any repository admission held for it, and re-assess the checkpoint against the actual repository. The app SHALL NOT keep a repository-wide write barrier for a session it has determined to be stale.

#### Scenario: Merge was finished outside the app
- **WHEN** a checkpoint is in conflict resolution, the repository no longer has a merge in progress, and the user chooses to resolve the conflict
- **THEN** the app reports that the recorded conflict is no longer in progress
- **AND** typing, undo, save and file-tree actions for notes in that repository work immediately without restarting the app

#### Scenario: A later sync already moved HEAD
- **WHEN** a conflict checkpoint recorded HEAD A and merge tip B, and the repository HEAD is now a descendant of both A and B
- **THEN** the checkpoint is recognized as completed work and retired, and the Backup and Sync status no longer asks the user to choose note changes for it

### Requirement: Completed or superseded checkpoints SHALL be retired automatically
At startup and after every synchronization finishes, the app SHALL retire journal checkpoints for a repository when the work they record is already contained in the current HEAD and they own no conflict draft that differs from the committed content. Retirement SHALL keep a bounded success summary and SHALL NOT modify notes, the index or refs. Checkpoints whose state cannot be matched to HEAD SHALL remain as recovery items.

#### Scenario: Earlier commit is an ancestor of HEAD
- **WHEN** a checkpoint's recorded commit is an ancestor of the current HEAD and it has no drafts
- **THEN** the checkpoint is removed from pending recovery and is not counted in the Backup and Sync status

#### Scenario: Recorded commit is not in history
- **WHEN** a checkpoint's recorded commit is not reachable from HEAD
- **THEN** it remains a recovery item that needs attention and its recovery material is preserved

### Requirement: Stale checkpoints with drafts SHALL require an explicit choice
When a checkpoint that owns a conflict draft is found to be stale or superseded, the app SHALL keep the draft and offer the user an explicit choice to open the draft as an ordinary document or discard the record. Discarding SHALL remove the checkpoint and its draft only after that confirmation; neither option SHALL rewrite notes that changed after the checkpoint.

#### Scenario: User keeps the draft
- **WHEN** the user chooses to keep the draft of a stale conflict record
- **THEN** the draft content opens as a document the user can save elsewhere, and the checkpoint is retired

#### Scenario: User discards the record
- **WHEN** the user confirms discarding a stale conflict record
- **THEN** the checkpoint and its draft are removed and the note on disk is unchanged

### Requirement: Conflict guarding SHALL be limited to conflict-owned paths
While a genuine conflict session is waiting for the user, ordinary typing, undo, save, autosave and file-tree writes SHALL be refused only for paths owned by that conflict. Other notes in the same repository SHALL remain editable and savable. A repository-wide exclusive write barrier SHALL be held only while Git is actively changing the worktree, index or refs.

#### Scenario: Editing an unrelated note during a conflict
- **WHEN** a conflict involving `notes/a.md` is waiting for resolution and the user edits and saves `notes/b.md` in the same repository
- **THEN** the edit and save succeed

#### Scenario: Editing the conflicted note
- **WHEN** a conflict involving `notes/a.md` is waiting for resolution and the user types in `notes/a.md`
- **THEN** the edit is refused and the status explains that the note has an unfinished sync conflict, with an entry to resolve it

### Requirement: Busy status SHALL name its cause
When an edit, save or file action is refused because of repository coordination, the status SHALL say whether Git is currently running an operation or whether an unfinished sync conflict or recovery item needs attention. The conflict/recovery message SHALL offer resolve and discard entry points and SHALL NOT tell the user to wait.

#### Scenario: Git operation is running
- **WHEN** a sync is actively changing the worktree and the user tries to save a note in that repository
- **THEN** the status says a sync is in progress and the save is not performed

#### Scenario: Unfinished conflict blocks a note
- **WHEN** no Git operation is running and a note is guarded by an unfinished conflict
- **THEN** the status says the note has an unfinished sync conflict and offers to resolve or discard it
