# tables-outline

## Purpose

Covers GFM table rendering, the row/column editing toolbars, and the document outline panel. Direct cell-level visual table editing is **not** part of this capability — it is a future candidate.

## Requirements

### Requirement: Document outline navigation
The editor SHALL provide a toggleable outline panel that lists the document's heading hierarchy, supports context-aware click-to-jump navigation, highlights the heading for the section containing the canonical cursor, and updates as headings change. In Read mode, clicking an outline heading label SHALL move the canonical cursor to that heading's source position, highlight that outline item, and bring the corresponding rendered heading into view in the preview pane. In Edit, Visual Edit, and Split Preview modes, heading-label clicks SHALL retain their existing editable-surface source-position navigation.

The outline SHALL present the heading hierarchy as an indented, collapsible tree. A heading that owns one or more following headings at deeper levels before the next heading at its own or a shallower level SHALL expose a disclosure control. A newly opened document outline SHALL start fully expanded. Activating a disclosure control SHALL collapse or expand that heading's descendant rows without invoking heading navigation; re-expanding an ancestor SHALL preserve any independently collapsed nested sections. Folding state SHALL remain isolated per open document and session-only.

The outline SHALL render compact rows with no extra inter-row margin and no more than 2px total vertical padding for a single-line row. When the visible heading list exceeds the panel height, the outline SHALL scroll vertically so every currently visible heading remains reachable by mouse-wheel or trackpad input and by dragging a visible right-side vertical scrollbar. The scrollbar thumb SHALL hide when the visible heading list fits or the active tab is an image. Folding SHALL affect presentation only and MUST NOT mutate Markdown, document version, dirty state, selection, or undo/redo history, and MUST NOT require recomputing the document's derived outline for an unchanged document version.

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
- **AND** a right-side vertical scrollbar thumb is shown and can be dragged with the left mouse button to scroll the same rows
- **AND** every currently visible heading can be brought into view and activated

#### Scenario: Fitting outline hides the scrollbar
- **WHEN** the outline panel is visible
- **AND** the expanded heading rows fit in the visible sidebar height, or the active tab is an image
- **THEN** no vertical scrollbar thumb is shown for the outline

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

### Requirement: GFM table columns size from cell content

The editor SHALL allocate GFM pipe-table column widths from recommended content widths of the cells in each column, not from an equal split, in Visual Edit, Split Preview, and Read mode. The table SHALL remain stretched to the document content column. A column whose cells are short SHALL receive a smaller share than a column whose cells are long, so a typical two-column name/description table presents a narrow first column and a wider second column. Long cell text SHALL wrap inside its allocated column rather than overflowing the pane. Visual Edit and the read-only preview grid SHALL use the same width recommendation for the same table rows.

Column recommendations SHALL be derived from each cell's rendered plain text (not from focused Visual Edit source markup, and not from GFM source pipe padding). Computing or applying those widths SHALL NOT mutate document text, dirty state, undo/redo history, or per-document-version derived Markdown caches. Typography-driven reflow MAY change the pixel sizes without bumping document version.

One-column tables SHALL continue to occupy the full content column. Empty or missing cells SHALL NOT collapse their column below a readable minimum share.

No column in a table of `n` columns (`n` ≥ 2) SHALL receive more than three equal-shares (`3 / n` of the recommended width sum), except when that column’s header minimum itself exceeds the cap. Header recommendations SHALL keep a parenthesis pair whose inner non-whitespace text is 1–3 characters on one line when that unit fits, and SHALL be wide enough that the header’s recommended wrap is at most three lines.

#### Scenario: Unequal content yields unequal columns

- **WHEN** a GFM table has a short-text column (for example header `名称` and cells such as `操作系统`) beside a long-text column (for example header `说明` and a multi-word technical description)
- **THEN** Visual Edit, Split Preview, and Read mode render the short column narrower than the long column
- **AND** the two columns are not an equal-width split of the table

#### Scenario: Visual Edit matches Read mode

- **WHEN** the same GFM table is shown in Visual Edit and in Read mode
- **THEN** both surfaces use the same per-column width recommendation for those rows

#### Scenario: Focused source markup does not reflow columns

- **WHEN** the user focuses a Visual Edit table cell whose authored source is longer than its rendered plain text (for example `**bold**`)
- **THEN** column width shares stay based on the rendered cell text
- **AND** focusing the cell does not widen that column solely because source markup is visible

#### Scenario: Table stays within the content column

- **WHEN** a GFM table is rendered in Visual Edit, Split Preview, or Read mode
- **THEN** the table still spans the document content column
- **AND** cell text that exceeds its allocated column wraps inside the cell instead of overflowing the pane

#### Scenario: Layout is presentation-only

- **WHEN** column widths are computed or applied for a GFM table
- **THEN** the document text, dirty flag, undo/redo history, and per-document-version derived Markdown caches remain unchanged

#### Scenario: One-column and empty cells remain usable

- **WHEN** a GFM table has a single column, or a column whose cells are empty
- **THEN** that column still occupies a usable share of the table
- **AND** a one-column table still spans the document content column

#### Scenario: Long body columns cannot exceed three equal-shares

- **WHEN** a GFM table has six columns and two of them contain paragraph-length body text
- **THEN** neither column’s recommended share exceeds half of the table (`3 / 6`)
- **AND** a short unit header such as `实际功率（W）` still receives a larger share than it would under an uncapped linear split of the same preferred widths

#### Scenario: Short header parenthesis units stay together

- **WHEN** a table header cell contains a parenthesis pair whose inner non-whitespace text is 1–3 characters (for example `实际功率（W）` or `Power (W)`)
- **THEN** that column’s recommended minimum is at least as wide as the parenthesis unit plus cell padding
- **AND** Visual Edit and Read mode use that same minimum

#### Scenario: Header wrap stays within three recommended lines

- **WHEN** a table header cell’s unwrapped content is wider than three times a single-line glyph run
- **THEN** that column’s recommended minimum is at least one third of the unwrapped header content width plus cell padding
