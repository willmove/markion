## MODIFIED Requirements

### Requirement: GFM table rendering with row/column handle editing
The editor SHALL render GFM tables as visual tables in the preview and Visual Edit surfaces. Tables in Split Preview and Read mode SHALL render as read-only visual grids without row/column handles, edge add affordances, table menus, or any add, delete, or move row/column controls. Visual Edit SHALL provide directly editable cells plus overlaid structural controls: a row handle for each row, a column handle for each column, a row menu and a column menu opened from those handles, drag-to-reorder from those handles, and edge add affordances below and to the right of the table. Source table commands SHALL remain available. Whole-table delete, duplicate, and move SHALL remain available through the existing block grip and block context menu.

A row handle SHALL be shown only while the pointer is over a cell of that row, and a column handle SHALL be shown only while the pointer is over a cell of that column; a handle whose menu is open SHALL stay shown while the menu is open. Edge add affordances SHALL be shown only while the pointer is over that table or the affordances themselves. Handles, edge affordances, drop indicators, and menus SHALL be overlays: showing, hiding, or hovering them SHALL NOT change the size or position of the table, its cells, or any other rendered content, and SHALL NOT mutate document text, dirty state, undo history, document version, or derived Markdown caches.

The row menu SHALL offer Insert above, Insert below, Move up, Move down, Duplicate, Clear contents, and Delete. For the header row, only Insert below and Clear contents SHALL be enabled. For the first body row, Move up SHALL be disabled; for the last row, Move down SHALL be disabled. The column menu SHALL offer Insert left, Insert right, Move left, Move right, Align left, Align center, Align right, Duplicate, Clear contents, and Delete. Move left SHALL be disabled for the first column, Move right SHALL be disabled for the last column, and Delete SHALL be disabled when the table has one column. The alignment item matching the column's declared alignment SHALL be marked current; choosing it again SHALL reset the column to the default (undeclared) alignment. Disabled items SHALL be visibly and interactively disabled.

Each handle, menu, drag, or edge action SHALL target the row or column index captured when the handle or affordance was rendered. It SHALL be revalidated against the current document version, table identity, and row and column counts before it is applied, and it SHALL do nothing if that validation fails. Each successful action SHALL produce one deterministic GFM table source replacement through the existing history and dirty-state path: exactly one undo step and one document version increment. It SHALL preserve the declared alignments of unaffected columns and the contents and order of unaffected rows, and it SHALL place the canonical source selection in a cell of the affected row or column. Ordinary cell text edits SHALL patch only the exact cell field range, preserve unrelated table bytes, row ordering and declared alignments, escape field-terminating input safely, and return the exact new source selection. Explicit Format and structural table commands MAY reformat the table. Table cell alignment is parsed from the separator row and used by the LaTeX/HTML exporters.

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
- **AND** Tab at the final cell appends one empty body row as one undoable source edit and selects its first cell; Shift-Tab at the first cell hands control to the preceding visual block

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


## ADDED Requirements

### Requirement: Rectangular GFM cell selection and matrix editing
Visual Edit SHALL support rectangular cell selection, including whole rows and columns. Single-column GFM tables SHALL support the same direct-cell, row and matrix editing commands. Selected cells SHALL be visibly highlighted and carry exact table identity/version. Copy SHALL publish a TSV matrix and HTML table; Delete SHALL clear only selected cells without removing table structure. Pasting a TSV matrix into a selected rectangle or active cell SHALL fill cells in row order, escape cell delimiters/newlines, and append required body rows without removing the GFM header. Each clear or paste SHALL be one checked undoable edit. Stale targets and unproven cell ranges SHALL remain unchanged.

#### Scenario: Clear and restore a rectangle
- **WHEN** the user selects a rectangle and presses Delete
- **THEN** only the selected cells are cleared and one Undo restores all original cells and alignments

#### Scenario: Paste a spreadsheet matrix
- **WHEN** a two-by-two TSV matrix is pasted at a selected cell
- **THEN** four target cells receive the corresponding escaped values in one undoable operation
- **AND** required rows are added without changing unrelated cells
