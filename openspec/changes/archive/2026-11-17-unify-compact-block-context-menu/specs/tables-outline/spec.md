## MODIFIED Requirements

### Requirement: Visual Edit table cells expose in-cell inline format controls
When Visual Edit owns a non-empty selection that lies entirely inside one table cell’s exact source range, the block context menu for that table SHALL expose the existing Bold, Italic, Inline Code, Strikethrough, and Link formatting group. Invoking a control SHALL wrap or unwrap that cell’s selected bytes through the existing Markdown format path as one undoable mutation. A selection that crosses a cell boundary, a `|` delimiter, or more than one cell SHALL NOT expose those actions and SHALL NOT apply inline wrapping that would break the table. Focused cells continue to reveal authored source markup; unfocused cells continue to render inline formatting.

#### Scenario: Cell selection shows Bold in the context menu

- **WHEN** the user selects a non-empty range fully inside one Visual Edit table cell and opens that table’s context menu
- **THEN** the menu includes Bold, Italic, Inline Code, Strikethrough, and Link
- **AND** opening the menu does not change document version

#### Scenario: Bold wraps only the cell selection

- **WHEN** the user invokes Bold for a selection fully inside one table cell
- **THEN** that cell’s selected source is wrapped with `**` as one undoable mutation
- **AND** neighboring cells and table structure are unchanged

#### Scenario: Cross-cell selection stays conservative

- **WHEN** the selection starts in one table cell and ends in another cell or includes a `|` delimiter
- **THEN** the context menu does not expose selection formatting actions for that range
- **AND** applying Bold through the keyboard format command does not mutate the document

## ADDED Requirements

### Requirement: Compact table-specific context menus
A table-cell editing context menu SHALL omit block type conversions and block/image insertion controls. It SHALL offer each inline Link action only once and retain selection clipboard/formatting plus separate Row and Column entries whose bounded side flyouts contain their respective existing checked operations. Whole-table operations SHALL remain at the root, with labels naming Table explicitly. Hover SHALL open either flyout without a click; Right/Enter and click SHALL remain supported. Pointer, keyboard, back navigation, scrolling and click-away SHALL preserve source, selection and caches, including short windows and left-flipped flyouts; stale submenu targets SHALL be rejected just like root targets.

#### Scenario: Safe cell catalog and separate axis flyouts
- **WHEN** the user opens a context menu inside a table cell
- **THEN** heading, list, quote, code-block, divider, table and image insertion/conversion controls are absent and Link occurs at most once
- **AND** separate Row and Column entries open only their respective axis operations beside the root without a left click, while whole-table operations remain directly available at the root
- **AND** Left or Back returns to the root, while outside clicks and tab changes dismiss the complete menu

### Requirement: Shift+Enter inserts a cell-local hard break
In Visual Edit, Shift+Enter within an exactly ranged header or body cell SHALL insert an HTML `<br>` at the caret or replace the cell-local selection with that break as one undoable edit. The caret SHALL remain in that cell after the break. Unfocused rendering SHALL display a line break. Other cells, pipe delimiters and surrounding source bytes SHALL remain unchanged. Ordinary Enter and Tab traversal, Shift+Enter search navigation and source-mode newline behavior SHALL remain available.

#### Scenario: Break within a Unicode cell and undo
- **WHEN** the user presses Shift+Enter inside a header or body cell containing Unicode text
- **THEN** only the selected range becomes `<br>` and the caret remains immediately after it in the same cell
- **AND** subsequent typing stays in that cell, and one Undo restores the original break edit and selection

### Requirement: Visual Edit context menu exposes exact row and column actions
The combined Visual Edit context menu SHALL retain whole-table block commands and selection clipboard/formatting while offering row and column action groups for the clicked exact cell. Keyboard invocation SHALL use the caret-owning cell or current structured cell-selection head. Row actions SHALL include insert above/below, move up/down, duplicate, clear and delete; column actions SHALL include insert left/right, move left/right, alignment, duplicate, clear and delete. These actions SHALL reuse the established header, table-edge and lone-column restrictions, capture document identity/version, table identity/shape and row/column indices, and reject stale or ambiguous targets. Selected text or rectangles SHALL remain the clipboard/inline-editing target even if the cell used for structural commands differs. Menu lifecycle SHALL not mutate source, selection, history, dirty state or derived caches; every successful structural action SHALL perform one checked source mutation and one undo step, preserving unrelated tables and content.

#### Scenario: Right-click a cell while the caret is elsewhere
- **WHEN** the user right-clicks row 2, column 1 of an editable table while the caret belongs to another cell or block
- **THEN** row/column groups target exactly row 2 and column 1 of the clicked table
- **AND** clipboard and inline formatting retain any active selection

#### Scenario: Keyboard menu uses the selected cell
- **WHEN** the user invokes the keyboard context menu while a table cell owns the caret or a rectangle is selected
- **THEN** structural groups target the caret cell or rectangle head cell respectively
- **AND** clipboard operations still act on the selected text or rectangle

#### Scenario: Structural restrictions and stale targets
- **WHEN** a header/edge/lone-column action is unavailable, or document identity/version or captured table shape is stale
- **THEN** the action is disabled or rejected without changing source, selection, history or caches

#### Scenario: Exact row or column mutation and undo
- **WHEN** a row or column action is activated for an exact live cell
- **THEN** only its captured table is edited with existing alignment and column-width metadata behavior
- **AND** one Undo restores the original source and selection
