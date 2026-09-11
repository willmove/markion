## ADDED Requirements

### Requirement: Workspace chrome SHALL expose one stateful backup-and-sync entry
Connected workspace chrome SHALL provide one compact Backup and Sync entry group containing a consumer-facing state summary and at most one context-sensitive primary action. The summary SHALL open a transient Backup and Sync center; a safe routine Sync Now action SHALL remain executable with one activation from chrome. The Files surface SHALL NOT repeat another Sync/Sync Now pair, and synchronization SHALL NOT occupy a persistent workspace sidebar tab. The entry SHALL not interfere with workspace switching or root-drop gestures. The existing active-document Git branch indicator SHALL retain its document-first semantics; workspace synchronization SHALL identify its own repository and SHALL not retarget when that indicator changes. Compact layout SHALL preserve existing document metrics/feedback and use text/icons as well as color.

#### Scenario: Routine sync requires one ordinary action
- **WHEN** a configured personal-notes workspace has only eligible changes
- **THEN** activating the entry's Sync Now action starts synchronization without opening a mandatory form or repository dashboard

#### Scenario: User opens synchronization details
- **WHEN** the user activates the state summary instead of its current primary action
- **THEN** a transient Backup and Sync center opens without changing the persistent Files/Outline navigation state

#### Scenario: Active document belongs to a different repository
- **WHEN** the active-document branch indicator refers to another repository
- **THEN** workspace sync remains visibly bound to its configured repository and does not borrow the document's target

#### Scenario: Window is narrow
- **WHEN** the existing status row and new sync information cannot fit in full
- **THEN** optional detail collapses without overlapping controls, removing essential error state, or wrapping the persistent status into uncontrolled rows

### Requirement: Backup and Sync details SHALL separate ordinary tasks from advanced Git tools
The transient Backup and Sync center SHALL default to the current consumer-facing state, human-readable sync location, last confirmed time, local items waiting to synchronize, updates from another device, one primary action, sync activity, version history, and settings. It SHALL NOT expose raw branch/upstream/path data, status codes, staged/unstaged layers, commit topology, or Commit Locally, Check Remote, Pull Updates, and Push Commits at equal prominence. Those details and actions SHALL remain available under an explicit Advanced Git disclosure together with complete repository changes, outgoing commit inspection, bounded recent history and file diffs.

Text/Markdown inspection SHALL expose source differences; image/binary inspection SHALL show available previews/metadata. Per-file Version History SHALL also be reachable from the file/tree/tab context. Historical content SHALL support Save a Copy without resetting the branch. Oversized content SHALL degrade to metadata/external inspection. The application menu SHALL expose ordinary Backup and Sync actions first and nest transport-specific commands under Advanced Git Tools. Actions SHALL integrate with existing command/shortcut registration without conflicting default bindings.

#### Scenario: User inspects an outgoing commit
- **WHEN** the user expands Advanced Git details and opens outgoing-history details
- **THEN** the app shows that commit's affected paths including paths outside the notes policy, and supports bounded file inspection

#### Scenario: Routine center opens
- **WHEN** a user opens the center for a healthy connected workspace
- **THEN** the first view explains whether notes are synchronized and the next ordinary action without requiring commit, branch, remote, staging, fetch, pull, or push terminology

#### Scenario: User requests one file's history
- **WHEN** the user invokes Version History from a note's file, tree, or tab context
- **THEN** the app opens bounded history for that file without requiring navigation to Advanced Git details

#### Scenario: Diff exceeds presentation limit
- **WHEN** a selected file exceeds the bounded text-diff limit
- **THEN** the app reports the limit and exposes metadata or external inspection without reading/rendering unbounded content

#### Scenario: User saves historical content as a copy
- **WHEN** the user selects a historical file and chooses a new destination
- **THEN** the app writes through normal destination checks/admission without resetting HEAD or silently replacing current content
