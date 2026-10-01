## REMOVED Requirements

### Requirement: GFM table rendering with row/column toolbar editing
**Reason**: The in-flow Visual Edit table editing header shifted document layout on hover and caret entry; it is replaced by overlaid row/column handles, menus, drag reorder, and edge add affordances.
**Migration**: Row/column operations move to the row and column handle menus and the edge add strips; whole-table delete/duplicate/move stay on the block grip and block context menu. All other behavior of this requirement is carried over unchanged by "GFM table rendering with row/column handle editing".

## ADDED Requirements

### Requirement: GFM table rendering with row/column handle editing
The editor SHALL render GFM tables as visual tables in the preview and Visual Edit surfaces. Tables in Split Preview and Read mode SHALL render as read-only visual grids without row/column handles, edge add affordances, table menus, or any add, delete, or move row/column controls. Visual Edit SHALL provide directly editable cells plus overlaid structural controls: a row handle for each row, a column handle for each column, a row menu and a column menu opened from those handles, drag-to-reorder from those handles, and edge add affordances below and to the right of the table. Source table commands SHALL remain available. Whole-table delete, duplicate, and move SHALL remain available through the existing block grip and block context menu.

A row handle SHALL be shown only while the pointer is over a cell of that row, and a column handle SHALL be shown only while the pointer is over a cell of that column; a handle whose menu is open SHALL stay shown while the menu is open. Edge add affordances SHALL be shown only while the pointer is over that table or the affordances themselves. Handles, edge affordances, drop indicators, and menus SHALL be overlays: showing, hiding, or hovering them SHALL NOT change the size or position of the table, its cells, or any other rendered content, and SHALL NOT mutate document text, dirty state, undo history, document version, or derived Markdown caches.

The row menu SHALL offer Insert above, Insert below, Move up, Move down, Duplicate, Clear contents, and Delete. For the header row, only Insert below and Clear contents SHALL be enabled. For the first body row, Move up SHALL be disabled; for the last row, Move down SHALL be disabled. The column menu SHALL offer Insert left, Insert right, Move left, Move right, Align left, Align center, Align right, Duplicate, Clear contents, and Delete. Move left SHALL be disabled for the first column, Move right SHALL be disabled for the last column, and Delete SHALL be disabled when the table has one column. The alignment item matching the column's declared alignment SHALL be marked current; choosing it again SHALL reset the column to the default (undeclared) alignment. Disabled items SHALL be visibly and interactively disabled.

Each handle, menu, drag, or edge action SHALL target the row or column index captured when the handle or affordance was rendered. It SHALL be revalidated against the current document version, table identity, and row and column counts before it is applied, and it SHALL do nothing if that validation fails. Each successful action SHALL produce one deterministic GFM table source replacement through the existing history and dirty-state path: exactly one undo step and one document version increment. It SHALL preserve the declared alignments of unaffected columns and the contents and order of unaffected rows, and it SHALL place the canonical source selection in a cell of the affected row or column. Each cell edit SHALL produce one deterministic GFM table source replacement, preserve row ordering and declared alignments, escape field-terminating input safely, and return the exact new source selection for the active cell. Table cell alignment is parsed from the separator row and used by the LaTeX/HTML exporters.

Inline formatting inside table cells (bold, italic, strikethrough, inline code, highlight, superscript, subscript, and links) SHALL render in Split Preview, Read mode, and Visual Edit. In Visual Edit, an unfocused table cell SHALL display rendered inline formatting; a focused cell SHALL reveal the authored source markup (e.g. `**bold**`, `[text](url)`) so the user edits the canonical Markdown directly. Editing a cell continues to target the cell's exact source range and produce one deterministic table replacement through the existing history and dirty-state path.

#### Scenario: GFM table renders as a visual table
- **WHEN** the document contains a GFM-style table
- **THEN** Split Preview and Read mode render it as a visual grid

#### Scenario: Preview tables expose no editing controls
- **WHEN** a GFM table is rendered in Split Preview or Read mode
- **THEN** the table has no editable cells, row or column handles, edge add affordances, or table menus
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

#### Scenario: No in-flow editing header
- **WHEN** the pointer enters, moves within, or leaves a Visual Edit table, or the canonical caret enters or leaves one of its cells
- **THEN** no editing row is inserted into or removed from the table
- **AND** the table and all content below it keep their rendered positions

#### Scenario: Hovering a cell reveals its row and column handles
- **WHEN** the pointer is over the body cell at row 2, column 1 of a Visual Edit table
- **THEN** the row handle for row 2 and the column handle for column 1 are shown
- **AND** no other row or column handle of that table is shown
- **AND** the table's layout and the document version are unchanged

#### Scenario: Edge add affordances append a row or a column
- **WHEN** the pointer is over a Visual Edit table and the user clicks the add strip below the table
- **THEN** an empty body row is appended after the last row as one undoable source mutation
- **AND** clicking the add strip to the right of the table instead appends an empty column after the last column with default alignment

#### Scenario: Row menu inserts, duplicates, clears, moves, and deletes body rows
- **WHEN** the user opens the row handle menu for a body row and chooses Insert above, Insert below, Duplicate, Clear contents, Move up, Move down, or Delete
- **THEN** the source table is rewritten with that row operation applied to the targeted row only
- **AND** the edit is one undo step and one document version increment

#### Scenario: Header row menu protects the GFM header
- **WHEN** the user opens the row handle menu for the header row
- **THEN** Insert below and Clear contents are enabled
- **AND** Insert above, Move up, Move down, Duplicate, and Delete are disabled and activating them does not mutate the document

#### Scenario: Column menu edits the targeted column
- **WHEN** the user opens a column handle menu and chooses Insert left, Insert right, Move left, Move right, Duplicate, Clear contents, or Delete
- **THEN** every row of the source table, including the header and separator, is rewritten with that column operation applied to the targeted column only
- **AND** the edit is one undo step and one document version increment

#### Scenario: Column alignment from the column menu
- **WHEN** the user chooses Align center from a column menu for a column with default alignment
- **THEN** that column's separator cell declares center alignment
- **AND** the Align center item is marked current the next time that menu is opened
- **AND** choosing Align center again resets the column to default alignment

#### Scenario: Dragging a row handle reorders body rows
- **WHEN** the user drags the handle of body row 3 and drops it on body row 1
- **THEN** row 3's cells become body row 1 and the former rows 1 and 2 shift down by one
- **AND** the header row is unchanged and the edit is one undo step
- **AND** dropping a row on itself, or dragging the header row, does not mutate the document

#### Scenario: Dragging a column handle reorders columns
- **WHEN** the user drags the handle of column 0 and drops it on column 2 of a three-column table
- **THEN** column 0's cells and alignment move to index 2 and the former columns 1 and 2 shift left by one
- **AND** the edit is one undo step

#### Scenario: Stale structural target does nothing
- **WHEN** a handle, menu, drag, or edge action targets a table whose document version, identity, or row or column count changed after it was rendered
- **THEN** the action does not mutate document text, selection, dirty state, document version, or undo history

#### Scenario: Structural actions are isolated to their table
- **WHEN** a document contains several tables and the user applies a handle or edge action to one of them
- **THEN** only that table's source changes
- **AND** neighboring tables and unrelated source bytes are unchanged

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

## MODIFIED Requirements

### Requirement: GFM tables support draggable persisted column widths

Visual Edit SHALL expose a column-resize handle on the boundary between adjacent cells of a GFM table that has a table cell editor and at least two columns. Pointer-move while dragging SHALL update that table’s on-screen column shares without mutating document text, dirty state, undo history, document version, or per-version derived Markdown caches. Releasing the drag SHALL write or replace one HTML comment immediately before the pipe table as a single undoable source mutation:

`<!-- markion-cols:P1,P2,… -->`

where each `Pi` is an integer percent, there is one value per column, values sum to 100, and each value is at least the per-column floor (5, or 1 when `n * 5 > 100`). Dragging a handle SHALL change only the two columns sharing that boundary; other columns keep their percents. Split Preview and Read mode SHALL apply the same authored percents as Visual Edit. When the comment is missing, malformed, or its value count does not match the table’s column count, surfaces SHALL use the existing content-heuristic column weights.

The comment SHALL NOT appear as a separate Html preview or Visual Edit block when it immediately precedes a GFM table at the same nesting. The table block’s source range SHALL include the comment so whole-table delete and duplicate keep comment and table together. Cell editing and structural row/column actions SHALL continue to target pipe-table bytes, not the comment, except that a structural column action rewrites a matching comment as part of the same mutation.

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

- **WHEN** a table already has a matching column-width comment and the user inserts, duplicates, or deletes a column through a column handle menu or the right edge add affordance
- **THEN** that same source mutation rewrites the percent list to the new column count
- **AND** no second undo step is introduced solely for the comment

#### Scenario: Moving a column carries its width

- **WHEN** a table has `<!-- markion-cols:20,30,50 -->` and the user moves column 0 to index 2 through the column menu or by dragging its handle
- **THEN** the same source mutation rewrites the comment to `<!-- markion-cols:30,50,20 -->`
