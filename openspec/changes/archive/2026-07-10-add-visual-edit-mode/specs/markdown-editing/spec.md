## ADDED Requirements

### Requirement: Source-backed Visual Edit mode
The editor SHALL provide a Visual Edit mode that presents common Markdown constructs in a rendered, editable form while preserving `MarkdownDocument.text` as the single canonical document representation. Visual Edit mutations SHALL update the Markdown source text through the same dirty-state, undo/redo, autosave, recovery, and per-tab isolation paths as source editing.

#### Scenario: Visual prose editing updates Markdown source
- **WHEN** the user edits visible prose inside a paragraph, heading, blockquote, or list item in Visual Edit mode
- **THEN** the corresponding Markdown source text is updated
- **AND** the document dirty flag and undo history are updated through the existing document mutation path

#### Scenario: Visual formatting actions remain source-backed
- **WHEN** the user applies bold, italic, inline code, link, image, heading, list, task list, blockquote, or fenced-code formatting in Visual Edit mode
- **THEN** the editor updates the underlying Markdown markers in `MarkdownDocument.text`
- **AND** switching to Edit mode shows Markdown source that represents the visual result

#### Scenario: Focused syntax can be exposed for editing
- **WHEN** the cursor enters visually formatted inline content whose hidden Markdown syntax is needed for precise editing
- **THEN** the editor SHALL expose the relevant source syntax or a source-backed edit island for that focused content

#### Scenario: Complex constructs use conservative edit islands
- **WHEN** the user focuses a fenced code block, math block, HTML/front matter region, image, or other construct not supported by direct visual editing in v1
- **THEN** the editor SHALL provide a source-backed editing affordance or preserve the existing source editing workflow
- **AND** the construct SHALL NOT be mutated through an ambiguous rendered-tree edit

#### Scenario: Visual-only interaction does not reparse unnecessarily
- **WHEN** the user moves the cursor, changes selection, hovers text, or focuses a visual edit island without changing document text
- **THEN** the document version SHALL remain unchanged
- **AND** derived Markdown caches SHALL NOT be invalidated

## MODIFIED Requirements

### Requirement: Editor view modes
The editor SHALL provide four mutually exclusive view modes: Edit, Visual Edit, Split Preview, and Read. Edit mode SHALL show the Markdown source editing surface without the rendered preview pane. Visual Edit mode SHALL show a single source-backed visual editing surface where common Markdown constructs render close to their preview appearance while remaining editable. Split Preview mode SHALL show the Markdown source editing surface and rendered preview pane together, preserving the current live-preview workflow. Read mode SHALL show the rendered Markdown preview without the source editing pane and SHALL NOT allow editing through the rendered preview.

#### Scenario: Edit mode shows only source editing
- **WHEN** the active view mode is Edit
- **THEN** the source editing surface is visible and accepts normal editing operations
- **AND** the rendered preview pane is not visible

#### Scenario: Visual Edit mode shows one editable visual surface
- **WHEN** the active view mode is Visual Edit
- **THEN** the editor shows a single visual editing surface
- **AND** common Markdown prose constructs are rendered visually where supported
- **AND** edits continue to mutate the underlying Markdown source text

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
- **WHEN** the user switches between Edit, Visual Edit, Split Preview, and Read for an open document
- **THEN** the document text, dirty flag, cursor/selection, undo/redo history, editor scroll position, preview scroll position, and tab identity are preserved
- **AND** derived preview blocks, outline, stats, syntax highlighting, visual edit blocks, and cached text handles continue to follow the existing per-document-version cache rules

### Requirement: View mode switching shortcuts
The editor SHALL provide keyboard shortcuts for switching to each view mode directly, using platform-appropriate modifier conventions. The editor MAY also retain an existing shortcut that cycles through the view modes.

#### Scenario: Direct shortcut enters Edit mode
- **WHEN** the user presses the Edit mode shortcut
- **THEN** the active view mode becomes Edit
- **AND** status feedback identifies Edit mode

#### Scenario: Direct shortcut enters Visual Edit mode
- **WHEN** the user presses the Visual Edit mode shortcut
- **THEN** the active view mode becomes Visual Edit
- **AND** status feedback identifies Visual Edit mode

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
