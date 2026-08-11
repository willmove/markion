## Purpose

Render raw HTML `<table>` blocks embedded in Markdown as visual tables in the preview and Read mode, including cells that span multiple rows or columns via `rowspan`/`colspan`, so datasheet-style tables that cannot be expressed as GFM pipe tables display correctly instead of collapsing to a flat text run.

## ADDED Requirements

### Requirement: Raw HTML tables render as visual tables

The editor SHALL detect a raw HTML `<table>...</table>` block in the document and render it in Split Preview and Read mode as a visual grid with the same borders, header emphasis, and padding as GFM pipe tables, rather than flattening its cell text into a single inline run. The rendered table SHALL be read-only in Split Preview and Read mode and SHALL NOT mutate the document text on interaction.

Supported row-organizing elements: `<table>`, `<thead>`, `<tbody>`, `<tfoot>`, `<tr>`. Supported cell elements: `<th>` (header cell, bold/emphasized) and `<td>` (body cell). Unknown nested elements inside a cell SHALL have their inline text content rendered as the cell's text.

#### Scenario: Basic HTML table renders as a grid

- **WHEN** the document contains a raw HTML `<table>` with one header row and one body row
- **THEN** Split Preview and Read mode render it as a bordered visual table
- **AND** header cells (`<th>`) appear visually distinct from body cells (`<td>`)

#### Scenario: HTML table is read-only in preview

- **WHEN** a raw HTML table is rendered in Split Preview or Read mode
- **THEN** the table has no editable cells, editing header, or add, delete, or move row/column controls
- **AND** interacting with the table does not mutate the document text

#### Scenario: Inline formatting renders inside HTML table cells

- **WHEN** an HTML table cell contains inline markup such as `**bold**`, `*italic*`, `` `code` ``, or `[text](url)` (parsed by the same inline pipeline as the rest of the document)
- **THEN** Split Preview and Read mode render that markup as styled text rather than literal source characters

### Requirement: Rowspan and colspan expand the grid

The editor SHALL read the `rowspan` and `colspan` attributes on `<th>` and `<td>` elements (defaulting each to `1` when absent or unparseable) and expand the rendered grid so a spanning cell occupies the corresponding number of rows and/or columns. A cell with `rowspan="N"` SHALL visually span N consecutive rows in its column, and a cell with `colspan="M"` SHALL visually span M consecutive columns in its row. Overlapping or out-of-range spans SHALL be clamped to the table bounds rather than causing a crash or an empty render.

#### Scenario: rowspan repeats a cell down its column

- **WHEN** the document contains an HTML table where a `<td>` declares `rowspan="3"`
- **THEN** the rendered grid places that cell's content in a single cell that visually spans three rows
- **AND** the following rows have their remaining cells shifted into the correct columns

#### Scenario: colspan widens a cell across columns

- **WHEN** the document contains an HTML table where a `<td>` or `<th>` declares `colspan="2"`
- **THEN** the rendered grid places that cell's content in a single cell that visually spans two columns
- **AND** the row's subsequent cells align to the columns after the span

#### Scenario: Combined rowspan and colspan

- **WHEN** the document contains an HTML table where a single cell declares both `rowspan="2"` and `colspan="2"`
- **THEN** the rendered grid places that cell's content in a single cell spanning two rows and two columns

#### Scenario: Invalid span values fall back to single cell

- **WHEN** a `rowspan` or `colspan` attribute holds a non-numeric, zero, or negative value
- **THEN** the attribute is treated as `1` and the cell occupies a single row and/or column

### Requirement: Malformed HTML tables fall back safely

When a raw HTML `<table>` block is malformed (unclosed tags, mismatched nesting, or a structure that cannot be resolved into a grid), the editor SHALL fall back to the existing flattened-text rendering for that block instead of panicking, producing an empty preview, or dropping the content.

#### Scenario: Unclosed table tags fall back to flattened text

- **WHEN** the document contains a raw HTML `<table>` with unbalanced or unclosed row/cell tags that cannot be resolved into a grid
- **THEN** the preview renders the block's text content using the existing HTML-block flattener behavior
- **AND** the editor does not panic or render an empty block

### Requirement: HTML tables are cached per document version

Parsing an HTML table into a grid SHALL be part of the per-document-version derived preview state computed once per version and shared via `Arc`, consistent with the existing `PreviewBlock::Html` flattening. The editor SHALL NOT reparse an HTML table on every keystroke; it SHALL reuse the cached result until the document version changes.

#### Scenario: Editing outside a table does not reparse it

- **WHEN** the user types in a paragraph that is not inside an HTML table block
- **THEN** the previously computed HTML table preview for an unchanged table block is reused from the per-version cache
- **AND** the table is not reparsed
