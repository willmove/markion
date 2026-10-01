## ADDED Requirements

### Requirement: Watched file-tree refreshes SHALL ignore events that cannot change the tree
While the workspace root is being watched, the editor SHALL refresh the file tree only for change events that can alter the visible tree: creating, deleting, or renaming an entry that the tree could show, events of unknown kind, and watcher signals that events were lost. It SHALL NOT refresh for content-only or metadata-only modifications of existing entries. It SHALL NOT refresh for events whose every affected path lies inside a directory the tree always excludes. While hidden entries are not shown, it SHALL NOT refresh for events whose every affected path is a dot-named entry or lies inside one. Manual Refresh and the periodic fallback refresh used when watching cannot start SHALL be unaffected.

#### Scenario: Saving a document does not rescan the tree
- **WHEN** a Markdown file under the watched workspace is saved or autosaved (its content changes but no entry is created, deleted, or renamed)
- **THEN** the file tree is not rescanned

#### Scenario: Activity inside excluded or hidden folders is ignored
- **WHEN** files change only inside `.git` or another always-excluded directory, or only inside a dot-named folder such as `.obsidian` while hidden entries are not shown
- **THEN** the file tree is not rescanned

#### Scenario: Hidden-folder activity refreshes when hidden entries are shown
- **WHEN** hidden entries are shown and a file is created inside a dot-named folder that is not always excluded
- **THEN** the file tree refreshes and shows it

#### Scenario: Visible creates, deletes, and renames still refresh
- **WHEN** a supported file or a folder outside excluded and hidden folders is created, deleted, or renamed
- **THEN** the file tree refreshes as before

#### Scenario: Unknown or lossy events still refresh
- **WHEN** the watcher reports an event of unknown kind or signals that events were dropped
- **THEN** the file tree refreshes
