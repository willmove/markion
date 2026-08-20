## ADDED Requirements

### Requirement: File paths SHALL be presented and persisted in platform-normal form

File paths shown to the user, written to the clipboard, or persisted in session state SHALL be in platform-normal form — on Windows, drive-rooted paths SHALL NOT carry the extended-length verbatim prefix (`\\?\`), and network paths SHALL NOT carry the `\\?\UNC\` form (they use `\\server\share\...`). Paths attached to opened content (document tabs, image tabs), the workspace root, file-tree entries, and recent-files entries SHALL all be in normal form, so that any surface reading them — copy actions, status feedback, reveal-in-file-manager, session persistence — presents the normal form. A path that genuinely requires the extended-length syntax (e.g. longer than the classic Windows path limit) MAY retain the verbatim form, since correct file access takes precedence over cosmetic presentation.

#### Scenario: Copy File Path omits the verbatim prefix

- **WHEN** the user copies the path of a file that was opened from the file tree, restored from a saved session, or opened from the recent-files list on Windows
- **THEN** the clipboard receives the drive-rooted path without the `\\?\` prefix (e.g. `C:\Workspace\Vaults\articles\AGENTS.md`)
- **AND** the status feedback following the copy shows the same normal-form path

#### Scenario: Reveal in File Manager uses a normal-form path

- **WHEN** the user reveals an opened file in the system file manager on Windows
- **THEN** the path handed to the file manager and shown in the feedback message is in normal form, without the `\\?\` prefix

#### Scenario: Paths attached to opened content stay in normal form

- **WHEN** a file is opened from the file tree, restored from a saved session, or opened from the recent-files list
- **THEN** the tab's stored path and the workspace root are in normal form
- **AND** tab dedupe and workspace-containment checks continue to treat the same file as the same file (case and symlink differences on Windows resolve to one identity)

#### Scenario: Legacy verbatim session entries are healed on load

- **WHEN** a session persisted by an earlier version contains `\\?\`-prefixed open-file, active-file, workspace-root, or recent-file entries
- **THEN** loading that session converts the entries to normal form wherever the shortened path is equivalent, before any tab, workspace, or recent list is built from them
- **AND** the next session save persists the healed normal-form paths

#### Scenario: Paths requiring extended-length syntax keep working

- **WHEN** a path exceeds the classic Windows path limit or otherwise requires verbatim syntax, so that removing the prefix would not yield an equivalent path
- **THEN** the verbatim form is retained for file access rather than stripped unconditionally
- **AND** open, save, and reveal operations on such a path continue to succeed
