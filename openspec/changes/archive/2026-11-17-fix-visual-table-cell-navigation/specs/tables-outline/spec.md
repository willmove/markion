## ADDED Requirements

### Requirement: Visual Edit table vertical navigation preserves the column

In an editable GFM table, Up/Down and their selection variants SHALL first move between painted lines inside the current cell. At the cell's upper or lower line boundary, they SHALL enter the previous or next logical row in the same column, including empty cells and the header, retaining the preferred horizontal position. The delimiter row SHALL NOT be a navigation stop. Only movement beyond the first or last logical row SHALL hand control to the adjacent visual block, without creating a row or mutating Markdown.

#### Scenario: Down traverses populated and empty rows
- **WHEN** the caret reaches the last painted line of a cell and the user presses Down
- **THEN** the caret enters the next row's cell in the same column, including an empty cell
- **AND** repeated Down reaches each existing empty row before leaving the table

#### Scenario: Up returns to the header
- **WHEN** the caret reaches the first painted line of a body cell and the user presses Up
- **THEN** the caret enters the preceding logical row in the same column
- **AND** the header is reachable without stopping on the Markdown delimiter row

#### Scenario: Wrapped cell keeps internal navigation
- **WHEN** the current cell has an adjacent painted line in the requested direction
- **THEN** Up/Down stays in that cell and retains the preferred horizontal position

#### Scenario: Selection uses the same cell targets
- **WHEN** the user presses Shift-Up or Shift-Down within an editable table
- **THEN** the selection head follows the same wrapped-line and same-column navigation targets
- **AND** the source selection stays normalized and UTF-8 safe

#### Scenario: Table boundary hands off without edits
- **WHEN** the user moves Up beyond the header or Down beyond the last body row
- **THEN** the caret moves to the adjacent visual block when one exists
- **AND** text, document version, dirty state, history, and shared derived Markdown caches remain unchanged

### Requirement: Empty Visual Edit table cells remain clickable and paint a caret

Each editable GFM table cell SHALL accept left-click caret placement throughout its interior, including empty content and padding, except on explicit structural controls. Text clicks SHALL retain precise text hit testing. An empty cell SHALL paint a visible caret at its source insertion position and accept text input and IME through the existing cell-editing path. Pointer placement and empty-cell geometry SHALL NOT insert placeholder bytes into Markdown or change document version, dirty state, history, or shared derived Markdown caches.

#### Scenario: Click an empty cell
- **WHEN** the user left-clicks the interior of an empty header or body cell
- **THEN** the source caret belongs to that cell and is visibly painted inside it
- **AND** typing edits that logical cell and Undo restores the prior source

#### Scenario: Click cell padding
- **WHEN** the user left-clicks cell padding outside its painted text and structural controls
- **THEN** the caret is placed inside that cell
- **AND** clicking its text continues to resolve the precise source-backed text position

#### Scenario: Empty cell reached by keyboard
- **WHEN** Tab, Shift-Tab, Up, or Down enters an empty cell
- **THEN** the cell paints a visible caret without adding source text

#### Scenario: Structural controls keep their actions
- **WHEN** the user activates a row grip, column grip, or resize handle
- **THEN** the intended structural interaction is retained without a cell-padding handler overriding it
