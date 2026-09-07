## ADDED Requirements

### Requirement: Workspace chrome SHALL expose one-click sync with explicit context
Connected workspace chrome SHALL provide a distinct Sync Now control and a compact persistence/remote-confirmation summary, with a Sync sidebar for setup, activity and actionable errors. The control SHALL not interfere with workspace switching or root-drop gestures. The existing active-document Git branch indicator SHALL retain its document-first semantics; workspace synchronization SHALL identify its own repository and SHALL not retarget when that indicator changes. Compact layout SHALL preserve existing document metrics/feedback and use text/icons as well as color.

#### Scenario: Routine sync does not require sidebar interaction
- **WHEN** a configured personal-notes workspace has only eligible changes
- **THEN** activating Sync Now from workspace chrome starts synchronization without opening a mandatory commit form

#### Scenario: Active document belongs to a different repository
- **WHEN** the active-document branch indicator refers to another repository
- **THEN** workspace sync remains visibly bound to its configured repository and does not borrow the document's target

#### Scenario: Window is narrow
- **WHEN** the existing status row and new sync information cannot fit in full
- **THEN** optional detail collapses without overlapping controls, removing essential error state, or wrapping the persistent status into uncontrolled rows

### Requirement: Sync details SHALL expose optional inspection and secondary commands
The Sync sidebar SHALL provide complete Git changes, outgoing commit inspection, bounded recent history and file diffs, plus Commit Locally, Check Remote, Pull Updates, and Push Commits. Text/Markdown inspection SHALL expose source differences; image/binary inspection SHALL show available previews/metadata. Historical content SHALL support Save a Copy without resetting the branch. Oversized content SHALL degrade to metadata/external inspection. Actions SHALL integrate with existing command/shortcut registration without conflicting default bindings.

#### Scenario: User inspects an outgoing commit
- **WHEN** the user opens outgoing-history details
- **THEN** the app shows that commit's affected paths including paths outside the notes policy, and supports bounded file inspection

#### Scenario: Diff exceeds presentation limit
- **WHEN** a selected file exceeds the bounded text-diff limit
- **THEN** the app reports the limit and exposes metadata or external inspection without reading/rendering unbounded content

#### Scenario: User saves historical content as a copy
- **WHEN** the user selects a historical file and chooses a new destination
- **THEN** the app writes through normal destination checks/admission without resetting HEAD or silently replacing current content
