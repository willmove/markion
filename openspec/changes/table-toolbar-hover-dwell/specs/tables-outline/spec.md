## MODIFIED Requirements

### Requirement: GFM table rendering with row/column toolbar editing
The editor SHALL render GFM tables as visual tables in the preview and Visual Edit surfaces. Tables in Split Preview and Read mode SHALL render as read-only visual grids without a table editing header or add, delete, or move row/column controls. Visual Edit SHALL provide directly editable cells plus a table-editing header that can add, delete, and move rows and columns of the corresponding source table, delete the entire table through the existing exact block-delete path, and source table commands SHALL remain available. The Visual Edit table-editing header SHALL be hidden by default and SHALL be shown only while the pointer is over that table's chrome (including the header itself) or the canonical caret belongs to a cell in that table. Hover-triggered showing SHALL be gated behind a fixed continuous hover dwell of 250 ms so a pointer merely passing over a table does not show the header; once shown, hover-triggered hiding SHALL be gated behind a fixed continuous absence of 120 ms so a brief excursion off the chrome does not hide it. Caret-triggered showing SHALL be immediate, with no dwell. Both delays SHALL be fixed constants, not user-configurable. Showing or hiding the header SHALL NOT mutate document text, dirty state, undo history, document version, or derived Markdown caches. Each cell edit SHALL produce one deterministic GFM table source replacement, preserve row ordering and declared alignments, escape field-terminating input safely, and return the exact new source selection for the active cell. Table cell alignment is parsed from the separator row and used by the LaTeX/HTML exporters.

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
- **WHEN** the pointer stays continuously over a Visual Edit table's chrome for 250 ms and the canonical caret does not belong to that table
- **THEN** that table's editing header is shown
- **AND** showing the header does not mutate document text, dirty state, undo history, or document version
- **AND** row and column controls remain disabled until a cell in that table owns the caret

#### Scenario: Pointer passing over a table shows no header
- **WHEN** the pointer enters a Visual Edit table's chrome and leaves again before the 250 ms hover dwell has elapsed, and the canonical caret does not belong to that table
- **THEN** that table's editing header is not shown
- **AND** the rendered layout does not shift

#### Scenario: Shown header survives a brief pointer excursion
- **WHEN** a Visual Edit table's editing header is shown from hover and the pointer leaves the table's chrome for less than 120 ms before returning
- **THEN** that table's editing header remains shown without hiding and re-showing

#### Scenario: Shown hover header hides after the pointer stays away
- **WHEN** a Visual Edit table's editing header is shown from hover and the pointer stays continuously off that table's chrome for 120 ms while the canonical caret does not belong to that table
- **THEN** that table's editing header is hidden

#### Scenario: Visual Edit table editing header appears when a cell is focused
- **WHEN** the user clicks a header or body cell in a Visual Edit table so the canonical caret belongs to that cell
- **THEN** that table's editing header is shown immediately, without waiting for the hover dwell
- **AND** it remains shown even if the pointer later leaves the table
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
