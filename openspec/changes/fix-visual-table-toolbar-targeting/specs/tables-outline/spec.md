## MODIFIED Requirements

### Requirement: GFM table rendering with row/column toolbar editing
The editor SHALL render GFM tables as visual tables in the preview and Visual Edit surfaces. Tables in Split Preview and Read mode SHALL render as read-only visual grids without a table editing header or add, delete, or move row/column controls. Visual Edit SHALL provide directly editable cells plus a toolbar to add, delete, and move rows and columns of the corresponding source table, and source table commands SHALL remain available. A Visual Edit toolbar action SHALL resolve its target from the cell containing the canonical caret endpoint in that same table at activation time and SHALL NOT replace that target with the table's first source offset. Row additions SHALL insert immediately after the active row, including creating the first body row when the header is active; row deletion and movement SHALL target the active body row. Column additions SHALL insert immediately after the active column, and column deletion SHALL target the active column. Controls without a valid active-cell target or whose operation is invalid at the active structural boundary SHALL be visibly and interactively disabled. Each successful toolbar or cell edit SHALL produce one deterministic GFM table source replacement through the existing history and dirty-state path, preserve row ordering and declared alignments, escape field-terminating input safely, and return the exact new source selection for the resulting logical cell. Table cell alignment is parsed from the separator row and used by the LaTeX/HTML exporters.

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

#### Scenario: Row operation targets the active body row
- **WHEN** the canonical caret is in a body cell and the user activates add row, delete row, move row up, or move row down on that table's Visual Edit toolbar
- **THEN** the operation targets that cell's logical row rather than the header or a fixed body row
- **AND** add row inserts immediately after it while delete and move act on that row
- **AND** the resulting selection remains at the same logical column in the inserted, surviving, or moved row

#### Scenario: Adding a row from the header creates the first body row
- **WHEN** the canonical caret is in a header cell and the user activates add row
- **THEN** a new first body row is inserted immediately after the separator row and before every existing body row
- **AND** the resulting selection is in the new row at the header cell's logical column

#### Scenario: Column operation targets the active column
- **WHEN** the canonical caret is in a table cell and the user activates add column or delete column on that table's Visual Edit toolbar
- **THEN** add column inserts immediately after that cell's logical column and delete column removes that logical column
- **AND** the resulting selection remains in the active row at the inserted or nearest surviving column

#### Scenario: Toolbar target is isolated to its owning table
- **WHEN** a document contains multiple tables and the canonical caret belongs to a cell in one table
- **THEN** only that table's toolbar has an active target
- **AND** activating or inspecting another table's toolbar cannot mutate either table through a guessed table-start offset

#### Scenario: Structurally invalid toolbar operations are disabled
- **WHEN** the active cell is in the header, first body row, last body row, or the only remaining column
- **THEN** delete/move controls that cannot validly operate at that boundary are visibly and interactively disabled
- **AND** an unavailable action does not change source text, selection, dirty state, document version, or undo history

#### Scenario: Toolbar has no active cell target
- **WHEN** the canonical caret does not belong to a cell in a Visual Edit table
- **THEN** that table's row and column controls are visibly and interactively disabled
- **AND** the toolbar does not substitute the header's first cell or any other default target

#### Scenario: Successful toolbar operation is one canonical edit
- **WHEN** the user activates an available row or column control
- **THEN** the corresponding source table is replaced exactly once through the existing table edit path
- **AND** the visual editing surface re-renders from the updated Markdown source
- **AND** one undo restores the pre-operation source and one redo reapplies the operation

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
