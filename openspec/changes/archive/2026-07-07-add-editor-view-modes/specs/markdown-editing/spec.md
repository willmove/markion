## ADDED Requirements

### Requirement: Editor view modes
The editor SHALL provide three mutually exclusive view modes: Edit, Split Preview, and Read. Edit mode SHALL show the Markdown source editing surface without the rendered preview pane. Split Preview mode SHALL show the Markdown source editing surface and rendered preview pane together, preserving the current live-preview workflow. Read mode SHALL show the rendered Markdown preview without the source editing pane and SHALL NOT allow editing through the rendered preview.

#### Scenario: Edit mode shows only source editing
- **WHEN** the active view mode is Edit
- **THEN** the source editing surface is visible and accepts normal editing operations
- **AND** the rendered preview pane is not visible

#### Scenario: Split Preview mode shows both panes
- **WHEN** the active view mode is Split Preview
- **THEN** the source editing surface and rendered preview pane are both visible
- **AND** edits in the source surface continue to update the preview through the existing derived Markdown state

#### Scenario: Read mode shows only rendered Markdown
- **WHEN** the active view mode is Read
- **THEN** the rendered preview pane is visible
- **AND** the source editing surface is not visible
- **AND** interacting with rendered preview content does not mutate the document text

#### Scenario: Mode switching preserves document state
- **WHEN** the user switches between Edit, Split Preview, and Read for an open document
- **THEN** the document text, dirty flag, cursor/selection, undo/redo history, editor scroll position, preview scroll position, and tab identity are preserved
- **AND** derived preview blocks, outline, stats, syntax highlighting, and cached text handles continue to follow the existing per-document-version cache rules

### Requirement: View mode switching shortcuts
The editor SHALL provide keyboard shortcuts for switching to each view mode directly, using platform-appropriate modifier conventions. The editor MAY also retain an existing shortcut that cycles through the view modes.

#### Scenario: Direct shortcut enters Edit mode
- **WHEN** the user presses the Edit mode shortcut
- **THEN** the active view mode becomes Edit
- **AND** status feedback identifies Edit mode

#### Scenario: Direct shortcut enters Split Preview mode
- **WHEN** the user presses the Split Preview mode shortcut
- **THEN** the active view mode becomes Split Preview
- **AND** status feedback identifies Split Preview mode

#### Scenario: Direct shortcut enters Read mode
- **WHEN** the user presses the Read mode shortcut
- **THEN** the active view mode becomes Read
- **AND** status feedback identifies Read mode

#### Scenario: Mode shortcuts follow platform conventions
- **WHEN** the editor runs on macOS versus Windows/Linux
- **THEN** the view mode shortcuts use the same `secondary` modifier convention as other application shortcuts
