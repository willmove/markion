## ADDED Requirements

### Requirement: Repository status refresh SHALL be change-driven with bounded Git cost
Markion SHALL refresh repository status and Backup and Sync details in response to observed changes rather than on a fixed short interval. Triggers SHALL include:
- worktree or repository control-file changes reported by the workspace watcher;
- the application window becoming active;
- completed saves and Git operations;
- workspace switches;
- opening Backup and Sync.

A fallback refresh MAY run at a low frequency only while the application window is active. Markion SHALL NOT start Git processes on a recurring timer while the window is inactive, except for an explicitly enabled background remote check. Concurrent or rapid triggers SHALL coalesce into at most one refresh in flight plus one queued follow-up.

The number of Git processes started by one refresh SHALL NOT grow with the number of changed paths, open documents, or attachment references in changed notes. Repository capability detection SHALL NOT traverse the full worktree on each refresh, and SHALL be evaluated at least once at the start of every write operation. A refresh SHALL NOT copy or hash the content of clean open documents or unchanged files on each refresh. These bounds SHALL NOT weaken any safety precondition, state projection, or capability limitation defined elsewhere.

#### Scenario: Idle application in the background
- **WHEN** a configured repository is open, nothing changes, and the Markion window is not active
- **THEN** no Git process is started for status refresh until a change notification, window activation, or user action occurs

#### Scenario: External commit is noticed
- **WHEN** the user commits in an external terminal while Markion is open on that repository
- **THEN** Backup and Sync details reflect the new repository state after the watcher notification or the next window activation, without requiring a manual refresh

#### Scenario: Many changed notes with images
- **WHEN** a refresh runs for a repository with hundreds of changed notes that reference many local images
- **THEN** the refresh starts a bounded number of Git processes that does not depend on the number of changed notes or image references

#### Scenario: Rapid saves
- **WHEN** several notes are saved within a short interval
- **THEN** the resulting refresh requests coalesce and at most one refresh runs at a time with at most one queued follow-up

#### Scenario: Unsupported repository shape still blocks writes
- **WHEN** a submodule or nested repository is added to a connected repository after the last refresh and the user starts Sync Now
- **THEN** the operation re-evaluates repository capabilities before writing and refuses one-click write synchronization as before
