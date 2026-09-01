## ADDED Requirements

### Requirement: HTML table cells occupy resolved grid lines

The rendered HTML table grid SHALL place every non-spacer cell on the column and row tracks implied by the resolved `HtmlTableGrid` (the same coordinates the parser uses after `rowspan`/`colspan` occupancy). Placement SHALL use explicit grid lines (start and exclusive end), not CSS-grid auto-placement. Two cells that occupy different columns of the same row SHALL paint as separate columns in Split Preview, Read mode, and Visual Edit — their text SHALL NOT concatenate into one visual run. Column-width weights SHALL apply to those tracks and SHALL NOT change coordinates.

#### Scenario: Cover-sheet label and value stay in separate columns

- **WHEN** the document contains an HTML table equivalent to a Word cover sheet with a left `rowspan` strut, a right `rowspan` strut, and a body row `<td>文档版本</td><td colspan="2">01</td>`
- **THEN** Split Preview, Read mode, and Visual Edit render `文档版本` and `01` in two distinct columns of the same row
- **AND** the two strings are not presented as a single concatenated run such as `文档版本01`
- **AND** each spanning empty strut still occupies its resolved columns and rows

#### Scenario: Datasheet rowspan still spans its column

- **WHEN** the document contains an HTML table where a body cell declares `rowspan="3"` beside per-row cells (the existing `12 V` / peak-current shape)
- **THEN** that cell still visually spans three rows in its column
- **AND** the following rows' remaining cells stay in the columns after the span

### Requirement: Empty spacer cells do not paint as cards

A resolved HTML table cell that has no visible text and no cell image SHALL still occupy its grid tracks so spanning geometry is preserved, but it SHALL NOT paint as a padded, filled, internally stroked card. Cells that have visible text or an image SHALL keep the existing pipe-table padding, fill, and internal strokes. Parser spacer slots (covered by an earlier `rowspan`) remain undrawn.

#### Scenario: Empty cover-sheet struts are quiet

- **WHEN** the document contains an HTML table whose first row is empty `<th>` cells and whose side columns are empty `rowspan` struts
- **THEN** those empty cells do not appear as a stack of blank padded bands
- **AND** content cells in the interior columns remain visible as a table

#### Scenario: Content cells keep table chrome

- **WHEN** an HTML table cell contains visible text or an image
- **THEN** that cell still renders with the existing table padding and internal grid strokes

### Requirement: Visual Edit HTML table uses one table chrome

When Visual Edit presents a raw HTML block whose preview parts include an HTML table grid, the table SHALL use the same grid presentation as Read mode and SHALL NOT be wrapped in a second rounded table-style border. A collapsible source payload (`</>` / caret-in-block) MAY still reveal the authored HTML. HTML blocks that do not include a table MAY keep the existing bordered collapsible chrome.

#### Scenario: Visual Edit table matches Read grid chrome

- **WHEN** Visual Edit displays a raw HTML `<table>` that parses as a table grid
- **THEN** the table renders as one bordered grid (not a card per row and not a card around a card)
- **AND** focusing the block can still reveal the authored HTML source payload
