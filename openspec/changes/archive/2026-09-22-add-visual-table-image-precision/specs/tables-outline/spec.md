## ADDED Requirements

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
