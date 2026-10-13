## MODIFIED Requirements

### Requirement: Personal notes policy SHALL authorize routine automatic commits once
Setup SHALL persist explicit repository-relative roots for tracked changes and explicit roots/classes for new notes and attachments. Routine Sync Now SHALL include eligible modifications, additions, and deletions without requiring repeated file selection or a commit message.

**Held-back changes.** New hidden files, dotpaths, symlinks, nested repositories, arbitrary binaries, and changes outside approved policy SHALL require attention rather than automatic inclusion. Such changes SHALL be held back from the automatic commit and listed as not included in sync. Holding a change back SHALL NOT prevent the remaining approved changes from being saved, committed, and synchronized. A tracked deletion whose content exactly matches a held-back new file SHALL be held back with it, so a move is never committed as only the deletion or only the addition.

**Ignore rules.** Git ignore rules SHALL apply to untracked candidates, not remove already-tracked paths from status. Well-known operating-system artifact files SHALL be treated as ignored untracked candidates by default and SHALL NOT be listed as exceptions. Already-tracked copies of such files SHALL follow the tracked-path policy.

**Per-file decisions.** Each held-back new file that can be committed safely SHALL offer three explicit decisions: include this file once, always synchronize its file type, or exclude it on this device only.
- An include-once decision SHALL be bound to the file's content at decision time. It SHALL lapse if the content changes before it is committed.
- A device-only exclusion SHALL NOT create a repository change or affect other devices.
- Settings SHALL list approved extra file types and device-only exclusions created by Markion, and SHALL allow removing them.
- Symlinks and nested repositories SHALL NOT be offered for inclusion.

Scope approval SHALL NOT be described as limiting transferred branch history.

#### Scenario: Normal new note and image fall within policy
- **WHEN** the user adds a supported note and attachment in approved directories and clicks Sync Now
- **THEN** both are included with eligible tracked changes and an automatically generated message without another selection dialog

#### Scenario: Unexpected file appears
- **WHEN** an unignored new private configuration file or a tracked change outside the approved roots appears
- **THEN** the app shows the exception and allows explicit inclusion, policy adjustment, or exclusion
- **AND** the ordinary automatic commit does not silently include it

#### Scenario: Unsupported attachment does not block other notes
- **WHEN** the user has edited two notes and added a new PDF that is outside the approved file classes, and clicks Sync Now
- **THEN** both notes are committed and synchronized
- **AND** the PDF remains on disk uncommitted and is listed as not included in sync with its reason

#### Scenario: Operating-system artifact appears
- **WHEN** the operating system creates `.DS_Store`, `Thumbs.db`, or `desktop.ini` inside the notes folder
- **THEN** the file is neither committed nor listed as an exception, and Sync Now proceeds normally

#### Scenario: User includes a held-back file once
- **WHEN** the user chooses to include a held-back file once and then syncs without changing it
- **THEN** that exact content is committed with the next automatic commit

#### Scenario: Included file changes before sync
- **WHEN** the user chooses to include a held-back file once and the file changes before the next sync
- **THEN** the changed file is held back again and is not committed

#### Scenario: User always allows a file type
- **WHEN** the user chooses to always synchronize the file type of a held-back `report.pdf`
- **THEN** that file and later new files with the same extension in approved note folders are included automatically, and settings list the extension with a way to remove it

#### Scenario: User excludes a file on this device
- **WHEN** the user chooses not to sync a held-back file on this device
- **THEN** the file is no longer listed or committed on this device
- **AND** no repository file changes and other devices are unaffected
- **AND** settings list the exclusion with a way to remove it

#### Scenario: Ignore rule matches an already-tracked note
- **WHEN** an already-tracked note changes and a new ignore rule matches its path
- **THEN** its tracked change remains visible and follows the tracked-path policy

#### Scenario: Rename crosses approved roots
- **WHEN** a renamed tracked file moves across a scope boundary
- **THEN** the app requests review of both paths and does not commit only the deletion or only the addition as an unnoticed scope effect

#### Scenario: Tracked attachment is moved into an excluded location
- **WHEN** a tracked attachment is moved to a path whose new location is held back and its content is unchanged
- **THEN** both the deletion of the old path and the new file are held back together, and other approved changes still synchronize
