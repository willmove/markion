## ADDED Requirements

### Requirement: Table block grips expose whole-table commands

The Visual Edit block grip for a Markdown table SHALL offer Copy Table, Cut Table, Make a Copy, Move Up, Move Down, Insert Paragraph Below Table, and a separated Delete Table action. A top-level table with exact source ownership SHALL offer Wrap in Quote when it is outside a quote. The table grip SHALL omit generic text, heading, list, and code conversions and cell-local link, formula, image and paste actions. This catalog specializes the general block grip settings for tables. Block actions use the established complete source unit, which is the enclosing quote for a table within a quote.

#### Scenario: Table grip opens a compact whole-table menu

- **WHEN** the user clicks an editable table's block grip in Visual Edit
- **THEN** the root contains the available whole-table commands and a labeled Insert Below entry
- **AND** the menu omits generic conversions and cell-local shortcuts
- **AND** a move past the document edge remains visible and disabled

#### Scenario: Insert a divider or new table below the current table

- **WHEN** the user opens Insert Below from a standalone table grip using pointer or keyboard navigation
- **THEN** the flyout offers labeled divider and new-table commands
- **AND** activating either adds a separate block after the current table in one undoable edit and preserves the original table and its neighbors
