## MODIFIED Requirements

### Requirement: GFM table rendering with row/column toolbar editing
The editor SHALL render GFM tables as visual tables in the preview and Visual Edit surfaces. Tables in Split Preview and Read mode SHALL render as read-only visual grids without a table editing header or add, delete, or move row/column controls. Visual Edit SHALL provide directly editable cells plus a table-editing header that can add, delete, and move rows and columns of the corresponding source table, delete the entire table through the existing exact block-delete path, and source table commands SHALL remain available. The Visual Edit table-editing header SHALL be hidden by default and SHALL be shown only while the pointer is over that table's chrome (including the header itself) or the canonical caret belongs to a cell in that table. Showing or hiding the header SHALL NOT mutate document text, dirty state, undo history, document version, or derived Markdown caches. Each cell edit SHALL produce one deterministic GFM table source replacement, preserve row ordering and declared alignments, escape field-terminating input safely, and return the exact new source selection for the active cell. Table cell alignment is parsed from the separator row and used by the LaTeX/HTML exporters.

Inline formatting inside table cells (bold, italic, strikethrough, inline code, highlight, superscript, subscript, and links) SHALL render in Split Preview, Read mode, and Visual Edit. In Visual Edit, an unfocused table cell SHALL display rendered inline formatting; a focused cell SHALL reveal the authored source markup (e.g. `**bold**`, `[text](url)`) so the user edits the canonical Markdown directly. Editing a cell continues to target the cell's exact source range and produce one deterministic table replacement through the existing history and dirty-state path.

A Visual Edit table with proven cell editors SHALL additionally expose the same on-demand raw-source affordance used by block math, diagrams, and raw-HTML blocks: a hover-visible source toggle expands one monospaced payload editor covering the complete authored table source below the rendered grid. While the raw source is expanded, the grid SHALL be read-only presentation — cell editors and the row/column controls SHALL NOT be active, the canonical caret SHALL be routed into the payload, and edits SHALL apply as one exact canonical source replacement through the existing history and dirty-state paths. Activating a cell while the raw source is expanded SHALL collapse the payload and place the caret in that cell against the current document version. Expanding and collapsing SHALL be presentation-only: they MUST NOT mutate document text, dirty state, undo history, document version, or derived Markdown caches. Tables whose exact structure cannot be proven keep the existing complete source-backed fallback and get no raw-source toggle.

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
- **WHEN** the user presses Tab or Shift+Tab from a directly editable table cell
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

#### Scenario: Raw table source expands below the rendered grid
- **WHEN** the user activates the hover source toggle on a Visual Edit table with proven cell editors
- **THEN** a monospaced payload editor shows the complete authored table source as one editable field below the rendered grid
- **AND** the grid above becomes read-only presentation with no active cell editors or row/column controls

#### Scenario: Raw table payload edit is one atomic replacement
- **WHEN** the user edits the raw payload — for example changing the separator row's alignment markers or pasting additional pipe rows
- **THEN** the edit applies as one exact canonical source replacement through the existing history and dirty-state path
- **AND** after re-parse the grid and exporters reflect the edited alignments and rows

#### Scenario: Cell activation collapses the raw view
- **WHEN** the raw source is expanded and the user clicks a cell in the rendered grid
- **THEN** the raw payload collapses and the caret lands in that cell of the current document version
- **AND** while the caret remains inside the raw payload the table stays expanded, and an outside click collapses it

#### Scenario: Raw table source toggling is presentation-only
- **WHEN** the user expands or collapses a table's raw source payload
- **THEN** document text, document version, dirty state, and undo history are unchanged
- **AND** derived Markdown caches are not invalidated by the toggle itself
