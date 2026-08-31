# tables-outline

## Purpose

Covers GFM table rendering, the row/column editing toolbars, and the document outline panel. Direct cell-level visual table editing is **not** part of this capability — it is a future candidate.
## Requirements
### Requirement: GFM table rendering with row/column toolbar editing
The editor SHALL render GFM tables as visual tables in the preview and Visual Edit surfaces. Tables in Split Preview and Read mode SHALL render as read-only visual grids without a table editing header or add, delete, or move row/column controls. Visual Edit SHALL provide directly editable cells plus a toolbar to add, delete, and move rows and columns of the corresponding source table, and source table commands SHALL remain available. Each cell edit SHALL produce one deterministic GFM table source replacement, preserve row ordering and declared alignments, escape field-terminating input safely, and return the exact new source selection for the active cell. Table cell alignment is parsed from the separator row and used by the LaTeX/HTML exporters.

Inline formatting inside table cells (bold, italic, strikethrough, inline code, highlight, superscript, subscript, and links) SHALL render in Split Preview, Read mode, and Visual Edit. In Visual Edit, an unfocused table cell SHALL display rendered inline formatting; a focused cell SHALL reveal the authored source markup (e.g. `**bold**`, `[text](url)`) so the user edits the canonical Markdown directly. Editing a cell continues to target the cell's exact source range and produce one deterministic table replacement through the existing history and dirty-state path.

#### Scenario: GFM table renders as a visual table
- **WHEN** the document contains a GFM-style table
- **THEN** Split Preview and Read mode render it as a visual grid

#### Scenario: Preview tables expose no editing controls
- **WHEN** a GFM table is rendered in Split Preview or Read mode
- **THEN** the table has no editable cells, editing header, or add, delete, or move row/column controls
- **AND** interacting with the preview table does not mutate the document text

#### Scenario: Inline formatting renders in preview table cells
- **WHEN** a table cell contains inline markup such as `**bold**` or `[text](url)`
- **THEN** Split Preview and Read mode render that markup as styled text (bold weight, colored underlined link, etc.) rather than literal source characters

#### Scenario: Visual Edit table cells render inline formatting while unfocused
- **WHEN** a table cell contains inline markup and the cell is not focused for editing
- **THEN** Visual Edit displays the rendered formatting (e.g. bold text, clickable link) in that cell

#### Scenario: Visual Edit table cells reveal source markup when focused
- **WHEN** the user focuses a table cell containing inline markup for editing
- **THEN** the cell displays the authored source markup (e.g. `**bold**`, `[text](url)`)
- **AND** the caret and selection map to exact positions in the canonical source
- **AND** edits produce one deterministic table source replacement through the existing history path

#### Scenario: Visual Edit table cells are directly editable
- **WHEN** the user focuses a header or body cell in a Visual Edit table
- **THEN** platform text input and IME edit that cell's source text in place
- **AND** the canonical source table is replaced once through the existing history and dirty-state path
- **AND** the resulting source selection remains in the same logical cell

#### Scenario: Cell traversal remains inside the visual grid
- **WHEN** the user presses Tab or Shift-Tab from a directly editable table cell
- **THEN** focus and the canonical source selection move to the next or previous logical cell
- **AND** traversal at the grid boundary hands control to the adjacent visual block without creating an implicit row

#### Scenario: Row and column operations via the Visual Edit toolbar
- **WHEN** the user clicks an add, delete, or move row/column button on a Visual Edit table's toolbar
- **THEN** the corresponding source table is updated through the existing source-table edit path
- **AND** the visual editing surface re-renders from the updated Markdown source

#### Scenario: Row and column operations via source commands
- **WHEN** the user invokes a source table command to format or add, delete, or move a row or column
- **THEN** the source Markdown table is reformatted or edited accordingly

#### Scenario: Alignment survives direct cell edits
- **WHEN** a table's separator row declares column alignments and a header or body cell is edited directly
- **THEN** the replacement table preserves those alignment markers semantically
- **AND** the LaTeX and HTML exporters continue to emit the declared alignment

#### Scenario: Unsafe or ambiguous table syntax falls back
- **WHEN** exact cell boundaries or a deterministic lossless table replacement cannot be proven
- **THEN** Visual Edit keeps the complete table source-backed
- **AND** it does not apply a guessed cell mutation

### Requirement: Document outline navigation
The editor SHALL provide a toggleable outline panel that lists the document's heading hierarchy, supports context-aware click-to-jump navigation, highlights the heading for the section containing the canonical cursor, and updates as headings change. In Read mode, clicking an outline heading label SHALL move the canonical cursor to that heading's source position, highlight that outline item, and bring the corresponding rendered heading into view in the preview pane. In Edit, Visual Edit, and Split Preview modes, heading-label clicks SHALL retain their existing editable-surface source-position navigation.

The outline SHALL present the heading hierarchy as an indented, collapsible tree. A heading that owns one or more following headings at deeper levels before the next heading at its own or a shallower level SHALL expose a disclosure control. A newly opened document outline SHALL start fully expanded. Activating a disclosure control SHALL collapse or expand that heading's descendant rows without invoking heading navigation; re-expanding an ancestor SHALL preserve any independently collapsed nested sections. Folding state SHALL remain isolated per open document and session-only.

The outline SHALL render compact rows with no extra inter-row margin and no more than 2px total vertical padding for a single-line row. When the visible heading list exceeds the panel height, the outline SHALL scroll vertically so every currently visible heading remains reachable by mouse-wheel or trackpad input. Folding SHALL affect presentation only and MUST NOT mutate Markdown, document version, dirty state, selection, or undo/redo history, and MUST NOT require recomputing the document's derived outline for an unchanged document version.

#### Scenario: Outline lists headings and tracks the document
- **WHEN** the outline panel is visible
- **THEN** it lists the current document's headings with hierarchy indentation and updates when headings are added, removed, or changed
- **AND** obsolete folding identities do not hide unrelated headings after the hierarchy changes

#### Scenario: Outline starts fully expanded
- **WHEN** a document is newly opened or created and its outline is shown
- **THEN** every heading is visible regardless of depth
- **AND** every heading with descendants shows an expanded disclosure state

#### Scenario: Collapse a heading subtree
- **WHEN** the user activates the expanded disclosure control for a heading with descendants
- **THEN** every consecutive descendant heading up to the next heading at the same or a shallower level is hidden
- **AND** the collapsed heading remains visible with a collapsed disclosure state
- **AND** no heading navigation occurs

#### Scenario: Expand a heading subtree
- **WHEN** the user activates the collapsed disclosure control for a heading
- **THEN** its descendant rows become visible again
- **AND** nested headings that the user independently collapsed remain collapsed

#### Scenario: Leaf headings have no disclosure action
- **WHEN** a heading has no descendant heading in the outline hierarchy
- **THEN** its row has no actionable disclosure control
- **AND** its label remains aligned with sibling heading labels

#### Scenario: Folding state is isolated per document
- **WHEN** the user collapses a section in one document and switches between open document tabs
- **THEN** each document retains its own outline folding state for the current session
- **AND** collapsing one document does not hide headings in another document

#### Scenario: Click to jump outside Read mode
- **WHEN** the user clicks a heading label in the outline while Edit, Visual Edit, or Split Preview mode is active
- **THEN** the active editable surface navigates to that heading's source position as it did before this change
- **AND** the click does not change the heading's folding state

#### Scenario: Click to jump in Read mode
- **WHEN** the user clicks a heading label in the outline while Read mode is active
- **THEN** the preview pane brings the rendered heading for that outline item into view
- **AND** the canonical cursor moves to the clicked heading's source position
- **AND** the clicked heading becomes the active outline item
- **AND** the click does not change the heading's folding state

#### Scenario: Active section is inside a collapsed subtree
- **WHEN** the canonical cursor's active heading is hidden beneath a collapsed ancestor
- **THEN** the nearest visible collapsed ancestor is highlighted as containing the active section
- **AND** cursor movement alone does not discard the user's collapsed state

#### Scenario: Outline interactions are non-mutating
- **WHEN** the user navigates, collapses, or expands headings through the outline
- **THEN** the document text, version, dirty state, selection, and undo/redo history remain unchanged

#### Scenario: Outline rows use compact vertical spacing
- **WHEN** the outline contains consecutive visible single-line headings
- **THEN** each row has no extra inter-row margin and no more than 2px total vertical padding
- **AND** hierarchy indentation, readable labels, disclosure affordances, hover feedback, active highlighting, and click targets remain intact

#### Scenario: Overflowing outline is vertically scrollable
- **WHEN** the expanded portions of the outline contain more headings than fit in the visible sidebar height
- **THEN** mouse-wheel or trackpad input over the outline scrolls its visible heading rows vertically
- **AND** every currently visible heading can be brought into view and activated

### Requirement: GFM tables render at their authored source position

The editor SHALL render every GFM pipe table that the CommonMark+GFM parser emits as a visual table at the table’s authored source position in Split Preview, Read mode, and Visual Edit. This includes one-column tables (a single header cell and delimiter row, with or without body rows) mixed in the same document with later multi-column tables. A table SHALL NOT appear earlier in the rendered stream than its source offset, and later tables SHALL NOT inherit source ranges that belong to earlier tables. Two-or-more-column tables SHALL keep their existing cell-editing and toolbar behavior; one-column tables MAY remain non-editable at the cell/toolbar layer when exact cell bounds cannot be proven.

#### Scenario: One-column command tables stay in place

- **WHEN** the document contains a one-column GFM table such as `| command |\n| --- |` between surrounding prose or headings
- **THEN** Split Preview, Read mode, and Visual Edit render that table as a visual grid at that source location
- **AND** the table does not appear at the start of the document unless it is authored there

#### Scenario: Later multi-column result tables are not hoisted

- **WHEN** a document begins with headings separated only by blank lines and contains `Dies | Throughput` (or other multi-column) tables much later, after earlier one-column GFM tables
- **THEN** those later tables render at their authored offsets
- **AND** they do not appear between the leading headings whose source gap contains only whitespace

#### Scenario: Two-column cell editing still targets the caret’s table

- **WHEN** the user edits a cell or uses the Visual Edit table toolbar on a two-or-more-column GFM table after preview table ranges are taken from parser events
- **THEN** the mutation still replaces that table’s source bytes
- **AND** it does not edit a different table in the document

