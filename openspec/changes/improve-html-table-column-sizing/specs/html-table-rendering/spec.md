## ADDED Requirements

### Requirement: HTML table columns scale with content

The rendered grid for a raw HTML `<table>` SHALL size its columns proportionally to cell content instead of equal fractions: a column whose covering cells are all empty (or near-empty) SHALL be rendered narrower than a column carrying wider content, approximating browser auto table layout. Placement SHALL remain exact: every cell keeps its resolved column/row coordinates and spans regardless of column widths.

#### Scenario: Cover table with empty spacer columns

- **WHEN** the document contains an HTML table where some logical columns hold only empty cells (e.g. a Word-exported cover sheet with empty `<th>`/`<td>` spacers around a few content cells)
- **THEN** the empty columns render as narrow slivers and the content columns receive the remaining width
- **AND** spanning cells (`rowspan`/`colspan`) still cover exactly their resolved rows and columns

#### Scenario: Fully populated table keeps balanced layout

- **WHEN** an HTML table's columns all carry comparable content
- **THEN** the rendered column widths stay approximately equal, matching the previous behavior

#### Scenario: Degenerate weights fall back to equal tracks

- **WHEN** a table has no visible content in any cell (or a weight computation that would be degenerate)
- **THEN** the table renders with equal column tracks rather than collapsing

### Requirement: Header emphasis requires visible header content

A `<th>` cell SHALL render with header emphasis (weight and shading) only when its row contains at least one header cell with visible content (non-whitespace text or an image); header rows whose cells are all empty SHALL render as body cells. Parsing and export behavior SHALL be unchanged — this governs visual presentation only.

#### Scenario: All-empty header frame renders as body

- **WHEN** an HTML table's first row consists entirely of empty `<th>` cells (a cover-sheet frame)
- **THEN** those cells render without header shading or weight, as plain body cells

#### Scenario: Matrix corner cell keeps header styling

- **WHEN** a header row contains an empty leading `<th>` beside `<th>` cells with content
- **THEN** the entire row's header cells, including the empty corner cell, render with header emphasis
