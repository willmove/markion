## MODIFIED Requirements

### Requirement: GFM table rendering with row/column toolbar editing
The editor SHALL render GFM tables as visual tables in the preview and Visual Edit surfaces. Tables in Split Preview and Read mode SHALL render as read-only visual grids without a table editing header or add, delete, or move row/column controls. Visual Edit SHALL provide directly editable plain-text cells plus a toolbar to add, delete, and move rows and columns of the corresponding source table, and source table commands SHALL remain available. Each cell edit SHALL produce one deterministic GFM table source replacement, preserve row ordering and declared alignments, escape field-terminating input safely, and return the exact new source selection for the active cell. Table cell alignment is parsed from the separator row and used by the LaTeX/HTML exporters; inline styles inside table cells are not required to render in the Visual Edit grid, though the HTML export keeps full fidelity.

#### Scenario: GFM table renders as a visual table
- **WHEN** the document contains a GFM-style table
- **THEN** Split Preview and Read mode render it as a visual grid

#### Scenario: Preview tables expose no editing controls
- **WHEN** a GFM table is rendered in Split Preview or Read mode
- **THEN** the table has no editable cells, editing header, or add, delete, or move row/column controls
- **AND** interacting with the preview table does not mutate the document text

#### Scenario: Visual Edit table cells are directly editable
- **WHEN** the user focuses a header or body cell in a Visual Edit table
- **THEN** platform text input and IME edit that cell's plain-text value in place
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
