# tables-outline

## Purpose

Covers GFM table rendering, the row/column editing toolbars, and the document outline panel. Direct cell-level visual table editing is **not** part of this capability — it is a future candidate.
## Requirements
### Requirement: GFM table rendering with row/column toolbar editing
The editor SHALL render GFM tables as visual tables in the preview and Visual Edit surfaces. Tables in Split Preview and Read mode SHALL render as read-only visual grids without a table editing header or add, delete, or move row/column controls. Visual Edit SHALL provide directly editable cells plus a table-editing header that can add, delete, and move rows and columns of the corresponding source table, delete the entire table through the existing exact block-delete path, and source table commands SHALL remain available. The Visual Edit table-editing header SHALL be hidden by default and SHALL be shown only while the pointer is over that table's chrome (including the header itself) or the canonical caret belongs to a cell in that table. Showing or hiding the header SHALL NOT mutate document text, dirty state, undo history, document version, or derived Markdown caches. Each cell edit SHALL produce one deterministic GFM table source replacement, preserve row ordering and declared alignments, escape field-terminating input safely, and return the exact new source selection for the active cell. Table cell alignment is parsed from the separator row and used by the LaTeX/HTML exporters.

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

#### Scenario: Visual Edit table editing header is hidden while idle
- **WHEN** a Visual Edit table is rendered, the pointer is not over that table, and the canonical caret does not belong to a cell in that table
- **THEN** that table's editing header (row/column controls and whole-table delete) is not shown
- **AND** document text, dirty state, undo history, and document version remain unchanged

#### Scenario: Visual Edit table editing header appears on hover
- **WHEN** the pointer is over a Visual Edit table's chrome and the canonical caret does not belong to that table
- **THEN** that table's editing header is shown
- **AND** showing the header does not mutate document text, dirty state, undo history, or document version
- **AND** row and column controls remain disabled until a cell in that table owns the caret

#### Scenario: Visual Edit table editing header appears when a cell is focused
- **WHEN** the user clicks a header or body cell in a Visual Edit table so the canonical caret belongs to that cell
- **THEN** that table's editing header remains shown even if the pointer later leaves the table
- **AND** the header hides after the caret leaves every cell of that table and the pointer is not over it

#### Scenario: Row and column operations via the Visual Edit toolbar
- **WHEN** the user clicks an add, delete, or move row/column button on a Visual Edit table's visible toolbar
- **THEN** the corresponding source table is updated through the existing source-table edit path
- **AND** the visual editing surface re-renders from the updated Markdown source

#### Scenario: Whole-table delete via the Visual Edit toolbar
- **WHEN** the user activates the delete-table control on a Visual Edit table whose exact block delete is supported
- **THEN** the complete table source unit is removed through the existing block-delete path
- **AND** one undo restores the prior source and selection
- **AND** neighboring tables and unrelated source bytes are unchanged

#### Scenario: Unsupported whole-table delete is disabled
- **WHEN** exact block delete is not supported for a Visual Edit table (nested or ambiguous ownership)
- **THEN** the delete-table control is visibly and interactively disabled
- **AND** activating it does not change source text, selection, dirty state, document version, or undo history

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

### Requirement: Outline heading anchors match in-document hash links
Each outline heading SHALL expose a stable `anchor` string that in-document `#fragment` links resolve against. The id SHALL be the authored heading attribute `{#id}` when pulldown-cmark reports one; otherwise a Unicode-preserving slug of the visible title. Empty titles SHALL use `section`. Duplicate ids in document order SHALL uniquify with a numeric suffix (`hello`, `hello-1`). Changing folding, hovering, or clicking the outline SHALL still not mutate Markdown.

#### Scenario: Outline anchors follow authored ids
- **WHEN** a heading is authored as `## Title {#custom}`
- **THEN** that outline item’s `anchor` is `custom`

#### Scenario: Duplicate outline titles uniquify
- **WHEN** two headings share the visible title `Hello` and neither has an authored id
- **THEN** their outline anchors are `hello` and `hello-1` in document order

### Requirement: GFM tables render at their authored source position

The editor SHALL render every GFM pipe table that the CommonMark+GFM parser emits as a visual table at the table’s authored source position in Split Preview, Read mode, and Visual Edit. This includes one-column tables (a single header cell and delimiter row, with or without body rows) mixed in the same document with later multi-column tables. A table SHALL NOT appear earlier in the rendered stream than its source offset, and later tables SHALL NOT inherit source ranges that belong to earlier tables. Two-or-more-column tables SHALL keep their cell-editing and interaction-gated toolbar behavior, including whole-table delete when exact block delete is supported; one-column tables MAY remain non-editable at the cell/toolbar layer when exact cell bounds cannot be proven.

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

### Requirement: GFM tables support draggable persisted column widths

Visual Edit SHALL expose a column-resize handle on the boundary between adjacent cells of a GFM table that has a table cell editor and at least two columns. Pointer-move while dragging SHALL update that table’s on-screen column shares without mutating document text, dirty state, undo history, document version, or per-version derived Markdown caches. Releasing the drag SHALL write or replace one HTML comment immediately before the pipe table as a single undoable source mutation:

`<!-- markion-cols:P1,P2,… -->`

where each `Pi` is an integer percent, there is one value per column, values sum to 100, and each value is at least the per-column floor (5, or 1 when `n * 5 > 100`). Dragging a handle SHALL change only the two columns sharing that boundary; other columns keep their percents. Split Preview and Read mode SHALL apply the same authored percents as Visual Edit. When the comment is missing, malformed, or its value count does not match the table’s column count, surfaces SHALL use the existing content-heuristic column weights.

The comment SHALL NOT appear as a separate Html preview or Visual Edit block when it immediately precedes a GFM table at the same nesting. The table block’s source range SHALL include the comment so whole-table delete and duplicate keep comment and table together. Cell editing and the row/column toolbar SHALL continue to target pipe-table bytes, not the comment.

#### Scenario: Drag overlay does not bump document version

- **WHEN** the user drags a Visual Edit table column handle without releasing
- **THEN** on-screen column shares follow the pointer
- **AND** document text, dirty state, undo history, and document version remain unchanged

#### Scenario: Mouse-up writes one column-width comment

- **WHEN** the user releases a column-resize drag on a two-or-more-column GFM table
- **THEN** the source contains `<!-- markion-cols:… -->` immediately before that table
- **AND** the edit is one undoable mutation that restores the prior source including the absence or previous comment

#### Scenario: Authored percents apply on Read and Visual Edit

- **WHEN** the document contains `<!-- markion-cols:30,70 -->` immediately before a two-column GFM table
- **THEN** Visual Edit, Split Preview, and Read mode give the first column 30% and the second 70% of the table
- **AND** the comment is not rendered as an Html island or raw comment row

#### Scenario: Missing or mismatched comment keeps the heuristic

- **WHEN** a GFM table has no column-width comment, or the comment lists a different number of values than the table has columns
- **THEN** column shares fall back to the content-heuristic weights
- **AND** the table remains editable

#### Scenario: Structural column edits keep an existing comment in sync

- **WHEN** a table already has a matching column-width comment and the user adds or deletes a column through the Visual Edit toolbar
- **THEN** that same source mutation rewrites the percent list to the new column count
- **AND** no second undo step is introduced solely for the comment

### Requirement: Visual Edit table cells expose in-cell inline format controls

When Visual Edit owns a non-empty selection that lies entirely inside one table cell’s exact source range, the block context menu for that table SHALL expose the existing Bold, Italic, Inline Code, and Link formatting group. Invoking a control SHALL wrap or unwrap that cell’s selected bytes through the existing Markdown format path as one undoable mutation. A selection that crosses a cell boundary, a `|` delimiter, or more than one cell SHALL NOT expose those actions and SHALL NOT apply inline wrapping that would break the table. Focused cells continue to reveal authored source markup; unfocused cells continue to render inline formatting.

#### Scenario: Cell selection shows Bold in the context menu

- **WHEN** the user selects a non-empty range fully inside one Visual Edit table cell and opens that table’s context menu
- **THEN** the menu includes Bold, Italic, Inline Code, and Link
- **AND** opening the menu does not change document version

#### Scenario: Bold wraps only the cell selection

- **WHEN** the user invokes Bold for a selection fully inside one table cell
- **THEN** that cell’s selected source is wrapped with `**` as one undoable mutation
- **AND** neighboring cells and table structure are unchanged

#### Scenario: Cross-cell selection stays conservative

- **WHEN** the selection starts in one table cell and ends in another cell or includes a `|` delimiter
- **THEN** the context menu does not expose selection formatting actions for that range
- **AND** applying Bold through the keyboard format command does not mutate the document

