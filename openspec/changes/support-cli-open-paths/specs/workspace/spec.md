## ADDED Requirements

### Requirement: Command-line startup paths
The editor SHALL accept a single optional positional path when launched from the command line. If the path resolves to a supported Markdown file (`.md`, `.markdown`, or `.mdown`, case-insensitive), the editor SHALL open that file as the initial document and scan the applicable file-tree workspace root. If the path resolves to a directory, the editor SHALL make that directory the file-tree workspace root, reveal the Files sidebar, and scan the directory using the existing Markdown-only file-tree behavior without replacing or modifying the welcome document. If no positional path is provided, startup behavior SHALL remain unchanged: the editor shows the in-memory welcome document and SHALL NOT scan the process working directory. Missing paths, unsupported file paths, and paths that are neither files nor directories SHALL NOT create files or folders.

#### Scenario: Launch with a Markdown file
- **WHEN** the editor is launched with one positional path that resolves to an existing `.md`, `.markdown`, or `.mdown` file
- **THEN** that file opens as the initial saved document in the first tab
- **AND** the file tree scans the applicable workspace root using the same behavior as an in-app file open
- **AND** the status bar reports the opened path through the existing opened-file feedback

#### Scenario: Launch with a folder
- **WHEN** the editor is launched with one positional path that resolves to an existing directory
- **THEN** that directory becomes the file-tree workspace root without replacing or modifying the welcome document
- **AND** the left sidebar becomes visible on the Files tab
- **AND** the selected directory is scanned asynchronously using the existing Markdown-only tree behavior

#### Scenario: Launch without a path
- **WHEN** the editor is launched without a positional path
- **THEN** the editor shows the in-memory welcome document
- **AND** the file tree does not scan the process working directory

#### Scenario: Launch with an invalid path
- **WHEN** the editor is launched with a positional path that is missing, unsupported, or neither a file nor a directory
- **THEN** the editor keeps the in-memory welcome document
- **AND** no file or folder is created
- **AND** the file tree does not scan the process working directory
- **AND** the status bar reports the failure through existing open-failure feedback

#### Scenario: Relative startup path resolution
- **WHEN** the editor is launched with a relative positional path
- **THEN** the path is resolved relative to the process working directory before file/folder classification
