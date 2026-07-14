## ADDED Requirements

### Requirement: Open files via drag-and-drop from the OS
The editor SHALL accept files dragged from the operating system file manager and dropped onto the editor pane or the preview pane. On drop, the editor SHALL open each dropped path whose extension is `md`, `markdown`, or `mdown` (case-insensitive) as a new tab, with the focus following the last opened file. Non-Markdown files and directories SHALL be ignored (they SHALL NOT be opened, and SHALL NOT produce an error). The sidebar / file-tree area SHALL NOT be a drop target. This open path SHALL reuse the same Markdown-only filter and new-tab behaviour as the file-tree open flow; it introduces no new user-visible strings.

#### Scenario: Dropping a single Markdown file opens it
- **WHEN** the user drags one Markdown file (`.md`, `.markdown`, or `.mdown`) from the OS file manager and drops it onto the editor pane or the preview pane
- **THEN** the editor opens that file in a new tab and focuses it
- **AND** the status bar reports the opened path through the existing `StatusOpened` message

#### Scenario: Dropping multiple Markdown files opens each in its own tab
- **WHEN** the user drops several Markdown files at once onto the editor pane or the preview pane
- **THEN** the editor opens each Markdown file in its own new tab
- **AND** the focus moves to the last opened file
- **AND** the status bar reports the last opened path through the existing `StatusOpened` message

#### Scenario: Non-Markdown files and directories are ignored
- **WHEN** the user drops only non-Markdown files (e.g. `.png`, `.txt`) and/or directories onto the editor pane or the preview pane
- **THEN** no file is opened and no error is raised
- **AND** the status bar is left unchanged (no "opened" or "skipped" message is shown)

#### Scenario: Mixed drop opens only the Markdown files
- **WHEN** the user drops a mix of Markdown and non-Markdown files at once
- **THEN** only the Markdown files are opened, each in its own tab, with focus on the last Markdown file
- **AND** the non-Markdown files are silently ignored

#### Scenario: The sidebar and file tree are not drop targets
- **WHEN** the user drops a Markdown file onto the sidebar (the file-tree panel) or the chrome around the panes
- **THEN** no file is opened and the file tree is unaffected
