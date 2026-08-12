## ADDED Requirements

### Requirement: File tree hidden-entry visibility SHALL be preference-controlled

The file tree SHALL classify a file or folder as **hidden** when its file name begins with `.` (on every platform) or, on Windows, when the entry carries the hidden file attribute. Hidden entries SHALL be omitted from the file tree when the Show-hidden-files preference is **off** (the default), and SHALL be included when the preference is **on**, subject in both states to the existing Markdown-only filter and the always-excluded build/dependency noise list. Hidden-entry visibility SHALL apply identically to files and folders — a hidden Markdown file and a hidden folder are treated the same way. The noise list (e.g. `target`, `node_modules`) SHALL remain excluded regardless of the preference.

#### Scenario: Hidden entries are omitted by default
- **WHEN** a workspace root contains a dotfile Markdown file (e.g. `.secret.md`) and a dotfile folder (e.g. `.notes/`) and the Show-hidden-files preference is off
- **THEN** neither the dotfile file nor any entry under the dotfile folder appears in the file tree
- **AND** non-hidden Markdown files and their ancestor folders continue to appear as before

#### Scenario: Toggling the preference on reveals hidden entries
- **WHEN** the user turns the Show-hidden-files preference on
- **THEN** hidden Markdown files and the folders containing them appear in the tree on the next scan
- **AND** the Markdown-only filter still excludes non-Markdown hidden files (e.g. `.env`) from the tree

#### Scenario: Toggling the preference off re-hides hidden entries
- **WHEN** the user turns the Show-hidden-files preference off after having revealed hidden entries
- **THEN** hidden files and folders are removed from the tree on the next scan
- **AND** the tree returns to the same visible set as the default-off state

#### Scenario: The build/dependency noise list stays excluded when hidden entries are revealed
- **WHEN** the Show-hidden-files preference is on and the workspace contains entries on the always-excluded noise list (e.g. `target/`, `node_modules/`)
- **THEN** those noise-list entries still do not appear in the file tree
- **AND** only OS-hidden Markdown entries (dotfile names, or Windows hidden-attribute entries) are newly revealed

#### Scenario: A hidden folder and its contents are omitted together, revealed together
- **WHEN** a hidden (dotfile) folder contains a Markdown file and the Show-hidden-files preference is off
- **THEN** neither the hidden folder nor any of its contents appear in the tree, because the scan never enters a skipped subtree
- **AND** when the preference is turned on, the hidden folder appears alongside its Markdown child

#### Scenario: Non-hidden folders are kept regardless of their children
- **WHEN** a non-hidden folder contains only a hidden Markdown file (and no other text/Markdown content) and the Show-hidden-files preference is off
- **THEN** the folder still appears in the tree (folders are not content-pruned), while its hidden Markdown child stays hidden
- **AND** when the preference is turned on, the hidden Markdown child appears under that folder

#### Scenario: Hidden-entry visibility persists across restarts
- **WHEN** the user sets the Show-hidden-files preference on, restarts the editor, and opens the same workspace
- **THEN** hidden Markdown entries appear in the file tree without any further user action
