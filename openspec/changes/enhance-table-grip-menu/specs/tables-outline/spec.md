## MODIFIED Requirements

### Requirement: Table block grips expose whole-table commands

The Visual Edit block grip for a Markdown table SHALL offer Copy Table, Cut Table, Make a Copy, Move Up, Move Down, a labeled Insert Below flyout entry, and a separated Delete Table action. A top-level editable table with two or more columns (up to 100) SHALL offer Equalize Column Widths, which writes or updates the table's leading `<!-- markion-cols:… -->` comment to equal integer percents summing to 100 in one undoable edit (or leaves the document unchanged when already equalized). A top-level table with exact source ownership SHALL offer Wrap in Quote when it is outside a quote. Paragraph insertion SHALL live inside the Insert Below flyout rather than as a separate root item. The Insert Below flyout SHALL offer the same compact grouped catalog as the paragraph context menu's Insert flyout—Paragraph and H1–H5, bulleted/numbered/task lists, quote, code block, divider, Table, the five GFM callouts, and Link, Formula, and Image—inserting the selected block or confirmed Link/Formula/Image content as a new block below the table's complete source unit in one undoable edit. Canceling a Link, Formula, or Image prompt opened from Insert Below SHALL leave document text, version, selection, undo history, and shared derived caches unchanged. The table grip SHALL omit generic text, heading, list, and code conversions and cell-local paste actions. Block actions use the established complete source unit, which is the enclosing quote for a table within a quote.

#### Scenario: Table grip opens a compact whole-table menu with Equalize Column Widths

- **WHEN** the user clicks an editable top-level multi-column table's block grip in Visual Edit
- **THEN** the root contains Copy, Cut, Duplicate, Move Up, Move Down, Equalize Column Widths, Wrap in Quote, a labeled Insert Below entry, and a separated Delete Table action
- **AND** paragraph insertion is not a separate root item
- **AND** the menu omits generic conversions and cell-local shortcuts
- **AND** a move past the document edge remains visible and disabled

#### Scenario: Equalize Column Widths persists equal shares and undoes atomically

- **WHEN** the user activates Equalize Column Widths on a top-level three-column table (with or without an existing `markion-cols` comment)
- **THEN** the source immediately before the table contains `<!-- markion-cols:34,33,33 -->`
- **AND** Visual Edit, Split Preview, and Read mode apply those equal column shares
- **AND** one Undo restores the exact previous source and selection

#### Scenario: Equalize Column Widths is omitted on single-column and quoted tables

- **WHEN** the user opens the block grip menu for a one-column table or a table inside a blockquote
- **THEN** Equalize Column Widths is omitted from the root menu

#### Scenario: Insert Below flyout matches the compact Insert catalog

- **WHEN** the user opens Insert Below from a table block grip using pointer or keyboard navigation
- **THEN** the flyout presents Paragraph and H1–H5 as a compact six-button row, lists/quote/code/divider as a compact six-button row, a labeled Table row, a compact five-callout row, and labeled Link, Formula, and Image rows
- **AND** activating any block or callout item inserts that block after the table's complete source unit in one undoable edit while preserving the original table and its neighbors
- **AND** activating Link, Formula, or Image opens the corresponding insertion prompt without mutating the document; canceling leaves the document untouched and confirming inserts the formatted content as a separate block below the table in one undoable edit
